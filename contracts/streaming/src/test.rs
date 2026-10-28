use soroban_sdk::{
    testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, IntoVal,
};

use crate::{storage, Error, Stream, StreamingContract, StreamingContractClient};

const NOW: u64 = 1_000;

/// A streaming contract, a Stellar asset contract used as the token, and a
/// sender holding `funds` of it. Auths are NOT mocked: tests that need the
/// sender's authorization call `env.mock_all_auths()` themselves.
struct Fixture {
    env: Env,
    contract: Address,
    token: Address,
    sender: Address,
    recipient: Address,
}

impl Fixture {
    fn new(funds: i128) -> Self {
        let env = Env::default();
        env.ledger().set_timestamp(NOW);

        let contract = env.register(StreamingContract, ());
        let admin = Address::generate(&env);
        let token = env
            .register_stellar_asset_contract_v2(admin.clone())
            .address();
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);

        // Authorize only the mint, so create_stream stays unauthorized by default.
        env.mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &token,
                fn_name: "mint",
                args: (sender.clone(), funds).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        StellarAssetClient::new(&env, &token).mint(&sender, &funds);

        Fixture {
            env,
            contract,
            token,
            sender,
            recipient,
        }
    }

    fn client(&self) -> StreamingContractClient<'_> {
        StreamingContractClient::new(&self.env, &self.contract)
    }

    fn token(&self) -> TokenClient<'_> {
        TokenClient::new(&self.env, &self.token)
    }

    fn try_create(
        &self,
        total: i128,
        start: u64,
        cliff: u64,
        end: u64,
        controller: Option<Address>,
    ) -> Result<u64, Error> {
        match self.client().try_create_stream(
            &self.sender,
            &self.recipient,
            &self.token,
            &total,
            &start,
            &cliff,
            &end,
            &true,
            &controller,
        ) {
            Ok(Ok(id)) => Ok(id),
            Ok(Err(_)) => panic!("unexpected conversion error"),
            Err(Ok(error)) => Err(error),
            Err(Err(_)) => panic!("unexpected host error"),
        }
    }

    fn stored(&self, id: u64) -> Result<Stream, Error> {
        self.env
            .as_contract(&self.contract, || storage::read_stream(&self.env, id))
    }

    fn assert_nothing_moved(&self, funds: i128) {
        assert_eq!(self.token().balance(&self.sender), funds);
        assert_eq!(self.token().balance(&self.contract), 0);
    }
}

#[test]
fn valid_stream_moves_total_and_stores_stream() {
    let f = Fixture::new(10_000);
    f.env.mock_all_auths();

    // start in the past, cliff at now, end in the future
    let id = f.try_create(1_000, 900, 1_000, 2_000, None).unwrap();

    assert_eq!(id, 1);
    assert_eq!(f.token().balance(&f.sender), 9_000);
    assert_eq!(f.token().balance(&f.contract), 1_000);
    assert_eq!(f.token().balance(&f.recipient), 0);
    assert_eq!(
        f.stored(1),
        Ok(Stream {
            id: 1,
            sender: f.sender.clone(),
            recipient: f.recipient.clone(),
            token: f.token.clone(),
            total: 1_000,
            start: 900,
            cliff: 1_000,
            end: 2_000,
            withdrawn: 0,
            cancelable: true,
            controller: None,
        })
    );
}

#[test]
fn create_stream_requires_the_senders_authorization() {
    let f = Fixture::new(10_000);
    f.env.mock_all_auths();
    f.try_create(1_000, 900, 1_000, 2_000, None).unwrap();

    let auths = f.env.auths();
    assert_eq!(auths.len(), 1);
    assert_eq!(auths[0].0, f.sender);
}

#[test]
fn create_stream_without_authorization_fails() {
    // The sender is funded, so only the missing authorization can fail the call.
    let f = Fixture::new(10_000);

    let result = f.client().try_create_stream(
        &f.sender,
        &f.recipient,
        &f.token,
        &1_000,
        &900,
        &1_000,
        &2_000,
        &true,
        &None,
    );

    assert!(result.is_err());
    f.assert_nothing_moved(10_000);
    assert_eq!(f.stored(1), Err(Error::StreamNotFound));
}

#[test]
fn ids_increment_from_one() {
    let f = Fixture::new(10_000);
    f.env.mock_all_auths();

    assert_eq!(f.try_create(100, 900, 1_000, 2_000, None), Ok(1));
    assert_eq!(f.try_create(200, 900, 1_000, 2_000, None), Ok(2));
    assert_eq!(f.try_create(300, 900, 1_000, 2_000, None), Ok(3));
    assert_eq!(f.stored(2).unwrap().total, 200);
    assert_eq!(f.token().balance(&f.contract), 600);
}

