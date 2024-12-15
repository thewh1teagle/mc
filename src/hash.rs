use blake2::{Blake2s256, Digest};
use eyre::{bail, Result};
use std::path::Path;

pub fn verify_hash(source_path: &Path, destination_path: &Path) -> Result<()> {
    let mut source_hash = Blake2s256::new();
    let mut dst_hash = Blake2s256::new();

    if source_path.is_dir() {
        file_hashing::get_hash_folder(source_path, &mut source_hash, num_cpus::get(), |_| {})?;
        file_hashing::get_hash_folder(destination_path, &mut dst_hash, num_cpus::get(), |_| {})?;
    } else {
        file_hashing::get_hash_file(source_path, &mut source_hash)?;
        file_hashing::get_hash_file(destination_path, &mut dst_hash)?;
    }

    let source_hash = source_hash.finalize();
    let dst_hash = dst_hash.finalize();
    let source_hash = hex::encode(source_hash);
    let dst_hash = hex::encode(dst_hash);

    tracing::info!("Source hash: {}", source_hash);
    tracing::info!("Destination hash: {}", dst_hash);

    if source_hash == dst_hash {
        Ok(())
    } else {
        bail!("Hashes do not match")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_verify_hash_file_identical() {
        // Create temporary files for source and destination
        let temp_dir = TempDir::new().unwrap();
        let mut source_file = NamedTempFile::new_in(temp_dir.path()).unwrap();
        let mut destination_file = NamedTempFile::new_in(temp_dir.path()).unwrap();

        // Write identical content to both files
        let content = b"Hello, world!";
        source_file.write_all(content).unwrap();
        destination_file.write_all(content).unwrap();

        // Ensure the files are correctly created
        let source_path = source_file.path();
        let destination_path = destination_file.path();

        // Call verify_hash to check that the hashes are identical
        let result = verify_hash(source_path, destination_path);
        assert!(
            result.is_ok(),
            "Hashes should be identical for the same content."
        );
    }

    #[test]
    fn test_verify_hash_file_different() {
        // Create temporary files for source and destination
        let temp_dir = TempDir::new().unwrap();
        let mut source_file = NamedTempFile::new_in(temp_dir.path()).unwrap();
        let mut destination_file = NamedTempFile::new_in(temp_dir.path()).unwrap();

        // Write different content to both files
        let content_1 = b"Hello, world!";
        let content_2 = b"Goodbye, world!";
        source_file.write_all(content_1).unwrap();
        destination_file.write_all(content_2).unwrap();

        // Ensure the files are correctly created
        let source_path = source_file.path();
        let destination_path = destination_file.path();

        // Call verify_hash to check that the hashes are different
        let result = verify_hash(source_path, destination_path);
        assert!(
            result.is_err(),
            "Hashes should be different for different content."
        );
    }
}
