use crate as pallet_relayer;
use crate::{mock::*, Error, Failures, Message, PrevalidateRelayer, Responsibility, SigRequest};
use frame_support::{
	assert_noop, assert_ok,
	weights::{GetDispatchInfo, Pays},
};
use pallet_relayer::Call as RelayerCall;
use sp_runtime::{
	traits::SignedExtension,
	transaction_validity::{TransactionValidity, ValidTransaction},
};

#[test]
fn it_preps_transaction() {
	new_test_ext().execute_with(|| {
		let sig_request = SigRequest { sig_id: 1u16, nonce: 1u32, signature: 1u32 };
		let message =
			Message { account: vec![1, 0, 0, 0, 0, 0, 0, 0], sig_request: sig_request.clone() };

		assert_ok!(Relayer::prep_transaction(Origin::signed(1), sig_request.clone()));

		assert_eq!(Relayer::messages(0), vec![message]);
	});
}

#[test]
fn it_registers_a_user() {
	new_test_ext().execute_with(|| {
		assert_ok!(Relayer::register(Origin::signed(1)));

		assert!(Relayer::registered(1));
	});
}

#[test]
fn it_confirms_done() {
	new_test_ext().execute_with(|| {
		Responsibility::<Test>::insert(5, 2);
		let failures = vec![0u32, 3u32];
		pallet_staking_extension::ThresholdAccounts::<Test>::insert(2, 1);

		assert_ok!(Relayer::confirm_done(Origin::signed(1), 5, failures.clone()));
		assert_eq!(Relayer::failures(5), Some(failures.clone()));

		assert_noop!(
			Relayer::confirm_done(Origin::signed(1), 5, failures.clone()),
			Error::<Test>::AlreadySubmitted
		);
		assert_noop!(
			Relayer::confirm_done(Origin::signed(1), 6, failures.clone()),
			Error::<Test>::NoResponsibility
		);
		Responsibility::<Test>::insert(6, 3);
		assert_noop!(
			Relayer::confirm_done(Origin::signed(2), 6, failures.clone()),
			Error::<Test>::NoThresholdKey
		);
		pallet_staking_extension::ThresholdAccounts::<Test>::insert(2, 5);
		assert_noop!(
			Relayer::confirm_done(Origin::signed(2), 5, failures.clone()),
			Error::<Test>::NotYourResponsibility
		);
	});
}

#[test]
fn moves_active_to_pending() {
	new_test_ext().execute_with(|| {
		// no failures pings unresponsive
		System::set_block_number(3);
		Responsibility::<Test>::insert(3, 1);
		Relayer::move_active_to_pending(5);
		assert_eq!(Relayer::unresponsive(1), 1);
		let failures = vec![0u32, 3u32];
		Failures::<Test>::insert(2, failures.clone());
		Failures::<Test>::insert(5, failures.clone());

		let sig_request = SigRequest { sig_id: 1u16, nonce: 1u32, signature: 1u32 };
		let message =
			Message { account: vec![1, 0, 0, 0, 0, 0, 0, 0], sig_request: sig_request.clone() };

		assert_ok!(Relayer::prep_transaction(Origin::signed(1), sig_request.clone()));
		assert_eq!(Relayer::messages(3), vec![message.clone()]);

		// prunes old failure remove messages put into pending
		assert_eq!(Relayer::failures(2), Some(failures.clone()));
		Relayer::move_active_to_pending(5);
		assert_eq!(Relayer::failures(2), None);
		assert_eq!(Relayer::messages(3), vec![]);
		assert_eq!(Relayer::pending(3), vec![message.clone()]);
		assert_eq!(Relayer::unresponsive(1), 0);
		// pending pruned
		Responsibility::<Test>::insert(4, 1);
		Relayer::move_active_to_pending(6);
		assert_eq!(Relayer::pending(3), vec![]);
	});
}

#[test]
fn notes_responsibility() {
	new_test_ext().execute_with(|| {
		Responsibility::<Test>::insert(2, 1);
		Relayer::note_responsibility(5);
		assert_eq!(Relayer::responsibility(4), Some(11));
		assert_eq!(Relayer::responsibility(2), None);
	});
}

#[test]
fn it_provides_free_txs_prep_tx() {
	new_test_ext().execute_with(|| {
		assert_ok!(Relayer::register(Origin::signed(1)));

		let p = PrevalidateRelayer::<Test>::new();
		let sig_request = SigRequest { sig_id: 1u16, nonce: 1u32, signature: 1u32 };

		let c = Call::Relayer(RelayerCall::prep_transaction { sig_request });
		let di = c.get_dispatch_info();
		assert_eq!(di.pays_fee, Pays::No);
		let r = p.validate(&1, &c, &di, 20);
		assert_eq!(r, TransactionValidity::Ok(ValidTransaction::default()));
	});
}

