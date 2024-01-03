// Copyright (C) 2023 Entropy Cryptography Inc.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use codec::Encode;
use entropy_shared::KeyVisibility;
use frame_support::{
    assert_noop, assert_ok,
    dispatch::{GetDispatchInfo, Pays},
    traits::Currency,
    BoundedVec,
};
use pallet_programs::ProgramInfo;
use pallet_relayer::Call as RelayerCall;
use sp_runtime::{
    traits::{Hash, SignedExtension},
    transaction_validity::{TransactionValidity, ValidTransaction},
};

use crate as pallet_relayer;
use crate::{
    mock::*, Error, Registered, RegisteredInfo, RegisteringDetails, ValidateConfirmRegistered,
};

#[test]
fn it_tests_get_validator_rotation() {
    new_test_ext().execute_with(|| {
        let result_1 = Relayer::get_validator_rotation(0, 0).unwrap();
        let result_2 = Relayer::get_validator_rotation(1, 0).unwrap();
        assert_eq!(result_1.0, 1);
        assert_eq!(result_2.0, 2);

        let result_3 = Relayer::get_validator_rotation(0, 1).unwrap();
        let result_4 = Relayer::get_validator_rotation(1, 1).unwrap();
        assert_eq!(result_3.0, 5);
        assert_eq!(result_4.0, 6);

        let result_5 = Relayer::get_validator_rotation(0, 100).unwrap();
        let result_6 = Relayer::get_validator_rotation(1, 100).unwrap();
        assert_eq!(result_5.0, 1);
        assert_eq!(result_6.0, 6);

        let result_7 = Relayer::get_validator_rotation(0, 101).unwrap();
        let result_8 = Relayer::get_validator_rotation(1, 101).unwrap();
        assert_eq!(result_7.0, 5);
        assert_eq!(result_8.0, 7);

        pallet_staking_extension::IsValidatorSynced::<Test>::insert(7, false);

        let result_9 = Relayer::get_validator_rotation(1, 101).unwrap();
        assert_eq!(result_9.0, 6);

        // really big number does not crash
        let result_10 = Relayer::get_validator_rotation(0, 1000000000000000000).unwrap();
        assert_eq!(result_10.0, 1);
    });
}

#[test]
fn it_registers_a_user() {
    new_test_ext().execute_with(|| {
        let empty_program = vec![];
        let program_hash = <Test as frame_system::Config>::Hashing::hash(&empty_program);
        pallet_programs::Programs::<Test>::insert(
            program_hash,
            ProgramInfo { bytecode: empty_program, program_modification_account: 1 },
        );

        assert_ok!(Relayer::register(
            RuntimeOrigin::signed(1),
            2 as <Test as frame_system::Config>::AccountId,
            KeyVisibility::Public,
            program_hash,
        ));
        assert_eq!(Relayer::dkg(0), vec![1u64.encode()]);
    });
}

