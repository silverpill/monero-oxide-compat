//! Monero addresses types and helper functions.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use monero_wallet::address::{MoneroAddress as OxideAddress, Network};
use rand::{Rng, rngs::OsRng};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as DeserializerError};

use crate::util::key::{KeyPair, PublicKey};

pub use monero_wallet::address::{AddressError as Error, AddressType};

const PAYMENT_ID_SIZE: usize = 8;

/// Short Payment Id for integrated address, a fixed 8-bytes array.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentId(pub [u8; PAYMENT_ID_SIZE]);

impl PaymentId {
    pub const fn repeat_byte(byte: u8) -> Self {
        Self([byte; PAYMENT_ID_SIZE])
    }

    pub const fn zero() -> Self {
        Self::repeat_byte(0u8)
    }

    /// Create a new hash with cryptographically random content.
    pub fn random() -> Self {
        // Adapted from fixed-hash crate
        let mut rng = OsRng;
        let mut payment_id = Self::zero();
        for byte in payment_id.0.iter_mut() {
            *byte = rng.r#gen();
        }
        payment_id
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for PaymentId {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsMut<[u8]> for PaymentId {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl FromStr for PaymentId {
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

impl fmt::Display for PaymentId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x")?;
        for i in &self.0[0..2] {
            write!(f, "{:02x}", i)?;
        }
        write!(f, "…")?;
        for i in &self.0[PAYMENT_ID_SIZE - 2..PAYMENT_ID_SIZE] {
            write!(f, "{:02x}", i)?;
        }
        Ok(())
    }
}

/// A complete Monero typed address valid for a specific network.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Address(pub(crate) OxideAddress);

impl Address {
    /// Create a standard address which is valid on the given network.
    pub fn standard(network: Network, public_spend: PublicKey, public_view: PublicKey) -> Self {
        let oxide_address = OxideAddress::new(
            network,
            AddressType::Legacy, // Standard
            public_spend.0,
            public_view.0,
        );
        Self(oxide_address)
    }

    /// Create an address with an integrated payment id which is valid on the given network.
    pub fn integrated(
        network: Network,
        public_spend: PublicKey,
        public_view: PublicKey,
        payment_id: PaymentId,
    ) -> Address {
        let oxide_address = OxideAddress::new(
            network,
            AddressType::LegacyIntegrated(payment_id.0),
            public_spend.0,
            public_view.0,
        );
        Self(oxide_address)
    }

    /// Create a standard address from a view pair which is valid on the given network.
    pub fn from_keypair(network: Network, keys: &KeyPair) -> Self {
        let public_spend = PublicKey::from_private_key(&keys.spend);
        let public_view = PublicKey::from_private_key(&keys.view);
        Self::standard(network, public_spend, public_view)
    }

    /// The network on which the address is valid and should be used.
    pub fn network(&self) -> Network {
        self.0.network()
    }

    pub fn addr_type(&self) -> &AddressType {
        self.0.kind()
    }

    pub fn public_spend(&self) -> PublicKey {
        PublicKey(self.0.spend())
    }

    pub fn public_view(&self) -> PublicKey {
        PublicKey(self.0.view())
    }
}

impl fmt::Display for Address {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl FromStr for Address {
    type Err = Error;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let oxide_address = OxideAddress::from_str_with_unchecked_network(value)?;
        Ok(Self(oxide_address))
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let address_str: String = Deserialize::deserialize(deserializer)?;
        let address = OxideAddress::from_str_with_unchecked_network(&address_str)
            .map_err(DeserializerError::custom)?;
        Ok(Self(address))
    }
}

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let address_str = self.0.to_string();
        serializer.serialize_str(&address_str)
    }
}

// TODO: https://github.com/monero-oxide/monero-oxide/issues/158
impl Hash for Address {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_string().hash(state);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_payment_id_ser() {
        let hex = "2d0ad2566627b50c";
        let hash = PaymentId::from_str(hex).unwrap();
        assert!(hash.to_string().starts_with("0x2d0a"));
    }

    // Copied from monero-rs
    #[test]
    fn deserialize_address() {
        let pub_spend = PublicKey::from_slice(&[
            226, 187, 17, 117, 6, 188, 105, 177, 58, 207, 205, 42, 205, 229, 251, 129, 118, 253,
            21, 245, 49, 67, 36, 75, 62, 12, 80, 90, 244, 194, 108, 210,
        ])
        .unwrap();
        let pub_view = PublicKey::from_slice(&[
            220, 115, 195, 55, 189, 88, 136, 78, 63, 32, 41, 33, 168, 205, 245, 3, 139, 234, 109,
            64, 198, 179, 53, 108, 247, 77, 183, 25, 172, 59, 113, 115,
        ])
        .unwrap();

        let address = "4ADT1BtbxqEWeMKp9GgPr2NeyJXXtNxvoDawpyA4WpzFcGcoHUvXeijE66DNfohE9r1bQYaBiQjEtKE7CtkTdLwiDznFzra";
        let add = Address::from_str(address);
        assert_eq!(
            Ok(Address::standard(Network::Mainnet, pub_spend, pub_view)),
            add
        );
    }

    // Copied from monero-rs
    #[test]
    fn deserialize_integrated_address() {
        let pub_spend = PublicKey::from_slice(&[
            17, 81, 127, 230, 166, 35, 81, 36, 161, 94, 154, 206, 60, 98, 195, 62, 12, 11, 234,
            133, 228, 196, 77, 3, 68, 188, 84, 78, 94, 109, 238, 44,
        ])
        .unwrap();
        let pub_view = PublicKey::from_slice(&[
            115, 212, 211, 204, 198, 30, 73, 70, 235, 52, 160, 200, 39, 215, 134, 239, 249, 129,
            47, 156, 14, 116, 18, 191, 112, 207, 139, 208, 54, 59, 92, 115,
        ])
        .unwrap();
        let payment_id = PaymentId([88, 118, 184, 183, 41, 150, 255, 151]);

        let address = "4Byr22j9M2878Mtyb3fEPcBNwBZf5EXqn1Yi6VzR46618SFBrYysab2Cs1474CVDbsh94AJq7vuV3Z2DRq4zLcY3LHzo1Nbv3d8J6VhvCV";
        let add = Address::from_str(address);
        assert_eq!(
            Ok(Address::integrated(
                Network::Mainnet,
                pub_spend,
                pub_view,
                payment_id
            )),
            add
        );
    }
}
