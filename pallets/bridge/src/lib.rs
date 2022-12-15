#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

use frame_support::{
	pallet_prelude::*,
	traits::{
		fungibles::{Inspect, Mutate, Transfer},
		UnixTime,
	},
	weights::constants::RocksDbWeight as DbWeight,
};
use frame_system::pallet_prelude::*;
use sp_std::{prelude::*, vec};
use primitives::{
	bridge::{LedgerIndex, TxHash},
	types::{AccountId, TokenId, Balance, Timestamp},
};
use crate::helpers::{Transaction, TxData};

mod helpers;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests_relayer;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config<AccountId = AccountId> {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Assets: Transfer<Self::AccountId, AssetId = TokenId, Balance = Balance>
		+ Inspect<Self::AccountId, AssetId = TokenId, Balance = Balance>
		+ Mutate<Self::AccountId, AssetId = TokenId, Balance = Balance>;

		/// Allowed origins to add/remove the relayers
		type ApproveOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		///  Asset Id set at runtime
		#[pallet::constant]
		type AssetId: Get<TokenId>;

		/// Challenge Period to wait for a challenge before processing the transaction
		#[pallet::constant]
		type ChallengePeriod: Get<u32>;

		/// Clear Period to wait for a transaction to be cleared from settled storages
		#[pallet::constant]
		type ClearTxPeriod: Get<u32>;

