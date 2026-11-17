# monero-oxide-compat

A wrapper around [monero-oxide](https://github.com/monero-oxide/monero-oxide) library that provides interface similar to [monero-rs](https://github.com/monero-rs/monero-rs) package.

Differences in API compared to `monero-rs`:

- `get_subaddress()` returns `Result`.
- `Address::network()`, `addr_type()`, `public_spend()` and `public_view()` are methods, not struct fields.
- `deserialize()` can only deserialize into `Block`.
- `AddressType` variants:
  - `Standard` replaced with `Legacy`.
  - `Integrated(PaymentId)` replaced with `LegacyIntegrated([u8; 32])`.
  - Added `Featured`.
