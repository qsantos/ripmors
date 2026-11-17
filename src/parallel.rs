//! Multi-threaded encoding and decoding.
//!
//! The calling thread (the producer) cuts the input into jobs, worker threads encode or decode
//! them independently, and a writer thread writes their outputs in order. The cuts are placed so
//! that the concatenation of the outputs is the same as processing the whole input at once:
//!
//! - when decoding, right after a separator, so that no Morse sequence spans two jobs;
//! - when encoding, before a byte that is not a UTF-8 continuation byte (any byte for
//!   [`Charset::Ascii`]); a job then starts with a fresh [`EncodeStream`][crate::EncodeStream],
//!   whose output only differs at the junction: the letter space held back at the end of the
//!   previous job is written, unless the job starts with whitespace, which replaces it.
//!
//! When reading from a [`Read`] object, each read is dispatched as soon as it returns (up to the
//! last cut point), so that the output follows the input promptly, as with the single-threaded
//! `*_stream` methods; large reads, when the input comes fast enough, keep the threads busy.

use std::collections::BTreeMap;
use std::io::{self, Read, Write};
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex, mpsc};

use crate::decode::is_separator;
use crate::{Charset, Decoder, Encoder};

// Job sizes are smaller in tests, to exercise more job boundaries. Larger jobs reduce the
// synchronization overhead, but their output buffers fall out of the cache, and more memory needs
// to be mapped (with page faults) for them.

/// Target number of input bytes per job when reading from a slice.
const JOB_SIZE: usize = if cfg!(test) { 1 << 12 } else { 1 << 18 };

/// Smallest number of input bytes per job when splitting a small slice between threads.
const MIN_JOB_SIZE: usize = if cfg!(test) { 1 << 10 } else { 1 << 16 };

/// Maximum number of bytes per read when reading with several threads.
const READ_SIZE: usize = if cfg!(test) { 1 << 13 } else { 1 << 19 };

/// Alignment of the destination of reads.
const READ_ALIGN: usize = 64;

/// Maximum number of bytes per read when reading with a single thread; this keeps the input in
/// the L2 cache between reading and processing it.
const SINGLE_THREAD_READ_SIZE: usize = if cfg!(test) { 1 << 11 } else { 1 << 18 };

/// How to cut the input into jobs, and how to process a job.
trait Codec: Sync {
    /// Information returned by [`Codec::work`] for the writer.
    type Info: Send;

    /// The first position from `target` (excluded: `0 < target < view.len()`) where a job can
    /// start, if any.
    fn next_cut(&self, view: &[u8], target: usize) -> Option<usize>;

    /// The last position in `view` (`view.len()` included) where a job can start, or 0.
    fn last_cut(&self, view: &[u8]) -> usize;

    /// Shorten the beginning of a job without any cut point, when this does not change the
    /// output; return its new length.
    fn compact(&self, job: &mut [u8]) -> usize {
        job.len()
    }

    /// Process a job, appending the result to `out`.
    fn work(&self, data: &[u8], out: &mut String) -> Self::Info;
}

/// Input of a job.
enum Data<'a> {
    Borrowed(&'a [u8]),
    Owned(Vec<u8>, std::ops::Range<usize>),
}

impl std::ops::Deref for Data<'_> {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        match self {
            Data::Borrowed(data) => data,
            Data::Owned(buf, range) => &buf[range.clone()],
        }
    }
}

/// Where the producer gets its input from.
enum Input<'a, R> {
    Slice(&'a [u8]),
    Reader(&'a mut R),
}

/// A counting semaphore that can be closed to wake up and fail all waiters.
struct Semaphore {
    state: Mutex<(usize, bool)>,
    cond: Condvar,
}

impl Semaphore {
    fn new(permits: usize) -> Self {
        Semaphore {
            state: Mutex::new((permits, false)),
            cond: Condvar::new(),
        }
    }

    /// Take a permit, or return `false` if the semaphore is closed.
    fn acquire(&self) -> bool {
        let mut state = self.state.lock().unwrap();
        while state.0 == 0 && !state.1 {
            state = self.cond.wait(state).unwrap();
        }
        if state.1 {
            return false;
        }
        state.0 -= 1;
        true
    }

    fn release(&self) {
        self.state.lock().unwrap().0 += 1;
        self.cond.notify_one();
    }

    fn close(&self) {
        self.state.lock().unwrap().1 = true;
        self.cond.notify_all();
    }
}

/// Closes the semaphore if a worker panics, so that the producer does not wait forever for the
/// job that will never be written.
struct PanicGuard<'a>(&'a Semaphore);

