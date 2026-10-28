use soroban_sdk::contracterror;

/// Errors returned by the streaming contract.
///
/// The numeric codes are part of the contract interface: never reorder or
/// reuse them. New variants go at the end.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// The amount is zero or negative.
    InvalidAmount = 1,
    /// start <= cliff <= end is violated, or end == start.
    InvalidSchedule = 2,
    /// No stream exists for the given id.
    StreamNotFound = 3,
    /// The sender tried to cancel a stream that is not cancelable.
    NotCancelable = 4,
    /// The withdrawable amount is zero.
    NothingToWithdraw = 5,
    /// A checked arithmetic operation overflowed.
    ArithmeticOverflow = 6,
    /// The caller is not the stream's controller.
    UnauthorizedController = 7,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_codes_are_stable() {
        assert_eq!(Error::InvalidAmount as u32, 1);
        assert_eq!(Error::InvalidSchedule as u32, 2);
        assert_eq!(Error::StreamNotFound as u32, 3);
        assert_eq!(Error::NotCancelable as u32, 4);
        assert_eq!(Error::NothingToWithdraw as u32, 5);
        assert_eq!(Error::ArithmeticOverflow as u32, 6);
        assert_eq!(Error::UnauthorizedController as u32, 7);
    }
}
