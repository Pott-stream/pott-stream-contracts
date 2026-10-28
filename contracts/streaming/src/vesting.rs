use crate::error::Error;
use crate::types::Stream;

// TODO(C-10): `slash_unvested` rebases the schedule, so vested_at will have to
// change for slashed streams (a fixed base plus a linear part on the reduced
// remainder). Until then every stream vests linearly from `start` to `end`.

/// The amount of `stream` vested at time `now`, rounded down.
///
/// 0 before the cliff, `total` from `end` on, and
/// floor(total * (now - start) / (end - start)) in between. The product is
/// taken before the division. `create_stream` rejects streams where
/// `total * (end - start)` overflows i128, so the multiplication cannot
/// overflow for a stored stream; the checked operations are a second line of
/// defense.
pub(crate) fn vested_at(stream: &Stream, now: u64) -> Result<i128, Error> {
    if now < stream.cliff {
        return Ok(0);
    }
    if now >= stream.end {
        return Ok(stream.total);
    }
    let elapsed = now
        .checked_sub(stream.start)
        .ok_or(Error::ArithmeticOverflow)?;
    let duration = stream
        .end
        .checked_sub(stream.start)
        .ok_or(Error::ArithmeticOverflow)?;
    stream
        .total
        .checked_mul(i128::from(elapsed))
        .ok_or(Error::ArithmeticOverflow)?
        .checked_div(i128::from(duration))
        .ok_or(Error::ArithmeticOverflow)
}

/// The amount the recipient can withdraw at `now`: vested minus withdrawn.
pub(crate) fn withdrawable_at(stream: &Stream, now: u64) -> Result<i128, Error> {
    vested_at(stream, now)?
        .checked_sub(stream.withdrawn)
        .ok_or(Error::ArithmeticOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    fn stream(total: i128, start: u64, cliff: u64, end: u64) -> Stream {
        let env = Env::default();
        Stream {
            id: 1,
            sender: Address::generate(&env),
            recipient: Address::generate(&env),
            token: Address::generate(&env),
            total,
            start,
            cliff,
            end,
            withdrawn: 0,
            cancelable: true,
            controller: None,
        }
    }

    #[test]
    fn zero_before_the_cliff() {
        // start 100, cliff 200, end 300
        let s = stream(1_000, 100, 200, 300);
        assert_eq!(vested_at(&s, 0), Ok(0));
        assert_eq!(vested_at(&s, 100), Ok(0));
        assert_eq!(vested_at(&s, 199), Ok(0));
    }

    #[test]
    fn at_the_cliff_gives_the_formula_value() {
        // 100 of 200 seconds have passed: half of the total unlocks at once
        let s = stream(1_000, 100, 200, 300);
        assert_eq!(vested_at(&s, 200), Ok(500));
    }

    #[test]
    fn at_the_cliff_is_zero_when_cliff_equals_start() {
        let s = stream(1_000, 100, 100, 300);
        assert_eq!(vested_at(&s, 99), Ok(0));
        assert_eq!(vested_at(&s, 100), Ok(0));
        assert_eq!(vested_at(&s, 101), Ok(5));
    }

    #[test]
    fn midpoint() {
        let s = stream(1_000, 100, 100, 300);
        assert_eq!(vested_at(&s, 200), Ok(500));
    }

    #[test]
    fn total_exactly_at_the_end() {
        let s = stream(1_000, 100, 200, 300);
        assert_eq!(vested_at(&s, 299), Ok(995));
        assert_eq!(vested_at(&s, 300), Ok(1_000));
    }

    #[test]
    fn total_long_after_the_end() {
        let s = stream(1_000, 100, 200, 300);
        assert_eq!(vested_at(&s, 301), Ok(1_000));
        assert_eq!(vested_at(&s, 1_000_000), Ok(1_000));
        assert_eq!(vested_at(&s, u64::MAX), Ok(1_000));
    }

    #[test]
    fn rounds_down() {
        // 10 over 3 seconds: 3.33 -> 3 and 6.67 -> 6
        let s = stream(10, 0, 0, 3);
        assert_eq!(vested_at(&s, 1), Ok(3));
        assert_eq!(vested_at(&s, 2), Ok(6));
        assert_eq!(vested_at(&s, 3), Ok(10));
    }

    #[test]
    fn cliff_equal_to_end_gives_zero_until_the_end() {
        let s = stream(1_000, 100, 300, 300);
        assert_eq!(vested_at(&s, 100), Ok(0));
        assert_eq!(vested_at(&s, 299), Ok(0));
        assert_eq!(vested_at(&s, 300), Ok(1_000));
        assert_eq!(vested_at(&s, 301), Ok(1_000));
    }

    #[test]
    fn overflowing_product_is_an_error_not_a_panic() {
        // create_stream never stores such a stream; the math still must not panic.
        let s = stream(i128::MAX, 0, 0, 10);
        assert_eq!(vested_at(&s, 1), Ok(i128::MAX / 10));
        assert_eq!(vested_at(&s, 2), Err(Error::ArithmeticOverflow));
    }

    #[test]
    fn corrupt_cliff_before_start_is_an_error_not_a_panic() {
        // create_stream rejects cliff < start; a corrupt stream must not
        // underflow now - start and panic.
        let s = stream(1_000, 100, 0, 300);
        assert_eq!(vested_at(&s, 50), Err(Error::ArithmeticOverflow));
    }

    #[test]
    fn withdrawable_is_vested_minus_withdrawn() {
        let mut s = stream(1_000, 100, 100, 300);
        s.withdrawn = 200;
        assert_eq!(withdrawable_at(&s, 200), Ok(300));
        assert_eq!(withdrawable_at(&s, 300), Ok(800));
    }

    #[test]
    fn withdrawable_is_negative_if_withdrawn_exceeds_vested() {
        // Cannot happen while withdrawn <= vested holds (see the invariants in
        // the architecture doc); the function reports the plain difference.
        let mut s = stream(1_000, 100, 100, 300);
        s.withdrawn = 200;
        assert_eq!(withdrawable_at(&s, 100), Ok(-200));
    }

    #[test]
    fn withdrawable_with_nothing_withdrawn_equals_vested() {
        let s = stream(1_000, 100, 100, 300);
        assert_eq!(withdrawable_at(&s, 150), vested_at(&s, 150));
    }

    #[test]
    fn vested_never_decreases_second_by_second() {
        for (total, start, cliff, end) in [
            (1_000, 5, 8, 20),
            (7, 5, 5, 20),
            (10, 0, 0, 3),
            (1, 3, 10, 11),
            (999_999_999_999, 2, 2, 30),
        ] {
            let s = stream(total, start, cliff, end);
            let mut previous = 0;
            for now in 0..=end + 5 {
                let v = vested_at(&s, now).unwrap();
                assert!(v >= previous, "vested went down at t={now}");
                assert!(v <= total, "vested above total at t={now}");
                previous = v;
            }
            assert_eq!(previous, total);
        }
    }
}
