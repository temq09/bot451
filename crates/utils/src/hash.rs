use std::io::{self, Read};

use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use digest_io::IoWrapper;
use sha2::{Digest, Sha256};

pub fn make_hash_for_file(path: &str) -> Option<String> {
    std::fs::File::open(path)
        .ok()
        .as_mut()
        .and_then(|file| get_hash(file).ok())
}

fn get_hash<R>(source: &mut R) -> anyhow::Result<String>
where
    R: Read,
{
    let mut sha256 = IoWrapper(Sha256::new());
    io::copy(source, &mut sha256)?;
    let hash = sha256.0.finalize();
    Ok(BASE64_STANDARD.encode(hash))
}

#[cfg(test)]
mod test {
    use crate::hash::get_hash;

    #[test]
    fn test_get_hash() -> anyhow::Result<()> {
        let mut data: &[u8] = b"test hash";

        let hash = get_hash(&mut data)?;

        assert_eq!(
            hash,
            "VKZIO4rKVcnfKjW69x2ZZd39YjRo2B1RIpvV630eHBs=".to_string()
        );

        Ok(())
    }
}
