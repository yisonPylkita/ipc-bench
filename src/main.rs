use clap::Parser;
use nix::sys::stat;
use nix::unistd;
use tempfile::tempdir;

/// Benchmark various Unix IPC mechanisms
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Message size in kilobytes
    #[arg(short, long, default_value_t = 4)]
    size_in_kilobytes: usize,
    // TODO: allow selecting IPC mechanism to test
}

fn test_named_pipe() {
    let tmp_dir = tempdir().unwrap();
    let fifo_path = tmp_dir.path().join("foo.pipe");

    match unistd::mkfifo(&fifo_path, stat::Mode::S_IRWXU) {
        Ok(_) => println!("created {:?}", fifo_path),
        Err(err) => println!("Error creating fifo: {}", err),
    }

    match fork() {
        Ok(ForkResult::Parent { child, .. }) => {
            let mut fifo_writer = std::fs::OpenOptions::new().write(true).open(FIFO_PATH)?;
            fifo_writer.write_all(b"Hello, FIFO!")?;
            child.wait()?;
        }
        Ok(ForkResult::Child) => {
            let fifo_fd = unsafe {
                libc::open(
                    CString::new(FIFO_PATH).unwrap().as_ptr(),
                    libc::O_RDONLY | libc::O_NONBLOCK,
                )
            };
            let mut fifo_reader = unsafe { std::fs::File::from_raw_fd(fifo_fd) };
            let mut buffer = [0u8; 1024];
            loop {
                match fifo_reader.read(&mut buffer) {
                    Ok(0) => break, // EOF reached
                    Ok(n) => {
                        let data = String::from_utf8_lossy(&buffer[..n]);
                        println!("Received: {}", data);
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(e) => {
                        eprintln!("Error reading from FIFO: {}", e);
                        break;
                    }
                }
            }
        }
        Err(_) => eprintln!("Fork failed"),
    }
}

fn main() {
    test_named_pipe();
}
