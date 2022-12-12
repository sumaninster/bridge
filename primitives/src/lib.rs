#![cfg_attr(not(feature = "std"), no_std)]

pub mod signature;

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>

pub mod types {
	use sp_runtime::traits::{IdentifyAccount, Verify};
	use crate::signature::BridgeSignature;
	/// An index to a block.
	pub type BlockNumber = u32;
	/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
	pub type Signature = BridgeSignature;
	
	pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

	/// Balance of an account.
	pub type Balance = u128;

	/// Id used for identifying assets.
	pub type TokenId = u32;

	pub type Timestamp = u64;
}

/// Bridge primitive types
pub mod bridge {
	/// An index to a block.
	pub type LedgerIndex = u64;

	/// An Bridge address (classic)
	pub type Address = sp_core::H160;

	/// An Bridge tx hash
	pub type TxHash = sp_core::H512;

	/// The type for identifying the Tx Nonce aka 'Sequence'
	pub type TxNonce = u32;
}