impl Drop for PanicGuard<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            self.0.close();
        }
    }
}

/// A pool of buffers, to avoid paying for page faults on each new job.
struct Pool<T>(Mutex<Vec<T>>);

impl<T: Default> Pool<T> {
    fn new() -> Self {
        Pool(Mutex::new(Vec::new()))
    }
    fn get(&self) -> T {
        self.0.lock().unwrap().pop().unwrap_or_default()
    }
    fn put(&self, buf: T) {
        self.0.lock().unwrap().push(buf);
    }
}

/// The producer: reads the input (if needed), and cuts it into jobs.
struct Producer<'a, 'c, R, C> {
    input: Input<'a, R>,
    codec: &'c C,
    /// Slice: target size of the jobs; reader: maximum size of each read.
    size: usize,
    /// Slice: offset of the next job.
    offset: usize,
    /// Reader: the input not yet dispatched, in the `filled` bytes of `buf` from `start`; the
    /// buffer is kept fully initialized to avoid initializing it before each read.
    buf: Vec<u8>,
    start: usize,
    filled: usize,
    eof: bool,
}

impl<'a, 'c, R: Read, C: Codec> Producer<'a, 'c, R, C> {
    /// Return the next job; `pool` provides the buffers for the input that is read.
    fn next(&mut self, pool: &Pool<Vec<u8>>) -> io::Result<Option<Data<'a>>> {
        let reader = match &mut self.input {
            Input::Slice(input) => {
                let input: &'a [u8] = input;
                let rest = &input[self.offset..];
                if rest.is_empty() {
                    return Ok(None);
                }
                let end = if rest.len() <= self.size {
                    rest.len()
                } else {
                    self.codec.next_cut(rest, self.size).unwrap_or(rest.len())
                };
                self.offset += end;
                return Ok(Some(Data::Borrowed(&rest[..end])));
            }
            Input::Reader(reader) => reader,
        };
        loop {
            let (start, filled) = (self.start, self.filled);
            if self.eof {
                // the rest of the input, which has no cut point
                if filled == 0 {
                    return Ok(None);
                }
                self.filled = 0;
                let buf = std::mem::take(&mut self.buf);
                return Ok(Some(Data::Owned(buf, start..start + filled)));
            }
            if self.buf.len() < start + filled + self.size {
                self.buf.resize(start + filled + self.size, 0);
            }
            let end = start + filled;
            match reader.read(&mut self.buf[end..end + self.size]) {
                Ok(0) => self.eof = true,
                Ok(n) => {
                    self.filled += n;
                    // dispatch what was read right away
                    let view = &mut self.buf[start..start + self.filled];
                    let cut = self.codec.last_cut(view);
                    if cut == 0 {
                        self.filled = self.codec.compact(view);
                        continue;
                    }
                    // move the rest of the input to a new buffer, such that the next read is
                    // aligned (which makes the copy from the kernel faster)
                    let mut next = pool.get();
                    let rest = self.filled - cut;
                    if next.len() < rest + self.size + READ_ALIGN {
                        next.resize(rest + self.size + READ_ALIGN, 0);
                    }
                    let next_start =
                        (READ_ALIGN - (next.as_ptr() as usize + rest) % READ_ALIGN) % READ_ALIGN;
                    next[next_start..next_start + rest]
                        .copy_from_slice(&self.buf[start + cut..start + self.filled]);
                    let buf = std::mem::replace(&mut self.buf, next);
                    self.start = next_start;
                    self.filled = rest;
                    return Ok(Some(Data::Owned(buf, start..start + cut)));
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
    }
}

/// Encode or decode `input` into `output` with `threads` threads; see the module documentation.
///
/// `sink` writes the output of each job, in order.
fn run<R, W, C>(
    input: Input<'_, R>,
    output: &mut W,
    threads: NonZeroUsize,
    codec: &C,
    mut sink: impl FnMut(&mut W, &str, C::Info) -> io::Result<()> + Send,
) -> io::Result<()>
where
    R: Read,
    W: Write + Send,
    C: Codec,
{
    let threads = threads.get();
    let size = match input {
        // split small inputs between all the threads
        Input::Slice(input) => (input.len() / (4 * threads)).clamp(MIN_JOB_SIZE, JOB_SIZE),
        Input::Reader(_) if threads == 1 => SINGLE_THREAD_READ_SIZE,
        Input::Reader(_) => READ_SIZE,
    };
    let single_job = matches!(input, Input::Slice(input) if input.len() <= size);
    let mut producer = Producer {
        input,
        codec,
        size,
        offset: 0,
        buf: Vec::new(),
        start: 0,
        filled: 0,
        eof: false,
    };
    let in_pool = Pool::new();

    if threads == 1 || single_job {
        let mut out = String::new();
        while let Some(data) = producer.next(&in_pool)? {
            out.clear();
            let info = codec.work(&data, &mut out);
            if let Data::Owned(buf, _) = data {
                in_pool.put(buf);
            }
            sink(output, &out, info)?;
        }
        return Ok(());
    }

    let permits = Semaphore::new(2 * threads);
    let out_pool = Pool::new();
    let (job_tx, job_rx) = mpsc::channel::<(usize, Data)>();
    let job_rx = Mutex::new(job_rx);
    let (res_tx, res_rx) = mpsc::channel::<(usize, String, C::Info)>();
    let failed = AtomicBool::new(false);
    std::thread::scope(|s| {
        for _ in 0..threads {
            let res_tx = res_tx.clone();
            let (job_rx, in_pool, out_pool) = (&job_rx, &in_pool, &out_pool);
            let guard = PanicGuard(&permits);
            s.spawn(move || {
                let _guard = guard;
                loop {
                    let job = job_rx.lock().unwrap().recv();
                    let Ok((seq, data)) = job else {
                        break;
                    };
                    let mut out: String = out_pool.get();
                    out.clear();
                    let info = codec.work(&data, &mut out);
                    if let Data::Owned(buf, _) = data {
                        in_pool.put(buf);
                    }
                    if res_tx.send((seq, out, info)).is_err() {
                        break;
                    }
                }
            });
        }
        drop(res_tx);

        let (permits, out_pool, failed) = (&permits, &out_pool, &failed);
        let writer = s.spawn(move || {
            let mut pending = BTreeMap::new();
            let mut next_seq = 0;
            let mut result = Ok(());
            for (seq, out, info) in res_rx {
                pending.insert(seq, (out, info));
                while let Some((out, info)) = pending.remove(&next_seq) {
                    next_seq += 1;
                    if !failed.load(Ordering::Relaxed)
                        && let Err(e) = sink(output, &out, info)
                    {
                        failed.store(true, Ordering::Relaxed);
                        permits.close();
                        result = Err(e);
                    }
                    out_pool.put(out);
                    permits.release();
                }
            }
            result
        });

        let mut seq = 0;
        let produced = loop {
            if !permits.acquire() {
                break Ok(());
            }
            match producer.next(&in_pool) {
                Ok(Some(data)) => {
                    job_tx.send((seq, data)).unwrap();
                    seq += 1;
                }
                Ok(None) => break Ok(()),
                Err(e) => break Err(e),
            }
        };
        drop(job_tx);
        let written = writer.join().unwrap();
        produced.and(written)
    })
}

impl Codec for Encoder {
    /// Whether the output ends with a letter space that was held back.
    type Info = bool;

    fn next_cut(&self, view: &[u8], target: usize) -> Option<usize> {
        match self.charset {
            Charset::Ascii => Some(target),
            Charset::Utf8 => (target..view.len()).find(|&i| !is_continuation(view[i])),
        }
    }

    fn last_cut(&self, view: &[u8]) -> usize {
        match self.charset {
            Charset::Ascii => view.len(),
            Charset::Utf8 => {
                // keep a truncated character for the next job
                let start = view.len().saturating_sub(4);
                match (start..view.len())
                    .rev()
                    .find(|&i| !is_continuation(view[i]))
                {
                    Some(i) if i + utf8_len(view[i]) > view.len() => i,
                    _ => view.len(),
                }
            }
        }
    }

    fn work(&self, data: &[u8], out: &mut String) -> bool {
        // placeholder for the letter space held back at the end of the previous job, which the
        // encoder does not look back at
        out.push(' ');
        let mut stream = self.stream();
        stream.feed(data, out);
        let pending_space = stream.pending_space();
        stream.finish(out);
        pending_space
    }
}

fn is_continuation(c: u8) -> bool {
    c & 0xc0 == 0x80
}

/// Length of the UTF-8 sequence starting with `c` (1 for invalid bytes, which are skipped).
fn utf8_len(c: u8) -> usize {
    match c {
        0xc0..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf7 => 4,
        _ => 1,
    }
}

impl Codec for Decoder {
    type Info = ();

    fn next_cut(&self, view: &[u8], target: usize) -> Option<usize> {
        let i = view[target - 1..].iter().position(|&c| is_separator(c))?;
        Some(target + i)
    }

    fn last_cut(&self, view: &[u8]) -> usize {
        view.iter()
            .rposition(|&c| is_separator(c))
            .map_or(0, |i| i + 1)
    }

    fn compact(&self, job: &mut [u8]) -> usize {
        // a sequence of more than 7 elements is dropped, whatever its content and length
        if job.len() > 8 {
            job[..8].copy_from_slice(b"xxxxxxxx");
            return 8;
        }
        job.len()
    }

    fn work(&self, data: &[u8], out: &mut String) {
        self.decode_into(data, out);
    }
}

/// Write the output of an encoding job; `pending_space` tells whether the output so far ends
/// with a letter space that was held back.
fn write_encoded(
    output: &mut impl Write,
    out: &str,
    job_pending_space: bool,
    pending_space: &mut bool,
) -> io::Result<()> {
    // skip the placeholder for the held-back letter space, unless it is needed
    let out = out.as_bytes();
    if out.len() > 1 {
        let replaced = matches!(out[1], b'\t' | b'\n' | b'\r');
        let start = if *pending_space && !replaced { 0 } else { 1 };
        output.write_all(&out[start..])?;
        *pending_space = job_pending_space;
    }
    Ok(())
}

impl Encoder {
    fn encode_parallel_impl<R: Read>(
        &self,
        input: Input<'_, R>,
        output: &mut (impl Write + Send),
        threads: NonZeroUsize,
    ) -> io::Result<()> {
        let mut pending_space = false;
        run(input, output, threads, self, move |output, out, info| {
            write_encoded(output, out, info, &mut pending_space)
        })
    }

    /// Encode text into a new [`String`], using up to `threads` threads.
    ///
    /// The result is the same as [`Encoder::encode`]. Small inputs are processed by the calling
    /// thread.
    ///
    /// ```
    /// use ripmors::{Charset, Encoder};
    ///
    /// let threads = std::thread::available_parallelism().unwrap();
    /// let encoder = Encoder::new(Charset::Utf8);
    /// let text = "Morse code\n".repeat(100_000);
    /// assert_eq!(encoder.encode_parallel(text.as_bytes(), threads), encoder.encode(text.as_bytes()));
    /// ```
    pub fn encode_parallel(&self, input: &[u8], threads: NonZeroUsize) -> String {
        let mut output = Vec::new();
        self.encode_parallel_write(input, &mut output, threads)
            .expect("writing to a Vec cannot fail");
        // SAFETY: the encoders only output ASCII
        unsafe { String::from_utf8_unchecked(output) }
    }

    /// Encode text into a [`Write`] object, using up to `threads` threads (the calling thread cuts
    /// the input into jobs, and another thread writes their outputs in order).
    ///
    /// This writes the same bytes as [`Encoder::encode`], without holding the whole output in
    /// memory.
    pub fn encode_parallel_write(
        &self,
        input: &[u8],
        output: &mut (impl Write + Send),
        threads: NonZeroUsize,
    ) -> io::Result<()> {
        self.encode_parallel_impl(Input::Slice::<&[u8]>(input), output, threads)
    }

    /// Encode text from a [`Read`] object into a [`Write`] object, until the end of the input,
    /// using up to `threads` threads (the calling thread reads the input, and another thread
    /// writes the output).
    ///
    /// This writes the same bytes as [`Encoder::encode_stream`]. Each read is processed as soon as
    /// it returns, so the output follows the input as promptly as with
    /// [`Encoder::encode_stream`]; when the reads return large amounts of data (for instance from a
    /// regular file, or a pipe written faster than it is read), they are processed in parallel.
    ///
    /// ```no_run
    /// let threads = std::thread::available_parallelism().unwrap();
    /// let mut stdin = std::io::stdin().lock();
    /// let mut stdout = std::io::stdout();
    /// ripmors::Encoder::default()
    ///     .encode_stream_parallel(&mut stdin, &mut stdout, threads)
    ///     .unwrap();
    /// ```
    pub fn encode_stream_parallel(
        &self,
        input: &mut impl Read,
        output: &mut (impl Write + Send),
        threads: NonZeroUsize,
    ) -> io::Result<()> {
        self.encode_parallel_impl(Input::Reader(input), output, threads)
    }
}

impl Decoder {
    fn decode_parallel_impl<R: Read>(
        &self,
        input: Input<'_, R>,
        output: &mut (impl Write + Send),
        threads: NonZeroUsize,
    ) -> io::Result<()> {
        run(input, output, threads, self, |output, out, ()| {
            output.write_all(out.as_bytes())
        })
    }

    /// Decode Morse code into a new [`String`], using up to `threads` threads.
    ///
    /// The result is the same as [`Decoder::decode`]. Small inputs are processed by the calling
    /// thread.
    ///
    /// ```
    /// use ripmors::Decoder;
    ///
    /// let threads = std::thread::available_parallelism().unwrap();
    /// let decoder = Decoder::default();
    /// let morse = "-- --- .-. ... . / -.-. --- -.. .\n".repeat(100_000);
    /// assert_eq!(decoder.decode_parallel(morse.as_bytes(), threads), decoder.decode(morse.as_bytes()));
    /// ```
    pub fn decode_parallel(&self, input: &[u8], threads: NonZeroUsize) -> String {
        let mut output = Vec::new();
        self.decode_parallel_write(input, &mut output, threads)
            .expect("writing to a Vec cannot fail");
        // SAFETY: the output of each job is a `String`, and jobs are cut between characters
        unsafe { String::from_utf8_unchecked(output) }
    }

    /// Decode Morse code into a [`Write`] object, using up to `threads` threads (the calling
    /// thread cuts the input into jobs, and another thread writes their outputs in order).
    ///
    /// This writes the same bytes as [`Decoder::decode`], without holding the whole output in
    /// memory.
    pub fn decode_parallel_write(
        &self,
        input: &[u8],
        output: &mut (impl Write + Send),
        threads: NonZeroUsize,
    ) -> io::Result<()> {
        self.decode_parallel_impl(Input::Slice::<&[u8]>(input), output, threads)
    }

    /// Decode Morse code from a [`Read`] object into a [`Write`] object, until the end of the
    /// input, using up to `threads` threads (the calling thread reads the input, and another
    /// thread writes the output).
    ///
    /// This writes the same bytes as [`Decoder::decode_stream`]. Each read is processed as soon as
    /// it returns (up to its last separator), so the output follows the input as promptly as with
    /// [`Decoder::decode_stream`]; when the reads return large amounts of data (for instance from
    /// a regular file, or a pipe written faster than it is read), they are processed in parallel.
    ///
    /// ```no_run
    /// let threads = std::thread::available_parallelism().unwrap();
    /// let mut stdin = std::io::stdin().lock();
    /// let mut stdout = std::io::stdout();
    /// ripmors::Decoder::default()
    ///     .decode_stream_parallel(&mut stdin, &mut stdout, threads)
    ///     .unwrap();
    /// ```
    pub fn decode_stream_parallel(
        &self,
        input: &mut impl Read,
        output: &mut (impl Write + Send),
        threads: NonZeroUsize,
    ) -> io::Result<()> {
        self.decode_parallel_impl(Input::Reader(input), output, threads)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Alphabet;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    use std::sync::Arc;

    const THREADS: [usize; 4] = [1, 2, 3, 8];

    /// A reader returning short reads of random sizes, like a pipe.
    struct ShortReads<'a> {
        data: &'a [u8],
        rng: StdRng,
        max: usize,
    }

    impl Read for ShortReads<'_> {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let n = self.rng.gen_range(1..=self.max);
            let n = buf.len().min(self.data.len()).min(n);
            buf[..n].copy_from_slice(&self.data[..n]);
            self.data = &self.data[n..];
            Ok(n)
        }
    }

