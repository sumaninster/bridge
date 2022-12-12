use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;
use primitives::types::{AccountId, Balance};
use sp_core::H160;

/// Helper function to get the xrp balance of an address slice
fn token_balance_of(address: &[u8]) -> u128 {
	<Test as Config>::Assets::balance(TokenAssetId::get(), &H160::from_slice(address).into()) as u128
}

fn process_transaction(account_address: &[u8; 20]) {
	let transaction_hash = b"6490B68F1116BFE87DDDAD4C5482D1514F9CA8B9B5B5BFD3CF81D8E68745317B";
	let transaction_hash_1 = b"6490B68F1116BFE87DDDAD4C5482D1514F9CA8B9B5B5BFD3CF81D8E68745317C";
	let relayer = create_account(b"6490B68F1116BFE87DDD");
	Bridge::initialize_relayer(&vec![relayer]);
	submit_transaction(relayer, 1_000_000, transaction_hash, account_address, 1);
	submit_transaction(relayer, 1_000_000, transaction_hash_1, account_address, 1);

	Bridge::on_initialize(TxChallengePeriod::get() as u64);
	System::set_block_number(TxChallengePeriod::get() as u64);

	let balance = token_balance_of(account_address);
	assert_eq!(balance, token(2000));
}

fn submit_transaction(
	relayer: AccountId,
	ledger_index: u64,
	transaction_hash: &[u8; 64],
	account_address: &[u8; 20],
	i: u64,
) {
	let transaction = TxData::Payment {
		amount: (i as u128 * token(1000u128)) as Balance,
		address: H160::from_slice(account_address),
	};
	assert_ok!(Bridge::submit_transaction(
		RuntimeOrigin::signed(relayer),
		ledger_index,
		TxHash::from_slice(transaction_hash),
		transaction,
		1234
	));
}

#[test]
fn submit_transaction_replay() {
	new_test_ext().execute_with(|| {
		let relayer = create_account(b"6490B68F1116BFE87DDD");
		let transaction_hash = b"6490B68F1116BFE87DDDAD4C5482D1514F9CA8B9B5B5BFD3CF81D8E68745317B";
		let transaction =
			TxData::Payment { amount: 1000 as Balance, address: H160::from_low_u64_be(555) };
		assert_ok!(Bridge::add_relayer(RuntimeOrigin::root(), relayer));
		assert_ok!(Bridge::submit_transaction(
			RuntimeOrigin::signed(relayer),
			1,
			TxHash::from_slice(transaction_hash),
			transaction.clone(),
			1234
		));
		assert_noop!(
			Bridge::submit_transaction(
				RuntimeOrigin::signed(relayer),
				1,
				TxHash::from_slice(transaction_hash),
				transaction,
				1234
			),
			Error::<Test>::TxReplay
		);
	});
}

#[test]
fn add_transaction_works() {
	new_test_ext().execute_with(|| {
		let transaction_hash = b"6490B68F1116BFE87DDDAD4C5482D1514F9CA8B9B5B5BFD3CF81D8E68745317B";
		let tx_address = b"6490B68F1116BFE87DDD";
		let relayer = create_account(b"6490B68F1116BFE87DDD");
		Bridge::initialize_relayer(&vec![relayer]);
		for i in 0..9u64 {
			let mut transaction_hash = transaction_hash.clone();
			transaction_hash[0] = i as u8;
			submit_transaction(relayer, i * 1_000_000, &transaction_hash, tx_address, i);
		}
	})
}

#[test]
fn process_transaction_works() {
	new_test_ext().execute_with(|| {
		let account_address = b"6490B68F1116BFE87DDC";
		process_transaction(account_address);
	})
}

#[test]
fn process_transaction_challenge_works() {
	new_test_ext().execute_with(|| {
		let transaction_hash = b"6490B68F1116BFE87DDDAD4C5482D1514F9CA8B9B5B5BFD3CF81D8E68745317B";
		let tx_address = b"6490B68F1116BFE87DDC";
		let relayer = create_account(b"6490B68F1116BFE87DDD");
		let challenger = create_account(b"6490B68F1116BFE87DDE");
		Bridge::initialize_relayer(&vec![relayer]);
		submit_transaction(relayer, 1_000_000, transaction_hash, tx_address, 1);
		assert_ok!(Bridge::submit_challenge(
			RuntimeOrigin::signed(challenger),
			TxHash::from_slice(transaction_hash),
		));
		Bridge::on_initialize(TxChallengePeriod::get() as u64);
		System::set_block_number(TxChallengePeriod::get() as u64);

		let balance = token_balance_of(tx_address);
		assert_eq!(balance, 0);
	})
}

