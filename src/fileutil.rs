use std::fs::File;
use std::path::PathBuf;
use std::{io, mem};

pub fn get_extension(filename: &str) -> &str {
    let mut split = filename.split('.');
    split.next_back().unwrap_or("")
}

pub fn create_temp_file() -> io::Result<(PathBuf, File)> {
    let dir = tempfile::tempdir()?;
    let filename = dir.path().join("graph.dot");
    let file = File::create(&filename)?;

    // file is leaked, but that's ok
    mem::forget(dir);
    Ok((filename, file))
}
