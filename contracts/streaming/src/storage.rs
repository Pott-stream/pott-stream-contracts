use soroban_sdk::{contracttype, Env};

use crate::error::Error;
use crate::types::Stream;

/// Storage keys. Streams are persistent; the id counter is instance storage.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// A stream by id (persistent).
    Stream(u64),
    /// The id the next created stream will get (instance).
    NextStreamId,
}

/// The id given to the first stream.
const FIRST_STREAM_ID: u64 = 1;

/// Reads a stream, or `StreamNotFound` if the id was never written.
// No contract function reads a stream until C-04/C-05; remove this then.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn read_stream(env: &Env, id: u64) -> Result<Stream, Error> {
    // TODO(C-09): extend the TTL of this entry on reads that matter.
    env.storage()
        .persistent()
        .get(&DataKey::Stream(id))
        .ok_or(Error::StreamNotFound)
}

/// Writes a stream under its own id, replacing any previous value.
pub(crate) fn write_stream(env: &Env, stream: &Stream) {
    // TODO(C-09): extend the TTL of this entry (and the instance) on write.
    env.storage()
        .persistent()
        .set(&DataKey::Stream(stream.id), stream);
}

/// Returns the next stream id and advances the counter. Ids start at 1.
pub(crate) fn take_next_id(env: &Env) -> Result<u64, Error> {
    let id: u64 = env
        .storage()
        .instance()
        .get(&DataKey::NextStreamId)
        .unwrap_or(FIRST_STREAM_ID);
    let next = id.checked_add(1).ok_or(Error::ArithmeticOverflow)?;
    // TODO(C-09): extend the instance TTL on write.
    env.storage().instance().set(&DataKey::NextStreamId, &next);
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StreamingContract;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::Address;

    fn sample_stream(env: &Env, id: u64) -> Stream {
        Stream {
            id,
            sender: Address::generate(env),
            recipient: Address::generate(env),
            token: Address::generate(env),
            total: 1_000,
            start: 100,
            cliff: 150,
            end: 200,
            withdrawn: 25,
            cancelable: true,
            controller: Some(Address::generate(env)),
        }
    }

    #[test]
    fn stream_round_trips_unchanged() {
        let env = Env::default();
        let contract = env.register(StreamingContract, ());
        let stream = sample_stream(&env, 7);

        env.as_contract(&contract, || {
            write_stream(&env, &stream);
            assert_eq!(read_stream(&env, 7), Ok(stream.clone()));
        });
    }

    #[test]
    fn stream_without_controller_round_trips() {
        let env = Env::default();
        let contract = env.register(StreamingContract, ());
        let mut stream = sample_stream(&env, 1);
        stream.controller = None;

        env.as_contract(&contract, || {
            write_stream(&env, &stream);
            assert_eq!(read_stream(&env, 1), Ok(stream.clone()));
        });
    }

    #[test]
    fn missing_stream_is_not_found() {
        let env = Env::default();
        let contract = env.register(StreamingContract, ());

        env.as_contract(&contract, || {
            assert_eq!(read_stream(&env, 1), Err(Error::StreamNotFound));
            write_stream(&env, &sample_stream(&env, 1));
            assert_eq!(read_stream(&env, 2), Err(Error::StreamNotFound));
        });
    }

    #[test]
    fn next_id_starts_at_one_and_never_repeats() {
        let env = Env::default();
        let contract = env.register(StreamingContract, ());

        env.as_contract(&contract, || {
            assert_eq!(take_next_id(&env), Ok(1));
            assert_eq!(take_next_id(&env), Ok(2));
            assert_eq!(take_next_id(&env), Ok(3));
        });
    }

    #[test]
    fn next_id_overflow_is_an_error() {
        let env = Env::default();
        let contract = env.register(StreamingContract, ());

        env.as_contract(&contract, || {
            env.storage()
                .instance()
                .set(&DataKey::NextStreamId, &u64::MAX);
            assert_eq!(take_next_id(&env), Err(Error::ArithmeticOverflow));
            // A failed call must not hand out the same id twice later.
            assert_eq!(take_next_id(&env), Err(Error::ArithmeticOverflow));
        });
    }
}
