//! Benchmarking setup for pallet-propgation
#![allow(unused_imports)]
use super::*;

#[allow(unused_imports)]
use crate::Pallet as Staking;

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, vec, whitelisted_caller};
use frame_support::{assert_ok, sp_runtime::traits::StaticLookup, traits::Currency};
use frame_system::{EventRecord, RawOrigin};
use pallet_staking::{Pallet as FrameStaking, RewardDestination, ValidatorPrefs};

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	let events = frame_system::Pallet::<T>::events();
	let system_event: <T as frame_system::Config>::Event = generic_event.into();
	// compare to the last event record
	let EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}

fn prep_bond_and_validate<T: Config>(
	validate_also: bool,
	caller: T::AccountId,
	bonder: T::AccountId,
	threshold: T::AccountId,
) {
	let reward_destination = RewardDestination::Account(caller.clone());
	let bond = <T as pallet_staking::Config>::Currency::minimum_balance() * 10u32.into();
	<T as Config>::Currency::make_free_balance_be(
		&bonder.clone(),
		<T as Config>::Currency::minimum_balance() * 10u32.into(),
	);
	assert_ok!(<FrameStaking<T>>::bond(
		RawOrigin::Signed(bonder).into(),
		T::Lookup::unlookup(caller.clone()),
		bond,
		reward_destination,
	));

	if validate_also {
		assert_ok!(<Staking<T>>::validate(
			RawOrigin::Signed(caller.clone()).into(),
			ValidatorPrefs::default(),
			vec![20, 20],
			threshold,
		));
	}
}

const SEED: u32 = 0;

benchmarks! {
	change_endpoint {
		let caller: T::AccountId = whitelisted_caller();
		let bonder: T::AccountId = account("bond", 0, SEED);
		let threshold: T::AccountId = account("threshold", 0, SEED);

		prep_bond_and_validate::<T>(true, caller.clone(), bonder.clone(), threshold.clone());


	}:  _(RawOrigin::Signed(caller.clone()), vec![30])
	verify {
		assert_last_event::<T>(Event::EndpointChanged(caller, vec![30]).into());
	}

	change_threshold_accounts {
		let caller: T::AccountId = whitelisted_caller();
		let bonder: T::AccountId = account("bond", 0, SEED);
		let threshold: T::AccountId = account("threshold", 0, SEED);

		prep_bond_and_validate::<T>(true, caller.clone(), bonder.clone(), threshold.clone());


	}:  _(RawOrigin::Signed(caller.clone()), bonder.clone())
	verify {
		assert_last_event::<T>(Event::ThresholdAccountChanged(bonder.clone(), bonder).into());
	}


	withdraw_unbonded {
		let caller: T::AccountId = whitelisted_caller();
		let bonder: T::AccountId = account("bond", 0, SEED);
		let threshold: T::AccountId = account("threshold", 0, SEED);

		prep_bond_and_validate::<T>(true, caller.clone(), bonder.clone(), threshold.clone());
		let bond = <T as pallet_staking::Config>::Currency::minimum_balance() * 10u32.into();

		// assume fully unbonded as slightly more weight, but not enough to handle partial unbond
		assert_ok!(<FrameStaking<T>>::unbond(
			RawOrigin::Signed(caller.clone()).into(),
			bond,
		));


	}:  _(RawOrigin::Signed(caller.clone()), 0u32)
	verify {
		// TODO: JA fix, pretty much benching this pathway requiers moving the session forward
		// This is diffcult, from the test we were able to mock it but benchamrks use runtime configs
		// It is fine for now but should come back to it
		// assert_last_event::<T>(Event::NodeInfoRemoved(caller).into());
	}

	validate {
		let caller: T::AccountId = whitelisted_caller();
		let bonder: T::AccountId = account("bond", 0, SEED);
		let threshold: T::AccountId = account("threshold", 0, SEED);

		prep_bond_and_validate::<T>(false, caller.clone(), bonder.clone(), threshold.clone());

		let validator_preferance = ValidatorPrefs::default();


	}:  _(RawOrigin::Signed(caller.clone()), validator_preferance, vec![20], threshold.clone())
	verify {
		assert_last_event::<T>(Event::NodeInfoChanged(caller,  vec![20], threshold).into());
	}



}

impl_benchmark_test_suite!(Staking, crate::mock::new_test_ext(), crate::mock::Test);
