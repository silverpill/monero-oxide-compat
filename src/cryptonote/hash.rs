//! Hash functions and types.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

const HASH_SIZE: usize = 32;

/// Result of the Keccak-256 hashing function.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Hash(pub [u8; HASH_SIZE]);

impl Hash {
    pub const fn repeat_byte(byte: u8) -> Self {
        Self([byte; HASH_SIZE])
    }

    pub const fn zero() -> Self {
        Self::repeat_byte(0u8)
    }

    fn from_low_u64_with_fn(val: u64, to_bytes: fn(u64) -> [u8; 8]) -> Self {
        // Copied from monero-rs & fixed-hash
        let buf = to_bytes(val);
        let capped = std::cmp::min(HASH_SIZE, 8);
        let mut bytes = [0x0; std::mem::size_of::<Self>()];
        bytes[(HASH_SIZE - capped)..].copy_from_slice(&buf[..capped]);
        let mut ret = Self::zero();
        ret.0.copy_from_slice(&bytes);
        ret
    }

    pub fn from_low_u64_be(val: u64) -> Self {
        Self::from_low_u64_with_fn(val, u64::to_be_bytes)
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsMut<[u8]> for Hash {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl FromStr for Hash {
    type Err = rustc_hex::FromHexError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input = input.strip_prefix("0x").unwrap_or(input);
        let mut iter = rustc_hex::FromHexIter::new(input);
        let mut result = Self::zero();
        for byte in result.as_mut() {
            *byte = iter.next().ok_or(Self::Err::InvalidHexLength)??;
        }
        if iter.next().is_some() {
            return Err(Self::Err::InvalidHexLength);
        }
        Ok(result)
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x")?;
        for i in &self.0[0..2] {
            write!(f, "{:02x}", i)?;
        }
        write!(f, "…")?;
        for i in &self.0[HASH_SIZE - 2..HASH_SIZE] {
            write!(f, "{:02x}", i)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use monero::Hash as LegacyHash;

    #[test]
    fn test_hash_ser() {
        let hex = "2d0ad2566627b50cd45125e89e963433b212b368cd2d91662c44813ba9ec90c2";
        let hash = Hash::from_str(hex).unwrap();
        assert!(hash.to_string().starts_with("0x2d0a"));
    }

    #[test]
    fn test_from_low_u64_be() {
        let input = 123;
        let hash = Hash::from_low_u64_be(input);
        let legacy_hash = LegacyHash::from_low_u64_be(input);
        assert_eq!(hash.0, legacy_hash.0);
    }
}
