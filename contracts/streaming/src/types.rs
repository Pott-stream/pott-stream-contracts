use soroban_sdk::{contracttype, Address};

/// A stream of `token` from `sender` to `recipient`, released between `start`
/// and `end` with an optional cliff. See docs/ARCHITECTURE.md, "Streaming contract".
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stream {
    /// Stream identifier.
    pub id: u64,
    /// Account that funded the stream (the circle contract for circle payouts).
    pub sender: Address,
    /// Account that can withdraw vested funds.
    pub recipient: Address,
    /// SEP-41 token or Stellar Asset Contract.
    pub token: Address,
    /// Amount deposited at creation.
    pub total: i128,
    /// Ledger timestamps in seconds; start <= cliff <= end.
    pub start: u64,
    pub cliff: u64,
    pub end: u64,
    /// Amount the recipient has already taken.
    pub withdrawn: i128,
    /// Whether the sender may cancel.
    pub cancelable: bool,
    /// Contract allowed to slash unvested funds (set for circle streams).
    pub controller: Option<Address>,
}
