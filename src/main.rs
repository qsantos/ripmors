use std::fs::File;
use std::io;
use std::mem::ManuallyDrop;
use std::num::NonZeroUsize;
use std::os::fd::FromRawFd;

use ripmors::{Alphabet, Charset, Decoder, Encoder};

use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Debug, Default, Clone, Copy, Eq, Parser, PartialEq)]
#[clap(rename_all = "kebab_case")]
enum EncodeVariant {
    #[default]
    Unicode,
    Ascii,
}

#[derive(ValueEnum, Debug, Default, Clone, Copy, Eq, Parser, PartialEq)]
#[clap(rename_all = "kebab_case")]
enum DecodeVariant {
    #[default]
    Standard,
    Greek,
    Russian,
    Japanese,
    Korean,
    Hebrew,
    Arabic,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, num_args = 0..=1, default_missing_value = "standard")]
    decode: Option<DecodeVariant>,
    #[arg(short, long, num_args = 0..=1, default_missing_value = "unicode")]
    encode: Option<EncodeVariant>,
    /// Number of threads [default: number of CPUs available]
    #[arg(short = 'j', long)]
    threads: Option<NonZeroUsize>,
}

/// A read-only memory map of a whole file.
#[cfg(feature = "fast-io")]
struct Mmap {
    ptr: *mut libc::c_void,
    len: usize,
}

#[cfg(feature = "fast-io")]
impl Mmap {
    /// Map `file` if it is a non-empty regular file.
    fn new(file: &File) -> Option<Mmap> {
        use std::os::fd::AsRawFd;
        let metadata = file.metadata().ok()?;
        let len = usize::try_from(metadata.len()).ok()?;
        if !metadata.is_file() || len == 0 {
            return None;
        }
        // NOTE: no MAP_POPULATE: populating the mapping is single-threaded, while page faults
        // are spread between the worker threads
        // SAFETY: we map a valid file descriptor, and check the result; the mapping is private and
        // read-only, so it cannot alias any Rust reference
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                len,
                libc::PROT_READ,
                libc::MAP_PRIVATE,
                file.as_raw_fd(),
                0,
            )
        };
        if ptr == libc::MAP_FAILED {
            return None;
        }
        Some(Mmap { ptr, len })
    }

    fn as_slice(&self) -> &[u8] {
        // SAFETY: the mapping is valid for reads of `len` bytes until it is dropped; the file
        // could be modified by another process meanwhile, which would only change the output
        // (or raise SIGBUS if it is truncated), as with any tool reading memory-mapped files
        unsafe { std::slice::from_raw_parts(self.ptr.cast(), self.len) }
    }
}

#[cfg(feature = "fast-io")]
impl Drop for Mmap {
    fn drop(&mut self) {
        // SAFETY: `ptr` and `len` describe a mapping created by `mmap`
        unsafe { libc::munmap(self.ptr, self.len) };
    }
}

fn run(args: &Args) -> io::Result<()> {
    let threads = args
        .threads
        .or_else(|| std::thread::available_parallelism().ok())
        .unwrap_or(NonZeroUsize::MIN);

    // Access the standard streams directly, without the locking and buffering of `Stdin` and
    // `Stdout` (which is line-buffered: it looks for line feeds in the output, and copies the
    // last line into its buffer); `ManuallyDrop` avoids closing them.
    // SAFETY: the file descriptors 0 and 1 are not used anywhere else in this program; if they
    // are not open, reads and writes return errors
    let mut stdin = ManuallyDrop::new(unsafe { File::from_raw_fd(0) });
    // SAFETY: see above
    let mut stdout = ManuallyDrop::new(unsafe { File::from_raw_fd(1) });
    let stdin: &mut File = &mut stdin;
    let stdout: &mut File = &mut stdout;

    // When the input is a regular file, process it in place rather than copying it.
    #[cfg(feature = "fast-io")]
    let mmap = Mmap::new(stdin);
    #[cfg(feature = "fast-io")]
    let slice = match &mmap {
        Some(mmap) => {
            // start from the current offset, like `read` would
            let offset = io::Seek::stream_position(stdin)?;
            Some(mmap.as_slice().get(offset as usize..).unwrap_or_default())
        }
        None => None,
    };
    #[cfg(not(feature = "fast-io"))]
    let slice: Option<&[u8]> = None;

    // When the input or output is a pipe, enlarge it to reduce the number of context switches
    // between the processes (this fails harmlessly for anything else than a pipe, or above the
    // limit of /proc/sys/fs/pipe-max-size).
    #[cfg(feature = "fast-io")]
    for fd in [0, 1] {
        // SAFETY: `fcntl` with `F_SETPIPE_SZ` has no effect on memory
        unsafe { libc::fcntl(fd, libc::F_SETPIPE_SZ, 1 << 20) };
    }

    let result = if let Some(variant) = args.decode {
        let alphabet = match variant {
            DecodeVariant::Standard => Alphabet::Standard,
            DecodeVariant::Greek => Alphabet::Greek,
            DecodeVariant::Russian => Alphabet::Russian,
            DecodeVariant::Japanese => Alphabet::Japanese,
            DecodeVariant::Korean => Alphabet::Korean,
            DecodeVariant::Hebrew => Alphabet::Hebrew,
            DecodeVariant::Arabic => Alphabet::Arabic,
        };
        let decoder = Decoder::new(alphabet);
        match slice {
            Some(input) => decoder.decode_parallel_write(input, stdout, threads),
            None => decoder.decode_stream_parallel(stdin, stdout, threads),
        }
    } else {
        let charset = match args.encode.unwrap_or_default() {
            EncodeVariant::Unicode => Charset::Utf8,
            EncodeVariant::Ascii => Charset::Ascii,
        };
        let encoder = Encoder::new(charset);
        match slice {
            Some(input) => encoder.encode_parallel_write(input, stdout, threads),
            None => encoder.encode_stream_parallel(stdin, stdout, threads),
        }
    };
    if slice.is_some() {
        // leave the offset at the end of the input, like `read` would
        io::Seek::seek(stdin, io::SeekFrom::End(0))?;
    }
    result
}

fn main() {
    let args = Args::parse();
    match run(&args) {
        Ok(()) => (),
        // the reader went away (e.g. `ripmors | head`): this is not an error
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => (),
        Err(e) => {
            eprintln!("ripmors: {e}");
            std::process::exit(1);
        }
    }
}
