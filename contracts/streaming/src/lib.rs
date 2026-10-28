#![no_std]

use soroban_sdk::{contract, contractimpl};

pub mod error;
// The storage helpers have no caller until create_stream lands in C-03.
#[cfg_attr(not(test), allow(dead_code))]
mod storage;
pub mod types;

pub use error::Error;
pub use types::Stream;

#[contract]
pub struct StreamingContract;

#[contractimpl]
impl StreamingContract {}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn placeholder_registers() {
        let env = Env::default();
        let id = env.register(StreamingContract, ());
        env.as_contract(&id, || {});
    }
}
