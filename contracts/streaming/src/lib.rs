#![no_std]

use soroban_sdk::{contract, contractimpl, token::Client as TokenClient, Address, Env};

pub mod error;
mod storage;
pub mod types;

#[cfg(test)]
mod test;

pub use error::Error;
pub use types::Stream;

#[contract]
pub struct StreamingContract;

#[contractimpl]
impl StreamingContract {
    /// Creates a stream and pulls `total` of `token` from `sender` into this
    /// contract. Returns the new stream id.
    ///
    /// A `start` in the past is allowed; `end` must be in the future.
    #[allow(clippy::too_many_arguments)]
    pub fn create_stream(
        env: Env,
        sender: Address,
        recipient: Address,
        token: Address,
        total: i128,
        start: u64,
        cliff: u64,
        end: u64,
        cancelable: bool,
        controller: Option<Address>,
    ) -> Result<u64, Error> {
        sender.require_auth();
        validate(&env, total, start, cliff, end)?;

        let id = storage::take_next_id(&env)?;
        let stream = Stream {
            id,
            sender: sender.clone(),
            recipient,
            token: token.clone(),
            total,
            start,
            cliff,
            end,
            withdrawn: 0,
            cancelable,
            controller,
        };
        // State is written before the transfer.
        storage::write_stream(&env, &stream);

        // TODO(C-07): emit the stream_created event here.
        // TODO(C-09): extend the TTL of the stream entry and the instance here.

        TokenClient::new(&env, &token).transfer(&sender, env.current_contract_address(), &total);

        Ok(id)
    }
}

/// Checks the amount and schedule of a new stream.
///
/// Open decision 3 (math width) is decided: amounts stay in i128 with checked
/// arithmetic. A stream is rejected if `total * (end - start)` would overflow
/// i128, so the vested amount can never overflow later.
fn validate(env: &Env, total: i128, start: u64, cliff: u64, end: u64) -> Result<(), Error> {
    if total <= 0 {
        return Err(Error::InvalidAmount);
    }
    if start > cliff || cliff > end || end <= start {
        return Err(Error::InvalidSchedule);
    }
    // A stream whose end is not in the future is already finished.
    if end <= env.ledger().timestamp() {
        return Err(Error::InvalidSchedule);
    }
    total
        .checked_mul(i128::from(end - start))
        .ok_or(Error::ArithmeticOverflow)?;
    Ok(())
}
