
use crate::{Error,mock::*};
use frame_support::{assert_ok, assert_noop};
use frame_system::RawOrigin;

#[test]
fn it_works_for_first_organization() {
	new_test_ext().execute_with(|| {
		let c = IdavollModule::counter_of();
		let org_id = create_new_organization(OWNER.clone(),100);
		assert_ne!(org_id,u128::MAX);
		assert_ok!(IdavollModule::deposit_to_origanization(RawOrigin::Signed(A).into(),
		c,200));
		// assert_eq!(IdavollModule::get_count_members(org_id),1);
		// vote to transfer the vault

		//
		// assert_noop!(IdavollModule::on_add_member(1,2,0),Error::<Test>::MemberDuplicate);
		// assert_ok!(IdavollModule::on_add_member(1,22,0));
		// assert_eq!(IdavollModule::get_count_members(org_id),5);
		// assert_ok!(IdavollModule::on_add_member(OWNER,23,0));
		// assert_eq!(IdavollModule::get_count_members(org_id),6);
	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		// assert_noop!(
		// 	IdavollModule::cause_error(Origin::signed(1)),
		// 	Error::<Test>::NoneValue
		// );
	});
}