#[test]
fn failed_creation_does_not_use_an_id() {
    let f = Fixture::new(10_000);
    f.env.mock_all_auths();

    assert_eq!(
        f.try_create(0, 900, 1_000, 2_000, None),
        Err(Error::InvalidAmount)
    );
    assert_eq!(f.try_create(100, 900, 1_000, 2_000, None), Ok(1));
}

#[test]
fn zero_total_fails() {
    let f = Fixture::new(10_000);
    f.env.mock_all_auths();

    assert_eq!(
        f.try_create(0, 900, 1_000, 2_000, None),
        Err(Error::InvalidAmount)
    );
    f.assert_nothing_moved(10_000);
}

#[test]
fn negative_total_fails() {
    let f = Fixture::new(10_000);
    f.env.mock_all_auths();

    assert_eq!(
        f.try_create(-1, 900, 1_000, 2_000, None),
        Err(Error::InvalidAmount)
    );
    assert_eq!(
        f.try_create(i128::MIN, 900, 1_000, 2_000, None),
        Err(Error::InvalidAmount)
    );
    f.assert_nothing_moved(10_000);
}

#[test]
fn cliff_before_start_fails() {
    let f = Fixture::new(10_000);
    f.env.mock_all_auths();

    assert_eq!(
        f.try_create(1_000, 1_500, 1_400, 2_000, None),
        Err(Error::InvalidSchedule)
    );
    f.assert_nothing_moved(10_000);
}

#[test]
fn cliff_after_end_fails() {
    let f = Fixture::new(10_000);
    f.env.mock_all_auths();

    assert_eq!(
        f.try_create(1_000, 1_500, 2_500, 2_000, None),
        Err(Error::InvalidSchedule)
    );
    f.assert_nothing_moved(10_000);
}

#[test]
fn end_equal_to_start_fails() {
    let f = Fixture::new(10_000);
    f.env.mock_all_auths();

    assert_eq!(
        f.try_create(1_000, 2_000, 2_000, 2_000, None),
        Err(Error::InvalidSchedule)
    );
    f.assert_nothing_moved(10_000);
}

#[test]
fn end_before_start_fails() {
    let f = Fixture::new(10_000);
    f.env.mock_all_auths();

    assert_eq!(
        f.try_create(1_000, 3_000, 3_000, 2_000, None),
        Err(Error::InvalidSchedule)
    );
    f.assert_nothing_moved(10_000);
}

#[test]
fn end_in_the_past_fails() {
    let f = Fixture::new(10_000);
    f.env.mock_all_auths();

    // end is before now
    assert_eq!(
        f.try_create(1_000, 100, 200, NOW - 1, None),
        Err(Error::InvalidSchedule)
    );
    // end is exactly now: the stream is already finished
    assert_eq!(
        f.try_create(1_000, 100, 200, NOW, None),
        Err(Error::InvalidSchedule)
    );
    f.assert_nothing_moved(10_000);
}

#[test]
fn end_one_second_in_the_future_is_accepted() {
    let f = Fixture::new(10_000);
    f.env.mock_all_auths();

    assert_eq!(f.try_create(1_000, 100, 200, NOW + 1, None), Ok(1));
}

#[test]
fn total_that_overflows_with_the_duration_fails() {
    let f = Fixture::new(10_000);
    f.env.mock_all_auths();

    assert_eq!(
        f.try_create(i128::MAX, 900, 1_000, 2_000, None),
        Err(Error::ArithmeticOverflow)
    );
    f.assert_nothing_moved(10_000);
}

#[test]
fn overflow_check_is_exact_at_the_boundary() {
    // duration = 2_000 - 1_000 = 1_000 seconds
    let largest_ok = i128::MAX / 1_000;

    let f = Fixture::new(largest_ok);
    f.env.mock_all_auths();
    assert_eq!(f.try_create(largest_ok, 1_000, 1_000, 2_000, None), Ok(1));
    assert_eq!(f.token().balance(&f.contract), largest_ok);

    let f = Fixture::new(largest_ok + 1);
    f.env.mock_all_auths();
    assert_eq!(
        f.try_create(largest_ok + 1, 1_000, 1_000, 2_000, None),
        Err(Error::ArithmeticOverflow)
    );
}

#[test]
fn stream_with_a_controller_stores_it() {
    let f = Fixture::new(10_000);
    f.env.mock_all_auths();
    let controller = Address::generate(&f.env);

    let id = f
        .try_create(1_000, 900, 1_000, 2_000, Some(controller.clone()))
        .unwrap();

    assert_eq!(f.stored(id).unwrap().controller, Some(controller));
}