		/// Unix time
		type UnixTime: UnixTime;
	}

	#[pallet::storage]
	#[pallet::getter(fn get_relayer)]
	/// List of all  transaction relayers
	pub type Relayer<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool>;

	#[pallet::storage]
	#[pallet::getter(fn process_transaction)]
	/// Temporary storage to set the transactions ready to be processed at specified block number
	pub type ProcessTransaction<T: Config> =
	StorageMap<_, Blake2_128Concat, T::BlockNumber, Vec<TxHash>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn process_transaction_details)]
	/// Stores submitted transactions waiting to be processed
	/// Transactions will be cleared after `ClearTxPeriod` blocks once processed
	pub type ProcessTransactionDetails<T: Config> =
	StorageMap<_, Identity, TxHash, (LedgerIndex, Transaction, T::AccountId)>;

	#[pallet::storage]
	#[pallet::getter(fn settled_transaction_details)]
	/// Settled transactions stored as history for a specific period
	pub type SettledTransactionDetails<T: Config> =
	StorageMap<_, Twox64Concat, T::BlockNumber, Vec<TxHash>>;

	#[pallet::storage]
	#[pallet::getter(fn challenge_transaction_list)]
	/// Challenge received for a transaction mapped by hash, will be cleared when sudo validates it
	pub type ChallengeTransactionList<T: Config> =
	StorageMap<_, Identity, TxHash, T::AccountId>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TransactionAdded(LedgerIndex, TxHash),
		Processed(LedgerIndex, TxHash),
		RelayerAdded(T::AccountId),
		RelayerRemoved(T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NotPermitted,
		RelayerDoesNotExists,
		TxReplay,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_initialize(n: T::BlockNumber) -> Weight {
			let weights = Self::process_tx(n);
			weights + Self::clear_storages(n)
		}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub relayers: Vec<T::AccountId>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { relayers: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			Pallet::<T>::initialize_relayer(&self.relayers);
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		/// submit a transaction to mint tokens to user account
		pub fn submit_transaction(
			origin: OriginFor<T>,
			ledger_index: LedgerIndex,
			transaction_hash: TxHash,
			transaction: TxData,
			timestamp: Timestamp,
		) -> DispatchResult {
			let relayer = ensure_signed(origin)?;
			let active_relayer = <Relayer<T>>::get(&relayer).unwrap_or(false);
			ensure!(active_relayer, Error::<T>::NotPermitted);
			ensure!(
				Self::process_transaction_details(transaction_hash).is_none(),
				Error::<T>::TxReplay
			);
			Self::add_to_relay(relayer, ledger_index, transaction_hash, transaction, timestamp)
		}

		/// Submit transaction challenge
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn submit_challenge(
			origin: OriginFor<T>,
			transaction_hash: TxHash,
		) -> DispatchResult {
			let challenger = ensure_signed(origin)?;
			ChallengeTransactionList::<T>::insert(&transaction_hash, challenger);
			Ok(())
		}

		/// Sudo verifies that the challenge failed
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn failed_challenge(
			origin: OriginFor<T>,
			transaction_hash: TxHash,
		) -> DispatchResult {
			T::ApproveOrigin::ensure_origin(origin)?;
			ChallengeTransactionList::<T>::remove(&transaction_hash);
			Self::add_to_process(transaction_hash)?;
			Ok(())
		}

		/// Sudo verifies that the challenge is true
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn success_challenge(
			origin: OriginFor<T>,
			transaction_hash: TxHash,
		) -> DispatchResult {
			T::ApproveOrigin::ensure_origin(origin)?;
			ProcessTransactionDetails::<T>::remove(&transaction_hash);
			Ok(())
		}

		/// add a relayer
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn add_relayer(origin: OriginFor<T>, relayer: T::AccountId) -> DispatchResult {
			T::ApproveOrigin::ensure_origin(origin)?;
			Self::initialize_relayer(&vec![relayer]);
			Self::deposit_event(Event::<T>::RelayerAdded(relayer));
			Ok(())
		}

		/// remove a relayer
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn remove_relayer(origin: OriginFor<T>, relayer: T::AccountId) -> DispatchResult {
			T::ApproveOrigin::ensure_origin(origin)?;
			if <Relayer<T>>::contains_key(relayer) {
				<Relayer<T>>::remove(relayer);
				Self::deposit_event(Event::<T>::RelayerRemoved(relayer));
				Ok(())
			} else {
				Err(Error::<T>::RelayerDoesNotExists.into())
			}
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn initialize_relayer(relayers: &Vec<T::AccountId>) {
		for relayer in relayers {
			<Relayer<T>>::insert(relayer, true);
		}
	}

	pub fn process_tx(n: T::BlockNumber) -> Weight {
		let tx_items: Vec<TxHash> = match <ProcessTransaction<T>>::take(n) {
			None => return DbWeight::get().reads(2),
			Some(v) => v,
		};
		let mut reads = 2;
		let mut writes = 0;
		for transaction_hash in tx_items {
			if !<ChallengeTransactionList<T>>::contains_key(transaction_hash) {
				let tx_details = <ProcessTransactionDetails<T>>::get(transaction_hash);
				reads += 1;
				match tx_details {
					None => {},
					Some((ledger_index, ref tx, _relayer)) => {
						match tx.transaction {
							TxData::Payment { amount, address } => {
								let _ = T::Assets::mint_into(
									T::AssetId::get(),
									&address.into(),
									amount,
								);
								writes += 1;
							},
						}
						let clear_block_number = <frame_system::Pallet<T>>::block_number() +
							T::ClearTxPeriod::get().into();
						<SettledTransactionDetails<T>>::append(
							&clear_block_number,
							transaction_hash.clone(),
						);
						writes += 1;
						Self::deposit_event(Event::Processed(ledger_index, transaction_hash));
					},
				}
			}
		}
		DbWeight::get().reads_writes(reads, writes)
	}

	/// Prune settled transaction data from storage
	/// if it was scheduled to do so at block `n`
	pub fn clear_storages(n: T::BlockNumber) -> Weight {
		let mut reads: u64 = 0;
		let mut writes: u64 = 0;
		reads += 1;
		if <SettledTransactionDetails<T>>::contains_key(n) {
			if let Some(tx_hashes) = <SettledTransactionDetails<T>>::take(n) {
				writes += 1 + tx_hashes.len() as u64;
				for tx_hash in tx_hashes {
					<ProcessTransactionDetails<T>>::remove(tx_hash);
				}
			}
		}
		DbWeight::get().reads_writes(reads, writes)
	}

	pub fn add_to_relay(
		relayer: T::AccountId,
		ledger_index: LedgerIndex,
		transaction_hash: TxHash,
		transaction: TxData,
		timestamp: Timestamp,
	) -> DispatchResult {
		let val = Transaction { transaction_hash, transaction, timestamp };
		<ProcessTransactionDetails<T>>::insert(&transaction_hash, (ledger_index, val, relayer));

		Self::add_to_process(transaction_hash)?;
		Self::deposit_event(Event::TransactionAdded(ledger_index, transaction_hash));
		Ok(())
	}

	pub fn add_to_process(transaction_hash: TxHash) -> DispatchResult {
		let process_block_number =
			<frame_system::Pallet<T>>::block_number() + T::ChallengePeriod::get().into();
		ProcessTransaction::<T>::append(&process_block_number, transaction_hash);
		Ok(())
	}
}
