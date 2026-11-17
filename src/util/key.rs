//! Monero public and private keys.

use std::fmt;
use std::io::Cursor;
use std::str::FromStr;

use curve25519_dalek::constants::ED25519_BASEPOINT_TABLE;
use monero_wallet::{
    ViewPair as OxideViewPair, ViewPairError,
    ed25519::{CompressedPoint, Point, Scalar},
};
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

/// Potential errors encountered during key decoding.
#[derive(Error, Debug)]
pub enum Error {
    /// Invalid input length.
    #[error("Invalid input")]
    InvalidInput(#[from] std::io::Error),
    /// Invalid point on the curve.
    #[error("Invalid point on the curve")]
    InvalidPoint,
    #[error("Invalid key length")]
    InvalidLength,
    /// Hex parsing error.
    #[error("Hex error: {0}")]
    Hex(#[from] hex::FromHexError),
}

// Scalar: private key
// Point: public key
fn scalar_to_point(scalar: Scalar) -> Point {
    Point::from(&scalar.into() * ED25519_BASEPOINT_TABLE)
}

/// A private key, a valid curve25519 scalar.
#[derive(Clone, Debug, PartialEq, Zeroize, ZeroizeOnDrop)]
pub struct PrivateKey(pub(crate) Scalar);

impl PrivateKey {
    pub fn from_slice(data: &[u8]) -> Result<Self, Error> {
        let mut cursor = Cursor::new(data);
        let scalar = Scalar::read(&mut cursor)?;
        if cursor.position() as usize != data.len() {
            return Err(Error::InvalidLength);
        };
        Ok(Self(scalar))
    }
}

impl FromStr for PrivateKey {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s)?;
        Self::from_slice(&bytes[..])
    }
}

impl fmt::Display for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let c_scalar: curve25519_dalek::Scalar = self.0.into();
        write!(f, "{}", hex::encode(c_scalar.to_bytes()))
    }
}

/// A public key, a valid edward point on the curve.
#[derive(Clone, Copy)]
pub struct PublicKey(pub(crate) Point);

impl PublicKey {
    fn to_bytes(self) -> [u8; 32] {
        self.0.into().compress().to_bytes()
    }

    /// Deserialize a public key from a slice.
    pub fn from_slice(data: &[u8]) -> Result<PublicKey, Error> {
        let mut cursor = Cursor::new(data);
        let compressed_point = CompressedPoint::read(&mut cursor)?;
        if cursor.position() as usize != data.len() {
            return Err(Error::InvalidLength);
        };
        let point = compressed_point.decompress().ok_or(Error::InvalidPoint)?;
        Ok(PublicKey(point))
    }

    /// Generate a public key from the private key.
    pub fn from_private_key(privkey: &PrivateKey) -> Self {
        Self(scalar_to_point(privkey.0))
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", hex::encode(self.to_bytes()))
    }
}

impl FromStr for PublicKey {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s)?;
        Self::from_slice(&bytes)
    }
}

/// Two private keys representing the view and the spend keys.
#[derive(Clone)]
pub struct KeyPair {
    pub view: PrivateKey,
    pub spend: PrivateKey,
}

/// View pair to scan transaction outputs and retrieve amounts, but can't spend outputs.
#[derive(Clone)]
pub struct ViewPair {
    pub view: PrivateKey,
    pub spend: PublicKey,
}

impl ViewPair {
    pub(crate) fn to_oxide(&self) -> Result<OxideViewPair, ViewPairError> {
        OxideViewPair::new(self.spend.0, Zeroizing::new(self.view.0))
    }
}

impl From<&KeyPair> for ViewPair {
    fn from(keys: &KeyPair) -> Self {
        let spend = scalar_to_point(keys.spend.0);
        Self {
            view: keys.view.clone(),
            spend: PublicKey(spend),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Copied from monero-rs
    #[test]
    fn public_key_from_secret() {
        let privkey = PrivateKey::from_str(
            "77916d0cd56ed1920aef6ca56d8a41bac915b68e4c46a589e0956e27a7b77404",
        )
        .unwrap();
        assert_eq!(
            "eac2cc96e0ae684388e3185d5277e51313bff98b9ad4a12dcd9205f20d37f1a3",
            PublicKey::from_private_key(&privkey).to_string()
        );
    }

    // Copied from monero-rs
    #[test]
    fn parse_public_key() {
        assert!(
            PublicKey::from_str("eac2cc96e0ae684388e3185d5277e51313bff98b9ad4a12dcd9205f20d37f1a3")
                .is_ok()
        );
    }

    #[test]
    fn private_key_length_validation() {
        let key_hex = concat!(
            "77916d0cd56ed1920aef6ca56d8a41bac915b68e4c46a589e0956e27a7b77404",
            "23"
        );
        let result = PrivateKey::from_str(key_hex);
        assert!(result.is_err());
        // monero-oxide
        let mut data = Cursor::new([0; 33]);
        let result = Scalar::read(&mut data);
        assert!(result.is_ok());
    }

    #[test]
    fn public_key_length_validation() {
        let key_hex = concat!(
            "eac2cc96e0ae684388e3185d5277e51313bff98b9ad4a12dcd9205f20d37f1a3",
            "23"
        );
        let result = PublicKey::from_str(key_hex);
        assert!(result.is_err());
        // monero-oxide
        let mut cursor = Cursor::new([0; 33]);
        let result = CompressedPoint::read(&mut cursor);
        assert!(result.is_ok());
        let mut cursor = Cursor::new([0; 31]);
        let result = CompressedPoint::read(&mut cursor);
        assert!(result.is_err());
    }
}
