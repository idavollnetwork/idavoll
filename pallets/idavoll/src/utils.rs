#![cfg_attr(not(feature = "std"), no_std)]


pub const LengthLimit: i32 = 32;
pub const InitOrgID: u64 = 1000;
pub const MaxMembers: i32 = 1000;


//#[pallet::error]
pub enum Error {
	/// Transfer amount should be non-zero.
	AmountZero,
	/// Account balance must be greater than or equal to the transfer amount.
	BalanceLow,
	/// Balance should be non-zero.
	BalanceZero,
	/// The signing account has no permission to do the operation.
	NoPermission,
	/// The given asset ID is unknown.
	Unknown,
	/// The origin account is frozen.
	Frozen,
	/// The asset ID is already taken.
	InUse,
	/// Too many zombie accounts in use.
	TooManyZombies,
	/// Attempt to destroy an asset class when non-zombie, reference-bearing accounts exist.
	NotFound,
	/// Invalid witness data given.
	BadWitness,
	/// Minimum balance should be non-zero.
	MinBalanceZero,
	/// A mint operation lead to an overflow.
	Overflow,
	/// Some internal state is broken.
	BadState,
	/// Invalid metadata given.
	BadMetadata,
}