use codec::{Decode, Encode};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_core::H160;

use primitives::{
	bridge::TxHash,
	types::Balance,
};

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Transaction {
	pub transaction_hash: TxHash,
	pub transaction: TxData,
	pub timestamp: u64,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub enum TxData {
	Payment { amount: Balance, address: H160 },
}

impl Default for Transaction {
	fn default() -> Self {
		Transaction {
			transaction_hash: TxHash::default(),
			transaction: TxData::default(),
			timestamp: 0,
		}
	}
}

impl Default for TxData {
	fn default() -> Self {
		TxData::Payment { amount: 0, address: H160::default() }
	}
}
