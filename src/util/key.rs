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
use zeroize::Zeroizing;

/// Potential errors encountered during key decoding.
#[derive(Error, Debug)]
pub enum Error {
    /// Invalid input length.
    #[error("Invalid input")]
    InvalidInput(#[from] std::io::Error),
    /// Invalid point on the curve.
    #[error("Invalid point on the curve")]
    InvalidPoint,
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrivateKey(pub(crate) Scalar);

impl PrivateKey {
    pub fn from_slice(data: &[u8]) -> Result<Self, Error> {
        let mut data = Cursor::new(data.to_vec());
        let scalar = Scalar::read(&mut data)?;
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
        let mut data = Cursor::new(data.to_vec());
        let compressed_point = CompressedPoint::read(&mut data)?;
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
#[derive(Clone, Copy)]
pub struct KeyPair {
    pub view: PrivateKey,
    pub spend: PrivateKey,
}

/// View pair to scan transaction outputs and retrieve amounts, but can't spend outputs.
#[derive(Clone, Copy)]
pub struct ViewPair {
    pub view: PrivateKey,
    pub spend: PublicKey,
}

impl ViewPair {
    pub(crate) fn to_oxide(self) -> Result<OxideViewPair, ViewPairError> {
        OxideViewPair::new(self.spend.0, Zeroizing::new(self.view.0))
    }
}

impl From<&KeyPair> for ViewPair {
    fn from(keys: &KeyPair) -> Self {
        let spend = scalar_to_point(keys.spend.0);
        Self {
            view: keys.view,
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
}
