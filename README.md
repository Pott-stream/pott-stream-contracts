# pott-stream-contracts

Soroban smart contracts for Pott-stream: a savings circle on Stellar where each round's pot reaches the winner as a stream instead of one lump sum.

This repo holds two contracts:

- `contracts/streaming`: a streaming engine for linear and cliff streams with cancel.
- `contracts/circle`: members, rounds, contributions, collateral and defaults. It pays winners through the streaming contract.

The design is in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md). Contributor and agent rules are in [CLAUDE.md](CLAUDE.md).

> **Experimental software. Testnet only.**
> These contracts are unaudited, under active development and not ready to hold real value. Do not deploy them to Stellar mainnet and do not use real funds. Nothing here is financial or legal advice.
>
> Pott-stream is not affiliated with or endorsed by the Stellar Development Foundation.

## Requirements

- Rust 1.95.0 (installed automatically by `rustup` from `rust-toolchain.toml`, together with `rustfmt`, `clippy` and the `wasm32v1-none` target)
- [Stellar CLI](https://developers.stellar.org/docs/tools/cli) 26.0.0

## Checks

Run these four commands from the repo root. CI runs the same ones on every pull request.

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
stellar contract build
```

`stellar contract build` compiles each contract to WASM for the `wasm32v1-none` target. Build with it rather than plain `cargo build`.

## License

Apache-2.0. See [LICENSE](LICENSE).