    fn threads(n: usize) -> NonZeroUsize {
        NonZeroUsize::new(n).unwrap()
    }

    /// Concatenation of random tokens until at least `len` bytes.
    fn random_input(rng: &mut StdRng, tokens: &[&[u8]], len: usize) -> Vec<u8> {
        let mut data = Vec::with_capacity(len + 64);
        while data.len() < len {
            data.extend_from_slice(tokens[rng.gen_range(0..tokens.len())]);
        }
        data
    }

    fn short_reads(data: &[u8], seed: usize, max: usize) -> ShortReads<'_> {
        ShortReads {
            data,
            rng: StdRng::seed_from_u64(seed as u64),
            max,
        }
    }

    fn check_encode(input: &[u8]) {
        for charset in [Charset::Utf8, Charset::Ascii] {
            let encoder = Encoder::new(charset);
            let expected = encoder.encode(input);
            for n in THREADS {
                let output = encoder.encode_parallel(input, threads(n));
                assert!(output == expected, "{charset:?} slice, {n} threads");
                for max in [7, 5000, 100_000] {
                    let mut output = Vec::new();
                    let mut reader = short_reads(input, n, max);
                    encoder
                        .encode_stream_parallel(&mut reader, &mut output, threads(n))
                        .unwrap();
                    assert!(
                        output == expected.as_bytes(),
                        "{charset:?} reader, {n} threads, {max}"
                    );
                }
            }
        }
    }

    fn check_decode(input: &[u8]) {
        let decoders = [
            Decoder::new(Alphabet::Standard),
            Decoder::new(Alphabet::Japanese),
            Decoder::from_fn(crate::to_russian),
        ];
        for decoder in decoders {
            let expected = decoder.decode(input);
            for n in THREADS {
                let output = decoder.decode_parallel(input, threads(n));
                assert!(output == expected, "slice, {n} threads");
                for max in [7, 5000, 100_000] {
                    let mut output = Vec::new();
                    let mut reader = short_reads(input, n, max);
                    decoder
                        .decode_stream_parallel(&mut reader, &mut output, threads(n))
                        .unwrap();
                    assert!(output == expected.as_bytes(), "reader, {n} threads, {max}");
                }
            }
        }
    }

    const TEXT_TOKENS: &[&[u8]] = &[
        b"e",
        b"T",
        b"7",
        b"%",
        b" ",
        b" ",
        b"\n",
        b"\t",
        b"\r",
        b"#",
        b"\x00",
        b"\xff",
        b"\x80",
        b"abc ",
        b"word\n",
        b"\n\n",
        b"#\n",
        " é".as_bytes(),
        "で".as_bytes(),
        "中".as_bytes(),
        "\u{1F600}".as_bytes(),
        "Ж".as_bytes(),
        " \u{a0}\n".as_bytes(),
    ];

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_parallel_encode_random() {
        let mut rng = StdRng::seed_from_u64(0);
        for len in [0, 1, 5, 1000, 5000, 20_000, 100_000] {
            check_encode(&random_input(&mut rng, TEXT_TOKENS, len));
        }
        let input: Vec<u8> = (0..50_000).map(|_| rng.r#gen()).collect();
        check_encode(&input);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_parallel_encode_structured() {
        let mut rng = StdRng::seed_from_u64(1);
        // whitespace right after the cut points, possibly after characters with no encoding
        for token in [&b"ab\n"[..], b"a\n", b"a#\n", b"a\x00\t", b"\n", b"a "] {
            check_encode(&token.repeat(20_000));
        }
        // multi-byte characters around the cut points
        for k in 0..4 {
            let mut input = vec![b'a'; JOB_SIZE - k];
            input.extend_from_slice(&"\u{1F600}é中".repeat(3000).into_bytes());
            check_encode(&input);
        }
        // invalid and truncated UTF-8 around the cut points
        for k in 0..4 {
            let mut input = random_input(&mut rng, &[b"ab ", b"cd\n", b"e"], JOB_SIZE - k);
            input.truncate(JOB_SIZE - k);
            input.extend_from_slice(&"中".as_bytes()[..2]);
            input.extend_from_slice(b"\xbf\xbf\nx");
            input.extend_from_slice(&random_input(&mut rng, TEXT_TOKENS, 3 * JOB_SIZE));
            check_encode(&input);
        }
        check_encode(&"中".repeat(20_000).into_bytes());
    }

    const MORSE_TOKENS: &[&[u8]] = &[
        b".",
        b"-",
        b".",
        b"-",
        b" ",
        b" ",
        b"/",
        b"\n",
        b"\t",
        b"\r",
        b"x",
        b"\x00",
        b".-.. ",
        b"--- / ",
        b"........ ",
        b"---------",
        b"/ ",
    ];

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_parallel_decode_random() {
        let mut rng = StdRng::seed_from_u64(2);
        for len in [0, 1, 7, 8, 100, 1000, 20_000, 100_000] {
            check_decode(&random_input(&mut rng, MORSE_TOKENS, len));
        }
        let input: Vec<u8> = (0..100_000).map(|_| rng.r#gen()).collect();
        check_decode(&input);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_parallel_decode_structured() {
        let mut rng = StdRng::seed_from_u64(3);
        // sequences of various lengths around the cut points
        for run in [1, 6, 7, 8, 9, 100, 3 * JOB_SIZE] {
            for k in 0..3 {
                for &separator in b" /\n" {
                    let mut input = random_input(&mut rng, MORSE_TOKENS, JOB_SIZE - k);
                    input.truncate(JOB_SIZE - k);
                    input.extend(std::iter::repeat_n(b'-', run));
                    input.push(separator);
                    input.extend_from_slice(&random_input(&mut rng, MORSE_TOKENS, 2 * JOB_SIZE));
                    check_decode(&input);
                    // same at the end of the input
                    input.truncate(JOB_SIZE - k + run);
                    check_decode(&input);
                }
            }
        }
        // no separators at all
        check_decode(&vec![b'.'; 5 * JOB_SIZE]);
        check_decode(&b".-".repeat(3));
    }

    /// A reader receiving its input from a channel, blocking until each piece is available.
    struct ChannelReader(mpsc::Receiver<Vec<u8>>, Vec<u8>);

    impl Read for ChannelReader {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if self.1.is_empty() {
                match self.0.recv() {
                    Ok(data) => self.1 = data,
                    Err(_) => return Ok(0),
                }
            }
            let n = buf.len().min(self.1.len());
            buf[..n].copy_from_slice(&self.1[..n]);
            self.1.drain(..n);
            Ok(n)
        }
    }

    /// A writer shared with the test.
    #[derive(Clone)]
    struct SharedWriter(Arc<Mutex<Vec<u8>>>);

    impl Write for SharedWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    /// Check that each piece of input is processed before the next one is available, like with
    /// `tail -f log | ripmors`: `pieces` gives each piece of input, and the output it completes.
    fn check_prompt(
        run: impl FnOnce(&mut ChannelReader, &mut SharedWriter) + Send,
        pieces: &[(&[u8], &[u8])],
    ) {
        let (tx, rx) = mpsc::channel();
        let output = SharedWriter(Arc::new(Mutex::new(Vec::new())));
        let mut writer = output.clone();
        std::thread::scope(|s| {
            s.spawn(move || run(&mut ChannelReader(rx, Vec::new()), &mut writer));
            let mut expected = Vec::new();
            for (input, more_output) in pieces {
                tx.send(input.to_vec()).unwrap();
                expected.extend_from_slice(more_output);
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
                while *output.0.lock().unwrap() != expected {
                    assert!(std::time::Instant::now() < deadline, "{expected:?}");
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }
            drop(tx);
        });
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_parallel_stream_is_prompt() {
        let e_acute = "é".as_bytes();
        for n in [1, 4] {
            let decoder = Decoder::default();
            check_prompt(
                |r, w| decoder.decode_stream_parallel(r, w, threads(n)).unwrap(),
                &[
                    (b"... --- ...\n", b"SOS\n"),
                    (b".- -...", b"A"),
                    (b"\n", b"B\n"),
                ],
            );
            let encoder = Encoder::default();
            check_prompt(
                |r, w| encoder.encode_stream_parallel(r, w, threads(n)).unwrap(),
                &[
                    (b"sos\n", b"... --- ...\n"),
                    (b"ab", b".- -..."),
                    (&e_acute[..1], b""),
                    (&[e_acute[1], b'\n'], b" ..-..\n"),
                ],
            );
        }
    }
}
