# monero-oxide-compat

A wrapper around [monero-oxide](https://github.com/monero-oxide/monero-oxide) library that provides interface similar to [monero-rs](https://github.com/monero-rs/monero-rs) package.

## Current status

- [x] `blockdata::block::Block`: partial implementation
- [ ] `blockdata::block::BlockHeader`
- [ ] `blockdata::transaction`
- [ ] `consensus::encode::VarInt`
- [ ] `consensus::encode::consensus_decode_sized_vec`
- [x] `consensus::encode::deserialize`: partial implementation; can only deserialize into `Block`.
- [ ] `consensus::encode::deserialize_partial`
- [ ] `consensus::encode::serialize`
- [ ] `consensus::encode::serialize_hex`
- [x] `cryptonote::hash::Hash`
- [ ] `cryptonote::hash::Hash8`
- [ ] `cryptonote::hash::keccak_256`
- [ ] `cryptonote::hash::tree_hash`
- [ ] `cryptonote::onetime_key`
- [x] `cryptonote::subaddress::Index`
- [ ] `cryptonote::subaddress::get_public_keys`
- [ ] `cryptonote::subaddress::get_secret_keys`
- [ ] `cryptonote::subaddress::get_secret_scalar`
- [ ] `cryptonote::subaddress::get_spend_public_key`
- [x] `cryptonote::subaddress::get_subaddress`: returns `Result`.
- [ ] `cryptonote::subaddress::get_view_secret_key`
- [x] `network::Network`
- [x] `util::address::Address`
  - `network()`, `addr_type()`, `public_spend()` and `public_view()` are methods, not struct fields.
- [x] `util::address::PaymentId`
- [x] `util::address::AddressType`
  - Replaced with `monero_address::AddressType`:
    - Replaced `Standard` with `Legacy`.
    - Replaced `Integrated(PaymentId)` with `LegacyIntegrated([u8; 32])`.
    - Added `Featured`.
- [x] `util::amount`
- [x] `util::key::KeyPair`
- [x] `util::key::PrivateKey`: does not implement `Copy` anymore
- [x] `util::key::PublicKey`
- [x] `util::key::ViewPair`
- [ ] `util::ringct`
