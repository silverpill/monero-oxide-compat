//! Consensus-encodable types and errors.

use std::io::Cursor;

use monero_oxide::block::Block as OxideBlock;
use thiserror::Error;

use crate::blockdata::block::Block;

/// Errors encountered when encoding or decoding data.
#[derive(Debug, Error)]
pub enum Error {
    /// And I/O error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Deserialize a block from a byte vector, will error if said deserialization doesn't consume
/// the entire vector.
pub fn deserialize(bytes: &[u8]) -> Result<Block, Error> {
    let mut cursor = Cursor::new(bytes);
    let oxide_block = OxideBlock::read(&mut cursor)?;
    Ok(Block(oxide_block))
}
