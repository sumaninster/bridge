use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;
use sp_core::H160;
use sp_runtime::traits::BadOrigin;

#[test]
fn test_approved_origin_enforced() {
	new_test_ext().execute_with(|| {
		let relayer_address = b"6490B68F1116BFE87DDD";
		let relayer = AccountId::from(H160::from_slice(relayer_address));
		let account_address = b"6490B68F1116BFE87DDD";
		let account = AccountId::from(H160::from_slice(account_address));
		// Should throw error on un_approved origin
		assert_noop!(Bridge::add_relayer(RuntimeOrigin::signed(account), relayer), BadOrigin);
		// Should work with approved origin
		assert_ok!(Bridge::add_relayer(RuntimeOrigin::root(), relayer));
	})
}

#[test]
fn test_add_relayer_works() {
	new_test_ext().execute_with(|| {
		let relayer_address = b"6490B68F1116BFE87DDD";
		let relayer = AccountId::from(H160::from_slice(relayer_address));
		let _ = Bridge::add_relayer(RuntimeOrigin::root(), relayer);
		assert_eq!(<Relayer<Test>>::iter_values().collect::<Vec<_>>(), vec![true]);

		let relayer_address2 = b"6490B68F1116BFE87DDE";
		let relayer2 = AccountId::from(H160::from_slice(relayer_address2));

		assert_ok!(Bridge::add_relayer(RuntimeOrigin::root(), relayer2));
		assert_eq!(<Relayer<Test>>::iter_values().collect::<Vec<_>>(), vec![true, true]);
	})
}

#[test]
fn test_remove_relayer_works() {
	new_test_ext().execute_with(|| {
		let relayer_address = b"6490B68F1116BFE87DDD";
		let relayer = AccountId::from(H160::from_slice(relayer_address));
		let relayer_address2 = b"6490B68F1116BFE87DDE";
		let relayer2 = AccountId::from(H160::from_slice(relayer_address2));

		let _ = Bridge::add_relayer(RuntimeOrigin::root(), relayer);
		let _ = Bridge::add_relayer(RuntimeOrigin::root(), relayer2);

		// Test removing an existing relayer.
		assert_ok!(Bridge::remove_relayer(RuntimeOrigin::root(), relayer));
		assert_eq!(<Relayer<Test>>::iter_values().collect::<Vec<_>>(), vec![true]);

		// Should throw error if non-existing relayer is tried to removed.
		assert_noop!(
			Bridge::remove_relayer(RuntimeOrigin::root(), relayer),
			Error::<Test>::RelayerDoesNotExists
		);
	})
}

#[test]
fn test_is_relayer_works() {
	new_test_ext().execute_with(|| {
		let relayer_address = b"6490B68F1116BFE87DDD";
		let relayer = AccountId::from(H160::from_slice(relayer_address));
		let relayer_address2 = b"6490B68F1116BFE87DDE";
		let relayer2 = AccountId::from(H160::from_slice(relayer_address2));
		let _ = Bridge::add_relayer(RuntimeOrigin::root(), relayer);
		// Positive test
		assert_eq!(Bridge::get_relayer(relayer), Some(true));
		// Negative test
		assert_eq!(Bridge::get_relayer(relayer2), None);
	})
}
