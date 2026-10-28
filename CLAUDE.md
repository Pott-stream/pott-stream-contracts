# Pott-stream contracts (pott-stream-contracts)

## What this repo is

Soroban smart contracts for Pott-stream: a savings circle on Stellar where each round's pot reaches the winner as a stream instead of one lump sum. Two contracts: a streaming contract and a circle contract. Testnet only, experimental.

The full design is in `docs/ARCHITECTURE.md`. That document is the source of truth. Issues are numbered C-01 to C-19 there.

## Build order

1. Streaming contract first (C-01 to C-11).
2. Circle contract after that (C-12 to C-19).
3. Do not start the circle contract until the streaming invariants pass under property tests.

## Commands (run these before saying a task is done)

- `cargo fmt --all -- --check` : fails if formatting differs, so style never becomes a review comment.
- `cargo clippy --all-targets -- -D warnings` : lints the code and treats warnings as errors.
- `cargo test --workspace` : runs unit tests and property tests.
- `stellar contract build` : builds each contract to WASM. Run it because code that passes tests can still fail to compile for Soroban.

The WASM build target is `wasm32v1-none`, not `wasm32-unknown-unknown` (which fails on Rust 1.82 and newer). `rust-toolchain.toml` installs it; `stellar contract build` uses it.

Always show the real output of these commands. Never summarize them from memory. For every command you run, say in the chat why you are running it and what it does, and explain the result.

## Layout

- `contracts/streaming/` : the streaming contract
- `contracts/circle/` : the circle contract
- `docs/ARCHITECTURE.md` : design, formulas, issue list
- `CLAUDE.md` : this file
- `Cargo.toml` : workspace manifest; pins `soroban-sdk`
- `Cargo.lock` : locked dependency versions (committed)
- `rust-toolchain.toml` : pins Rust, with `rustfmt`, `clippy` and the `wasm32v1-none` target
- `.github/workflows/ci.yml` : pull request CI (fmt, clippy, test, contract build)
- `LICENSE` : Apache-2.0
- `README.md` : project overview, disclaimer and how to run the checks
- `.gitignore` : excludes `target/`, `.env` files and key material

## Rules for contract code

- Amounts are `i128`. Use checked arithmetic (`checked_add`, `checked_sub`, `checked_mul`, `checked_div`).
- Never use `unwrap`, `expect` or `panic!` on paths users can reach. Return a typed error enum with `#[contracterror]`.
- Multiply before dividing. Round down on payouts. Round up on collateral.
- Call `require_auth` on the correct address in every state-changing function.
- Storage: persistent for circles and streams, instance for configuration. Extend TTL on writes and on reads that matter. Never keep money-related data in temporary storage.
- Use `env.ledger().timestamp()` for time and nothing else.
- Loops must be bounded. The member count per circle is capped.
- Only tokens on the admin-managed allowlist may be used.
- Emit an event for every state change listed in the architecture doc.
- No upgrade function in v1.
- Use `#![no_std]` and the latest released `soroban-sdk`, pinned in the workspace `Cargo.toml`.

## Design-critical: do not change without asking

- The vesting formula (architecture doc, "Streaming contract").
- The collateral formula and the slashing rebase (architecture doc, "Defaults and collateral").
- Open decisions 1 and 2 in the architecture doc are undecided. Do not invent answers. Leave a `TODO` that names the decision and ask me.
- Math width (decision 3, decided): amounts stay `i128` with checked arithmetic, and `create_stream` rejects any stream where `total * (end - start)` overflows `i128`. No 256-bit intermediate.

## How to work

- One issue per branch and per pull request. Branch names look like `c-05-withdraw`.
- A pull request description lists the issue ID, what changed, and the test output.
- For High issues, write a short plan first and wait for my approval before coding.
- Tests use `soroban-sdk` test utilities. Use `proptest` for invariants. Time-dependent tests advance the ledger.
- Never commit secrets: no secret keys, seed phrases or `.env` files. Testnet identities live in the Stellar CLI keystore, not in the repo.
- Keep this file current. At the end of each task, tick the box in the Status list below.

## Status

- [x] C-01 Scaffold workspace and CI
- [x] C-02 Stream data model, errors, storage keys
- [x] C-03 create_stream
- [x] C-04 vested and withdrawable views
- [ ] C-05 withdraw
- [ ] C-06 cancel
- [ ] C-07 Stream events
- [ ] C-08 Stream property tests
- [ ] C-09 TTL extension for streams
- [ ] C-10 Controller role and slash_unvested
- [ ] C-11 Token allowlist
- [ ] C-12 Circle data model and create_circle
- [ ] C-13 join with collateral
- [ ] C-14 start and contribute
- [ ] C-15 settle_round
- [ ] C-16 Default handling
- [ ] C-17 Pre-win default policy
- [ ] C-18 Circle completion
- [ ] C-19 Model-based fuzz test
