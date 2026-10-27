#![no_std]

use soroban_sdk::{contract, contractimpl};

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
