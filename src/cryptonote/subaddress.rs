//! Sub-address index structure and key calculation helper functions.

use monero_wallet::{
    ViewPairError,
    address::{Network, SubaddressIndex as OxideSubaddressIndex},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::util::{address::Address, key::ViewPair};

#[derive(Debug, Error)]
pub enum SubaddressError {
    #[error("Invalid view pair")]
    InvalidViewPair(#[from] ViewPairError),
    #[error("Invalid index")]
    InvalidIndex,
}

/// A sub-address index with `major` and `minor` indexes, primary address is `0/0`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Index {
    pub major: u32,
    pub minor: u32,
}

/// Compute the sub-address at index `i` valid on the given network.
pub fn get_subaddress(
    keys: &ViewPair,
    index: Index,
    network: Option<Network>,
) -> Result<Address, SubaddressError> {
    let net = network.unwrap_or(Network::Mainnet);
    let oxide_address =
        if let Some(oxide_index) = OxideSubaddressIndex::new(index.major, index.minor) {
            keys.to_oxide()?.subaddress(net, oxide_index)
        } else {
            return Err(SubaddressError::InvalidIndex);
        };
    Ok(Address(oxide_address))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::key::{PrivateKey, PublicKey};
    use std::str::FromStr;

    // Copied from monero-rs
    #[test]
    fn get_subaddress_test() {
        let a = PrivateKey::from_str(
            "77916d0cd56ed1920aef6ca56d8a41bac915b68e4c46a589e0956e27a7b77404",
        )
        .unwrap();
        let b = PrivateKey::from_str(
            "8163466f1883598e6dd14027b8da727057165da91485834314f5500a65846f09",
        )
        .unwrap();
        let viewpair = ViewPair {
            view: a,
            spend: PublicKey::from_private_key(&b),
        };

        let index = Index {
            major: 2,
            minor: 18,
        };
        let address = get_subaddress(&viewpair, index, Some(Network::Mainnet)).unwrap();

        assert_eq!(
            "89pMNxzcCo5LAPZDX4qaTeanA6ZiS3VRdUbeKHzbDZkD1Q3YsDDfmXbT2zyjLeHWuuN4vxKne8kNpjH3cMk7nmhwSALCxsd",
            address.to_string()
        );
    }
}
