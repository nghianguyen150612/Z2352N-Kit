use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct ImageHash {
    pub filename: String,
    pub bytes: u64,
    pub sha256: String,
}

pub fn hash_file(path: &Path) -> Result<ImageHash, String> {
    let file = File::open(path).map_err(|e| format!("cannot open image: {e}"))?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("expected a regular file".into());
    }
    let mut reader = BufReader::new(file);
    let mut digest = Sha256::new();
    let mut bytes = 0u64;
    let mut block = [0u8; 128 * 1024];

    loop {
        let count = reader
            .read(&mut block)
            .map_err(|e| format!("read error: {e}"))?;
        if count == 0 {
            break;
        }
        bytes = bytes
            .checked_add(count as u64)
            .ok_or("file size overflow")?;
        digest.update(&block[..count]);
    }

    Ok(ImageHash {
        filename: path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "<unknown>".into()),
        bytes,
        sha256: format!("{:x}", digest.finalize()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonexistent_path_rejected() {
        assert!(hash_file(Path::new("/this-file-does-not-exist-z2352n")).is_err());
    }
}
