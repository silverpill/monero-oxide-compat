//! Block and block header structures.

use monero_oxide::block::Block as OxideBlock;

/// A full block
#[allow(dead_code)]
pub struct Block(pub(crate) OxideBlock);

impl Block {
    /// Returns `monero_oxide::block::Block`
    pub fn as_oxide(&self) -> &OxideBlock {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use crate::consensus::encode::deserialize;

    #[test]
    fn test_block_ser() {
        // sample from monero-rs test
        let hex = "0c0c94debaf805beb3489c722a285c092a32e7c6893abfc7d069699c8326fc3445a749c5276b6200000000029b892201ffdf882201b699d4c8b1ec020223df524af2a2ef5f870adb6e1ceb03a475c39f8b9ef76aa50b46ddd2a18349402b012839bfa19b7524ec7488917714c216ca254b38ed0424ca65ae828a7c006aeaf10208f5316a7f6b99cca60000";
        let bytes = hex::decode(hex).unwrap();
        let block = deserialize(&bytes).unwrap();
        assert_eq!(block.0.serialize(), bytes);
    }
}
