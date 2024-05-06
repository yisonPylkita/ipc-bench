use memmap::MmapMut;
use nix::libc;
use nix::{
    sys::wait::waitpid,
    unistd::{fork, ForkResult},
};
use os_pipe::pipe;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::Instant;

const BUFFER_SIZE: usize = 4096 * 1024 * 1024;

fn main() {
    println!("Benchmarking unnamed pipes");
    unnamed_pipe();

    println!("");

    println!("Benchmarking shared memory");
    shared_memory();
}

fn unnamed_pipe() {
    let (mut reader, mut writer) = pipe().unwrap();

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            let start = Instant::now();
            let data = vec![0u8; BUFFER_SIZE];
            writer.write_all(&data).unwrap();
            let end = start.elapsed();
            println!("Parrent -> Child took {}ms", end.as_millis());

            waitpid(child, None).unwrap();
        }
        Ok(ForkResult::Child) => {
            let start = Instant::now();
            let mut data = vec![0u8; BUFFER_SIZE];
            reader.read_exact(&mut data).unwrap();
            let end = start.elapsed();
            println!("Child -> Parrent took {}ms", end.as_millis());

            unsafe { libc::_exit(0) };
        }
        Err(_) => println!("Fork failed"),
    }
}

fn shared_memory() {
    let path: PathBuf = "mapfile".into();
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&path)
        .unwrap();
    file.set_len(BUFFER_SIZE as u64).unwrap();

    let mut mmap = unsafe { MmapMut::map_mut(&file).unwrap() };

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            let start = Instant::now();
            let data = vec![0u8; BUFFER_SIZE];
            mmap.copy_from_slice(&data);
            let end = start.elapsed();
            println!("Parrent -> Child took {}ms", end.as_millis());

            waitpid(child, None).unwrap();
        }
        Ok(ForkResult::Child) => {
            let start = Instant::now();
            let mut data = vec![0u8; BUFFER_SIZE];
            data.copy_from_slice(&mmap);
            let end = start.elapsed();
            println!("Child -> Parrent took {}ms", end.as_millis());

            unsafe { libc::_exit(0) };
        }
        Err(_) => println!("Fork failed"),
    }
}
