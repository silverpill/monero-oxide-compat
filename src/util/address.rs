//! Monero addresses types and helper functions.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use monero_wallet::address::{AddressType, MoneroAddress as OxideAddress, Network};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as DeserializerError};

use crate::util::key::{KeyPair, scalar_to_point};

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
    /// The network on which the address is valid and should be used.
    pub fn network(&self) -> Network {
        self.0.network()
    }

    /// Create a standard address from a view pair which is valid on the given network.
    pub fn from_keypair(network: Network, keys: &KeyPair) -> Self {
        let public_spend = scalar_to_point(keys.spend.0);
        let public_view = scalar_to_point(keys.view.0);
        let oxide_address = OxideAddress::new(
            network,
            AddressType::Legacy, // Standard
            public_spend,
            public_view,
        );
        Self(oxide_address)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
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
}