#[test]
fn it_confirms_registers_a_user() {
    new_test_ext().execute_with(|| {
        let expected_verifying_key = BoundedVec::default();
        assert_noop!(
            Relayer::confirm_register(RuntimeOrigin::signed(1), 1, 0, BoundedVec::default()),
            Error::<Test>::NoThresholdKey
        );

        pallet_staking_extension::ThresholdToStash::<Test>::insert(1, 1);

        assert_noop!(
            Relayer::confirm_register(RuntimeOrigin::signed(1), 1, 0, BoundedVec::default()),
            Error::<Test>::NotRegistering
        );

        let empty_program = vec![];
        let program_hash = <Test as frame_system::Config>::Hashing::hash(&empty_program);
        pallet_programs::Programs::<Test>::insert(
            program_hash,
            ProgramInfo { bytecode: empty_program, program_modification_account: 1 },
        );

        assert_ok!(Relayer::register(
            RuntimeOrigin::signed(1),
            2 as <Test as frame_system::Config>::AccountId,
            KeyVisibility::Private([0; 32]),
            program_hash.clone(),
        ));

        assert_noop!(
            Relayer::confirm_register(RuntimeOrigin::signed(1), 1, 3, BoundedVec::default()),
            Error::<Test>::InvalidSubgroup
        );

        pallet_staking_extension::ThresholdToStash::<Test>::insert(2, 2);

        assert_noop!(
            Relayer::confirm_register(RuntimeOrigin::signed(2), 1, 0, BoundedVec::default()),
            Error::<Test>::NotInSigningGroup
        );

        assert_eq!(Relayer::registered(1), None);

        assert_ok!(Relayer::confirm_register(
            RuntimeOrigin::signed(1),
            1,
            0,
            expected_verifying_key.clone()
        ));

        assert_noop!(
            Relayer::confirm_register(
                RuntimeOrigin::signed(1),
                1,
                0,
                expected_verifying_key.clone()
            ),
            Error::<Test>::AlreadyConfirmed
        );

        let registering_info = RegisteringDetails::<Test> {
            confirmations: vec![0],
            program_pointer: program_hash.clone(),
            key_visibility: KeyVisibility::Private([0; 32]),
            verifying_key: Some(expected_verifying_key.clone()),
            program_modification_account: 2,
        };

        assert_eq!(Relayer::registering(1), Some(registering_info));

        assert_ok!(Relayer::confirm_register(
            RuntimeOrigin::signed(2),
            1,
            1,
            expected_verifying_key.clone()
        ));

        assert_eq!(Relayer::registering(1), None);
        assert_eq!(
            Relayer::registered(1).unwrap(),
            RegisteredInfo {
                key_visibility: KeyVisibility::Private([0; 32]),
                verifying_key: expected_verifying_key,
                program_pointer: program_hash,
                program_modification_account: 2
            }
        );
    });
}

#[test]
fn it_changes_a_program_pointer() {
    new_test_ext().execute_with(|| {
        let empty_program = vec![];
        let program_hash = <Test as frame_system::Config>::Hashing::hash(&empty_program);
        pallet_programs::Programs::<Test>::insert(
            program_hash,
            ProgramInfo { bytecode: empty_program, program_modification_account: 1 },
        );

        let new_program = vec![10];
        let new_program_hash = <Test as frame_system::Config>::Hashing::hash(&new_program);
        let expected_verifying_key = BoundedVec::default();

        let mut registered_info = RegisteredInfo {
            key_visibility: KeyVisibility::Public,
            verifying_key: expected_verifying_key,
            program_pointer: program_hash,
            program_modification_account: 2,
        };

        Registered::<Test>::insert(1, registered_info.clone());
        assert_eq!(Relayer::registered(1).unwrap(), registered_info.clone());

        assert_ok!(Relayer::change_program_pointer(RuntimeOrigin::signed(2), 1, new_program_hash,));
        registered_info.program_pointer = new_program_hash;
        assert_eq!(Relayer::registered(1).unwrap(), registered_info);
    });
}

#[test]
fn it_fails_on_non_matching_verifying_keys() {
    new_test_ext().execute_with(|| {
        let empty_program = vec![];
        let program_hash = <Test as frame_system::Config>::Hashing::hash(&empty_program);
        pallet_programs::Programs::<Test>::insert(
            program_hash,
            ProgramInfo { bytecode: empty_program, program_modification_account: 1 },
        );

        let expected_verifying_key = BoundedVec::default();
        let unexpected_verifying_key = vec![10];
        assert_ok!(Relayer::register(
            RuntimeOrigin::signed(1),
            2 as <Test as frame_system::Config>::AccountId,
            KeyVisibility::Private([0; 32]),
            program_hash,
        ));
        pallet_staking_extension::ThresholdToStash::<Test>::insert(1, 1);
        pallet_staking_extension::ThresholdToStash::<Test>::insert(2, 2);

        assert_ok!(Relayer::confirm_register(
            RuntimeOrigin::signed(1),
            1,
            0,
            expected_verifying_key
        ));

        // uses different verifying key
        assert_ok!(Relayer::confirm_register(
            RuntimeOrigin::signed(2),
            1,
            1,
            unexpected_verifying_key.try_into().unwrap()
        ));

        // not registered or registering
        assert_eq!(Relayer::registering(1), None);
        assert_eq!(Relayer::registered(1), None);
    })
}
#[test]
fn it_doesnt_allow_double_registering() {
    new_test_ext().execute_with(|| {
        // register a user
        let empty_program = vec![];
        let program_hash = <Test as frame_system::Config>::Hashing::hash(&empty_program);
        pallet_programs::Programs::<Test>::insert(
            program_hash,
            ProgramInfo { bytecode: empty_program, program_modification_account: 1 },
        );

        assert_ok!(Relayer::register(
            RuntimeOrigin::signed(1),
            2,
            KeyVisibility::Permissioned,
            program_hash.clone(),
        ));

        // error if they try to submit another request, even with a different program key
        assert_noop!(
            Relayer::register(
                RuntimeOrigin::signed(1),
                2,
                KeyVisibility::Permissioned,
                program_hash
            ),
            Error::<Test>::AlreadySubmitted
        );
    });
}

