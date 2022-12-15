//! Benchmarking setup for pallet-bridge

use super::*;

#[allow(unused)]
use crate::Pallet as Bridge;
use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;

const SEED: u32 = 0;
const MAX_MEMBERS: u32 = 100;

benchmarks! {
	add_relayer {
		let m in 1 .. MAX_MEMBERS;
		let member: T::AccountId = account("soume_account", m, SEED);
	}: add_relayer(RawOrigin::Root, member.clone())
	verify {
		assert!(Relayer::<T>::get().contains(&member));
	}

	remove_relayer {
		let m in 1 .. MAX_MEMBERS;
		let member: T::AccountId = account("soume_account", m, SEED);
		let mut relayers = Relayer::<T>::get();
		relayers.push(member.clone());
		<Relayer<T>>::put(relayers);
	}: remove_relayer(RawOrigin::Root, member.clone())
	verify {
		assert!(!Relayer::<T>::get().contains(&member));
	}

	impl_benchmark_test_suite!(Bridge, crate::mock::new_test_ext(), crate::mock::Test);
}
