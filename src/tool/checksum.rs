use std::path::Path;
use std::fs::File;
use std::io;
use sha2::{Sha256, Digest};
use md5::Md5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChecksumMethod {
    None,
    Sha256,
    Md5,
}

pub fn verify(method: ChecksumMethod, file_path: &Path, expected: &str) -> io::Result<bool> {
    let result = match method {
        ChecksumMethod::None => true,
        ChecksumMethod::Sha256 => expected.trim() == checksum_sha256(file_path)?,
        ChecksumMethod::Md5 => expected.trim() == checksum_md5(file_path)?,
    };

    Ok(result)
}

fn checksum_md5(file_path: &Path) -> io::Result<String> {
    let mut file = File::open(file_path)?;
    let mut hasher = Md5::new();
    io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

fn checksum_sha256(file_path: &Path) -> io::Result<String> {
    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}
