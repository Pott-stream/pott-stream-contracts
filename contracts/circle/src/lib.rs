#![no_std]

use soroban_sdk::{contract, contractimpl};

#[contract]
pub struct CircleContract;

#[contractimpl]
impl CircleContract {}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn placeholder_registers() {
        let env = Env::default();
        let id = env.register(CircleContract, ());
        env.as_contract(&id, || {});
    }
}