#[test]
fn it_fails_no_program() {
    new_test_ext().execute_with(|| {
        // register a user
        let non_existing_program = vec![10];
        let program_hash = <Test as frame_system::Config>::Hashing::hash(&non_existing_program);

        assert_noop!(
            Relayer::register(
                RuntimeOrigin::signed(1),
                2,
                KeyVisibility::Permissioned,
                program_hash
            ),
            Error::<Test>::ProgramDoesNotExist
        );
    });
}

#[test]
fn it_tests_prune_registration() {
    new_test_ext().execute_with(|| {
        let inital_program = vec![10];
        let program_hash = <Test as frame_system::Config>::Hashing::hash(&inital_program);
        pallet_programs::Programs::<Test>::insert(
            program_hash,
            ProgramInfo { bytecode: inital_program, program_modification_account: 1 },
        );

        Balances::make_free_balance_be(&2, 100);
        // register a user
        assert_ok!(Relayer::register(
            RuntimeOrigin::signed(1),
            2,
            KeyVisibility::Permissioned,
            program_hash,
        ));
        assert!(Relayer::registering(1).is_some(), "Make sure there is registering state");
        assert_ok!(Relayer::prune_registration(RuntimeOrigin::signed(1)));
        assert_eq!(Relayer::registering(1), None, "Make sure registering is pruned");
    });
}
#[test]
fn it_provides_free_txs_confirm_done() {
    new_test_ext().execute_with(|| {
        let empty_program = vec![];
        let program_hash = <Test as frame_system::Config>::Hashing::hash(&empty_program);
        pallet_programs::Programs::<Test>::insert(
            program_hash,
            ProgramInfo { bytecode: empty_program, program_modification_account: 1 },
        );

        let expected_verifying_key = BoundedVec::default();
        assert_ok!(Relayer::register(
            RuntimeOrigin::signed(5),
            2 as <Test as frame_system::Config>::AccountId,
            KeyVisibility::Public,
            program_hash,
        ));
        let p = ValidateConfirmRegistered::<Test>::new();
        let c = RuntimeCall::Relayer(RelayerCall::confirm_register {
            sig_req_account: 5,
            signing_subgroup: 0,
            verifying_key: expected_verifying_key,
        });
        let di = c.get_dispatch_info();
        assert_eq!(di.pays_fee, Pays::No);
        let r = p.validate(&7, &c, &di, 20);
        assert_eq!(r, TransactionValidity::Ok(ValidTransaction::default()));
    });
}

#[test]
#[should_panic = "TransactionValidityError::Invalid(InvalidTransaction::Custom(1)"]
fn it_provides_free_txs_confirm_done_fails_1() {
    new_test_ext().execute_with(|| {
        let expected_verifying_key = BoundedVec::default();
        let p = ValidateConfirmRegistered::<Test>::new();
        let c = RuntimeCall::Relayer(RelayerCall::confirm_register {
            sig_req_account: 5,
            signing_subgroup: 0,
            verifying_key: expected_verifying_key,
        });
        let di = c.get_dispatch_info();
        assert_eq!(di.pays_fee, Pays::No);
        let r = p.validate(&2, &c, &di, 20);
        assert_eq!(r, TransactionValidity::Ok(ValidTransaction::default()));
    });
}

#[test]
#[should_panic = "TransactionValidityError::Invalid(InvalidTransaction::Custom(2)"]
fn it_provides_free_txs_confirm_done_fails_2() {
    new_test_ext().execute_with(|| {
        let expected_verifying_key = BoundedVec::default();
        let p = ValidateConfirmRegistered::<Test>::new();
        let c = RuntimeCall::Relayer(RelayerCall::confirm_register {
            sig_req_account: 5,
            signing_subgroup: 0,
            verifying_key: expected_verifying_key,
        });
        let di = c.get_dispatch_info();
        assert_eq!(di.pays_fee, Pays::No);
        let r = p.validate(&7, &c, &di, 20);
        assert_eq!(r, TransactionValidity::Ok(ValidTransaction::default()));
    });
}

