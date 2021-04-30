
use crate::{Error, mock::*, Trait};
use frame_support::{assert_ok, assert_noop};
use frame_system::RawOrigin;

#[test]
fn it_works_for_first_organization() {
	new_test_ext().execute_with(|| {
		let c = IdavollModule::counter_of();
		let org_id = create_new_organization(OWNER.clone(),100);
		assert_ne!(org_id,u128::MAX);
		assert_ok!(IdavollModule::deposit_to_origanization(RawOrigin::Signed(A).into(),c,200));
		assert_eq!(IdavollModule::get_count_members(org_id),1);

		// vote to transfer the vault
		// make the proposal with the proposal id
		// let call = make_transfer_proposal(10);
		// let mut tmp_proposal = create_proposal(org_id,IdavollModule::call_to_vec(call.clone()));
		// let proposal_id = IdavollModule::make_proposal_id(&tmp_proposal.clone());
		//
		// assert_ok!(IdavollModule::create_proposal(RawOrigin::Signed(OWNER.clone()),
		// c,tmp_proposal.detail.end_dt,tmp_proposal.detail.sub_param.clone(),call));
		// assert_eq!(IdavollModule::get_proposal_by_id(proposal_id.clone()),Ok(tmp_proposal.clone()));

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
