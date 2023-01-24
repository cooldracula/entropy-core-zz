use frame_support::{assert_noop, assert_ok};
use sp_core::{H160, H256};

use crate::{mock::*, Acl, AllowedToModifyConstraints, Constraints, Error};

/// consts used for testing
const CONSTRAINT_ACCOUNT: u64 = 1u64;
const SIG_REQ_ACCOUNT: u64 = 2u64;

// Integration Test
#[test]
fn assert_permissions_are_restricted_properly() {
    new_test_ext().execute_with(|| {
        // In practice, we should use `None` instead of `Some(Acl::default())`,
        // but this is fine to test permission
        let valid_constraints = Constraints {
            evm_acl: Some(Acl::<H160>::default()),
            btc_acl: Some(Acl::<H256>::default()),
        };

        // make sure noone can add a constraint without explicit permissions
        assert_noop!(
            ConstraintsPallet::update_constraints(
                RuntimeOrigin::signed(CONSTRAINT_ACCOUNT),
                SIG_REQ_ACCOUNT,
                valid_constraints.clone(),
            ),
            Error::<Test>::NotAuthorized
        );

        // give permission to modify constraints and make sure the acl can be updated
        AllowedToModifyConstraints::<Test>::insert(&CONSTRAINT_ACCOUNT, &SIG_REQ_ACCOUNT, ());
        assert_ok!(ConstraintsPallet::update_constraints(
            RuntimeOrigin::signed(CONSTRAINT_ACCOUNT),
            SIG_REQ_ACCOUNT,
            valid_constraints.clone()
        ));
        assert!(ConstraintsPallet::evm_acl(SIG_REQ_ACCOUNT).is_ok());

        // make sure sig-req key can't modify or delete constraints
        assert_noop!(
            ConstraintsPallet::update_constraints(
                RuntimeOrigin::signed(SIG_REQ_ACCOUNT),
                SIG_REQ_ACCOUNT,
                valid_constraints.clone(),
            ),
            Error::<Test>::NotAuthorized
        );
        assert_noop!(
            ConstraintsPallet::update_constraints(
                RuntimeOrigin::signed(SIG_REQ_ACCOUNT),
                SIG_REQ_ACCOUNT,
                valid_constraints.clone(),
            ),
            Error::<Test>::NotAuthorized
        );

        // removing permissions should prevent modification
        AllowedToModifyConstraints::<Test>::remove(&CONSTRAINT_ACCOUNT, &SIG_REQ_ACCOUNT);
        assert_noop!(
            ConstraintsPallet::update_constraints(
                RuntimeOrigin::signed(CONSTRAINT_ACCOUNT),
                SIG_REQ_ACCOUNT,
                valid_constraints
            ),
            Error::<Test>::NotAuthorized
        );
    });
}

#[test]
fn return_error_if_constraints_arent_set() {
    new_test_ext().execute_with(|| {
        // In practice, we should use `None` instead of `Some(Acl::default())`,
        // but this is fine to test permission
        let valid_constraints = Constraints {
            evm_acl: Some(Acl::<H160>::default()),
            btc_acl: Some(Acl::<H256>::default()),
        };

        // give permission to modify constraints
        AllowedToModifyConstraints::<Test>::insert(&CONSTRAINT_ACCOUNT, &SIG_REQ_ACCOUNT, ());

        // make sure acl is empty
        assert!(ConstraintsPallet::evm_acl(SIG_REQ_ACCOUNT).is_err());

        // make sure we can update the ACL
        assert_ok!(ConstraintsPallet::update_constraints(
            RuntimeOrigin::signed(CONSTRAINT_ACCOUNT),
            SIG_REQ_ACCOUNT,
            valid_constraints
        ));

        // make sure acl updates
        assert_eq!(ConstraintsPallet::evm_acl(SIG_REQ_ACCOUNT).unwrap(), Acl::<H160>::default());
    });
}