#[test]
fn it_fails_a_free_tx_prep_tx() {
	new_test_ext().execute_with(|| {
		let p = PrevalidateRelayer::<Test>::new();
		let sig_request = SigRequest { sig_id: 1u16, nonce: 1u32, signature: 1u32 };

		let c = Call::Relayer(RelayerCall::prep_transaction { sig_request });
		let di = c.get_dispatch_info();
		let r = p.validate(&42, &c, &di, 20);
		assert!(r.is_err());
	});
}

#[test]
fn it_provides_free_txs_confirm_done() {
	new_test_ext().execute_with(|| {
		Responsibility::<Test>::insert(5, 1);
		pallet_staking_extension::ThresholdAccounts::<Test>::insert(1, 2);
		let p = PrevalidateRelayer::<Test>::new();
		let c = Call::Relayer(RelayerCall::confirm_done { block_number: 5, failures: vec![] });
		let di = c.get_dispatch_info();
		assert_eq!(di.pays_fee, Pays::No);
		let r = p.validate(&2, &c, &di, 20);
		assert_eq!(r, TransactionValidity::Ok(ValidTransaction::default()));
	});
}

#[test]
#[should_panic = "TransactionValidityError::Invalid(InvalidTransaction::Custom(1)"]
fn it_fails_a_free_tx_confirm_done_err_1() {
	new_test_ext().execute_with(|| {
		let sig_request = SigRequest { sig_id: 1u16, nonce: 1u32, signature: 1u32 };

		let p = PrevalidateRelayer::<Test>::new();
		let c = Call::Relayer(RelayerCall::prep_transaction { sig_request });
		let di = c.get_dispatch_info();
		let r = p.validate(&1, &c, &di, 20);
		r.unwrap()
	});
}

#[test]
#[should_panic = "TransactionValidityError::Invalid(InvalidTransaction::Custom(2)"]
fn it_fails_a_free_tx_confirm_done_err_2() {
	new_test_ext().execute_with(|| {
		let p = PrevalidateRelayer::<Test>::new();
		let c = Call::Relayer(RelayerCall::confirm_done { block_number: 5, failures: vec![] });
		let di = c.get_dispatch_info();
		let r = p.validate(&1, &c, &di, 20);
		r.unwrap()
	});
}

#[test]
#[should_panic = "TransactionValidityError::Invalid(InvalidTransaction::Custom(3)"]
fn it_fails_a_free_tx_confirm_done_err_3() {
	new_test_ext().execute_with(|| {
		Responsibility::<Test>::insert(5, 1);
		let p = PrevalidateRelayer::<Test>::new();
		let c = Call::Relayer(RelayerCall::confirm_done { block_number: 5, failures: vec![] });
		let di = c.get_dispatch_info();
		let r = p.validate(&42, &c, &di, 20);
		r.unwrap()
	});
}

#[test]
#[should_panic = "TransactionValidityError::Invalid(InvalidTransaction::Custom(4)"]
fn it_fails_a_free_tx_confirm_done_err_4() {
	new_test_ext().execute_with(|| {
		Responsibility::<Test>::insert(5, 1);
		pallet_staking_extension::ThresholdAccounts::<Test>::insert(1, 2);
		Failures::<Test>::insert(5, vec![1]);
		let p = PrevalidateRelayer::<Test>::new();
		let c = Call::Relayer(RelayerCall::confirm_done { block_number: 5, failures: vec![] });
		let di = c.get_dispatch_info();
		let r = p.validate(&1, &c, &di, 20);
		r.unwrap()
	});
}

#[test]
#[should_panic = "TransactionValidityError::Invalid(InvalidTransaction::Custom(5)"]
fn it_fails_a_free_tx_confirm_done_err_5() {
	new_test_ext().execute_with(|| {
		Responsibility::<Test>::insert(5, 1);
		pallet_staking_extension::ThresholdAccounts::<Test>::insert(1, 2);
		Failures::<Test>::insert(5, vec![1]);
		let p = PrevalidateRelayer::<Test>::new();
		let c = Call::Relayer(RelayerCall::confirm_done { block_number: 5, failures: vec![] });
		let di = c.get_dispatch_info();
		let r = p.validate(&2, &c, &di, 20);
		r.unwrap()
	});
}

#[test]
fn it_provides_free_txs_register() {
	new_test_ext().execute_with(|| {
		let p = PrevalidateRelayer::<Test>::new();
		let c = Call::Relayer(RelayerCall::register {});
		let di = c.get_dispatch_info();
		assert_eq!(di.pays_fee, Pays::No);
		let r = p.validate(&1, &c, &di, 20);
		assert_eq!(r, TransactionValidity::Ok(ValidTransaction::default()));
	});
}