// TODO fails 3
#[test]
#[should_panic = "TransactionValidityError::Invalid(InvalidTransaction::Custom(3)"]
fn it_provides_free_txs_confirm_done_fails_3() {
    new_test_ext().execute_with(|| {
        let empty_program = vec![];
        let program_hash = <Test as frame_system::Config>::Hashing::hash(&empty_program);
        pallet_programs::Programs::<Test>::insert(
            program_hash,
            ProgramInfo { bytecode: empty_program, program_modification_account: 1 },
        );

        let expected_verifying_key = BoundedVec::default();
        assert_ok!(Relayer::register(
            RuntimeOrigin::signed(5),
            2 as <Test as frame_system::Config>::AccountId,
            KeyVisibility::Public,
            program_hash,
        ));

        assert_ok!(Relayer::confirm_register(
            RuntimeOrigin::signed(7),
            5,
            0,
            expected_verifying_key.clone()
        ));
        let p = ValidateConfirmRegistered::<Test>::new();
        let c = RuntimeCall::Relayer(RelayerCall::confirm_register {
            sig_req_account: 5,
            signing_subgroup: 0,
            verifying_key: expected_verifying_key,
        });
        let di = c.get_dispatch_info();
        assert_eq!(di.pays_fee, Pays::No);
        let r = p.validate(&7, &c, &di, 20);
        assert_eq!(r, TransactionValidity::Ok(ValidTransaction::default()));
    });
}

#[test]
#[should_panic = "TransactionValidityError::Invalid(InvalidTransaction::Custom(4)"]
fn it_provides_free_txs_confirm_done_fails_4() {
    new_test_ext().execute_with(|| {
        let empty_program = vec![];
        let program_hash = <Test as frame_system::Config>::Hashing::hash(&empty_program);
        pallet_programs::Programs::<Test>::insert(
            program_hash,
            ProgramInfo { bytecode: empty_program, program_modification_account: 1 },
        );

        let expected_verifying_key = BoundedVec::default();
        assert_ok!(Relayer::register(
            RuntimeOrigin::signed(5),
            2 as <Test as frame_system::Config>::AccountId,
            KeyVisibility::Public,
            program_hash,
        ));
        let p = ValidateConfirmRegistered::<Test>::new();
        let c = RuntimeCall::Relayer(RelayerCall::confirm_register {
            sig_req_account: 5,
            signing_subgroup: 5,
            verifying_key: expected_verifying_key,
        });
        let di = c.get_dispatch_info();
        assert_eq!(di.pays_fee, Pays::No);
        let r = p.validate(&7, &c, &di, 20);
        assert_eq!(r, TransactionValidity::Ok(ValidTransaction::default()));
    });
}

#[test]
#[should_panic = "TransactionValidityError::Invalid(InvalidTransaction::Custom(5)"]
fn it_provides_free_txs_confirm_done_fails_5() {
    new_test_ext().execute_with(|| {
        let empty_program = vec![];
        let program_hash = <Test as frame_system::Config>::Hashing::hash(&empty_program);
        pallet_programs::Programs::<Test>::insert(
            program_hash,
            ProgramInfo { bytecode: empty_program, program_modification_account: 1 },
        );

        let expected_verifying_key = BoundedVec::default();
        assert_ok!(Relayer::register(
            RuntimeOrigin::signed(5),
            2 as <Test as frame_system::Config>::AccountId,
            KeyVisibility::Public,
            program_hash,
        ));
        let p = ValidateConfirmRegistered::<Test>::new();
        let c = RuntimeCall::Relayer(RelayerCall::confirm_register {
            sig_req_account: 5,
            signing_subgroup: 0,
            verifying_key: expected_verifying_key,
        });
        let di = c.get_dispatch_info();
        assert_eq!(di.pays_fee, Pays::No);
        let r = p.validate(&4, &c, &di, 20);
        assert_eq!(r, TransactionValidity::Ok(ValidTransaction::default()));
    });
}