#[test]
fn process_transaction_failed_challenge_works() {
	new_test_ext().execute_with(|| {
		let transaction_hash = b"6490B68F1116BFE87DDDAD4C5482D1514F9CA8B9B5B5BFD3CF81D8E68745317B";
		let tx_address = b"6490B68F1116BFE87DDC";
		let relayer = create_account(b"6490B68F1116BFE87DDD");
		let challenger = create_account(b"6490B68F1116BFE87DDE");
		Bridge::initialize_relayer(&vec![relayer]);
		submit_transaction(relayer, 1_000_000, transaction_hash, tx_address, 1);
		assert_ok!(Bridge::submit_challenge(
			RuntimeOrigin::signed(challenger),
			TxHash::from_slice(transaction_hash),
		));
		Bridge::on_initialize(TxChallengePeriod::get() as u64);
		System::set_block_number(TxChallengePeriod::get() as u64);
		let balance = token_balance_of(tx_address);
		assert_eq!(balance, 0);

		assert_ok!(Bridge::failed_challenge(
			RuntimeOrigin::root(),
			TxHash::from_slice(transaction_hash),
		));

		Bridge::on_initialize(TxChallengePeriod::get() as u64 + TxChallengePeriod::get() as u64);
		System::set_block_number(TxChallengePeriod::get() as u64 + TxChallengePeriod::get() as u64);
		let balance = token_balance_of(tx_address);
		assert_eq!(balance, token(1000));
	})
}

#[test]
fn process_transaction_success_challenge_works() {
	new_test_ext().execute_with(|| {
		let transaction_hash = b"6490B68F1116BFE87DDDAD4C5482D1514F9CA8B9B5B5BFD3CF81D8E68745317B";
		let tx_address = b"6490B68F1116BFE87DDC";
		let relayer = create_account(b"6490B68F1116BFE87DDD");
		let challenger = create_account(b"6490B68F1116BFE87DDE");
		Bridge::initialize_relayer(&vec![relayer]);
		submit_transaction(relayer, 1_000_000, transaction_hash, tx_address, 1);
		assert_ok!(Bridge::submit_challenge(
			RuntimeOrigin::signed(challenger),
			TxHash::from_slice(transaction_hash),
		));
		Bridge::on_initialize(TxChallengePeriod::get() as u64);
		System::set_block_number(TxChallengePeriod::get() as u64);
		let balance = token_balance_of(tx_address);
		assert_eq!(balance, 0);

		assert_ok!(Bridge::success_challenge(
			RuntimeOrigin::root(),
			TxHash::from_slice(transaction_hash),
		));

		Bridge::on_initialize(TxChallengePeriod::get() as u64);
		System::set_block_number(TxChallengePeriod::get() as u64);
		let balance = token_balance_of(tx_address);
		assert_eq!(balance, token(0));
	})
}

#[test]
fn clear_storages() {
	new_test_ext().execute_with(|| {
		let process_block = 5;
		let tx_hash_1 = TxHash::from_low_u64_be(123);
		let tx_hash_2 = TxHash::from_low_u64_be(123);

		// <ProcessXRPTransaction<Test>>::append(process_block,tx_hash_1);
		// <ProcessXRPTransaction<Test>>::append(process_block,tx_hash_2);
		<SettledTransactionDetails<Test>>::append(process_block, tx_hash_1);
		<SettledTransactionDetails<Test>>::append(process_block, tx_hash_2);

		let account: AccountId = [1_u8; 20].into();
		<ProcessTransactionDetails<Test>>::insert(
			tx_hash_1,
			(2 as LedgerIndex, Transaction::default(), account),
		);
		<ProcessTransactionDetails<Test>>::insert(
			tx_hash_2,
			(2 as LedgerIndex, Transaction::default(), account),
		);

		Bridge::on_initialize(process_block);

		assert!(<SettledTransactionDetails<Test>>::get(process_block).is_none());
		assert!(<ProcessTransactionDetails<Test>>::get(tx_hash_1).is_none());
		assert!(<ProcessTransactionDetails<Test>>::get(tx_hash_2).is_none());
	});
}
