
use crate::{Error, mock::*, Trait};
use frame_support::{assert_ok, assert_noop};
use frame_system::RawOrigin;

#[test]
fn it_works_for_create_organization() {
	new_test_ext().execute_with(|| {

		for i in 1..10 {
			let c = IdavollModule::counter_of();
			let org_id = create_new_organization(i,100*i as u64);
			assert_ne!(org_id,u128::MAX);
			let asset_id = IdavollModule::get_token_id_by_oid(org_id.clone()).unwrap();
			assert_eq!(IdavollModule::get_total_token_by_oid(org_id.clone()),Ok(100*i as u64));
			assert_ok!(IdavollModule::deposit_to_origanization(RawOrigin::Signed(A).into(),c,10+i as u64));
			assert_eq!(IdavollModule::get_count_members(org_id),1);
			assert_eq!(IdavollAsset::vault_balance_of(org_id.clone()),Ok(10+i as u64));
			assert_eq!(IdavollAsset::free_balance(asset_id.clone(),&i),100*i as u64);
			assert_eq!(IdavollModule::get_local_balance(org_id.clone()),Ok(10+i as u64));
		}
	});
}

#[test]
fn it_works_for_add_member() {
	new_test_ext().execute_with(|| {
		let c = IdavollModule::counter_of();
		let org_id = create_new_organization(OWNER.clone(),100);
		assert_ne!(org_id,u128::MAX);
		let asset_id = IdavollModule::get_token_id_by_oid(org_id.clone()).unwrap();
		assert_eq!(IdavollModule::get_total_token_by_oid(org_id.clone()),Ok(100));
		assert_ok!(IdavollModule::deposit_to_origanization(RawOrigin::Signed(A).into(),c,10));
		assert_eq!(IdavollModule::get_count_members(org_id),1);
		assert_eq!(IdavollAsset::vault_balance_of(org_id.clone()),Ok(10));
		assert_eq!(IdavollAsset::free_balance(asset_id.clone(),&OWNER.clone()),100);
		assert_eq!(IdavollModule::get_local_balance(org_id.clone()),Ok(10));

		// add members in the organization by org_id
		// add member '1' and '2' account by the owner of the organization
		assert_ok!(IdavollModule::add_member_by_onwer(RawOrigin::Signed(OWNER.clone()).into(),1,c));
		assert_eq!(IdavollModule::get_count_members(org_id),2);
		assert_ok!(IdavollModule::add_member_by_onwer(RawOrigin::Signed(OWNER.clone()).into(),2,c));
		assert_eq!(IdavollModule::get_count_members(org_id),3);
		// add member '3' and '4' account by the '1' account
		assert_ok!(IdavollModule::add_member_by_onwer(RawOrigin::Signed(1).into(),3,c));
		assert_eq!(IdavollModule::get_count_members(org_id),4);
		assert_ok!(IdavollModule::add_member_by_onwer(RawOrigin::Signed(1).into(),4,c));
		assert_eq!(IdavollModule::get_count_members(org_id),5);

		// transfer the token for vote from the owner
		// the total issuance was 100
		assert_eq!(IdavollAsset::total_issuances(asset_id.clone()),100);
		assert_eq!(IdavollAsset::free_balance(asset_id.clone(),&OWNER.clone()),100);
		// transfer 10 token to the members
		assert_ok!(IdavollAsset::transfer(RawOrigin::Signed(OWNER.clone()).into(),asset_id.clone(),1,10));
		assert_ok!(IdavollAsset::transfer(RawOrigin::Signed(OWNER.clone()).into(),asset_id.clone(),2,10));
		assert_ok!(IdavollAsset::transfer(RawOrigin::Signed(OWNER.clone()).into(),asset_id.clone(),3,10));
		assert_ok!(IdavollAsset::transfer(RawOrigin::Signed(OWNER.clone()).into(),asset_id.clone(),4,10));

		assert_eq!(IdavollAsset::free_balance(asset_id.clone(),&OWNER.clone()),60);
		assert_eq!(IdavollAsset::free_balance(asset_id.clone(),&1),10);
		assert_eq!(IdavollAsset::free_balance(asset_id.clone(),&2),10);
		assert_eq!(IdavollAsset::free_balance(asset_id.clone(),&3),10);
		assert_eq!(IdavollAsset::free_balance(asset_id.clone(),&4),10);
	});
}

#[test]
fn it_works_for_first_dao() {
	new_test_ext().execute_with(|| {
		let c = IdavollModule::counter_of();
		let org_id = create_new_organization(OWNER.clone(),100);
		assert_ne!(org_id,u128::MAX);
		assert_ok!(IdavollModule::deposit_to_origanization(RawOrigin::Signed(A).into(),c,200));
		assert_eq!(IdavollModule::get_count_members(org_id),1);
		assert_eq!(IdavollAsset::vault_balance_of(org_id.clone()),Ok(200 as u64));

		set_block_number(1);
		assert_eq!(get_block_number(),1 as u64);
		// vote to transfer the vault
		// make the proposal with the proposal id
		let call = make_transfer_proposal(10);
		let tmp_proposal = create_proposal_without_storage(org_id,5,call_to_vec(call.clone()));
		let proposal_id = IdavollModule::make_proposal_id(&tmp_proposal.clone());

		assert_ok!(IdavollModule::create_proposal(RawOrigin::Signed(OWNER.clone()).into(),c,
		5,tmp_proposal.detail.sub_param.clone(),call));
		// get the proposal from the storage
		assert_eq!(IdavollModule::get_proposal_by_id(proposal_id.clone()),Ok(tmp_proposal.clone()));

		assert_eq!(IdvBalances::free_balance(RECEIVER.clone()),0 as u64);
		// vote for the proposal by the same proposal_id
		assert_ok!(IdavollModule::vote_proposal(RawOrigin::Signed(OWNER.clone()).into(),proposal_id.clone(),80,true));
		// make sure the RECEIVER has the 'value' balance and the vault was reduce 'value' balance
		assert_eq!(IdvBalances::free_balance(RECEIVER.clone()),10 as u64);
		assert_eq!(IdavollAsset::vault_balance_of(org_id.clone()),Ok(190 as u64));
	});
}

#[test]
fn it_works_for_5_members_vote_pass() {
	new_test_ext().execute_with(|| {
		let c = IdavollModule::counter_of();
		let org_id = create_new_organization(OWNER.clone(),100);
		let asset_id = IdavollModule::get_token_id_by_oid(org_id.clone()).unwrap();
		assert_ne!(org_id,u128::MAX);
		assert_ok!(IdavollModule::deposit_to_origanization(RawOrigin::Signed(A).into(),c,200));
		assert_eq!(IdavollModule::get_count_members(org_id),1);
		assert_eq!(IdavollAsset::vault_balance_of(org_id),Ok(200 as u64));
		// add 4 members
		assert_ok!(IdavollModule::add_member_by_onwer(RawOrigin::Signed(OWNER.clone()).into(),1,c));
		assert_ok!(IdavollModule::add_member_by_onwer(RawOrigin::Signed(OWNER.clone()).into(),2,c));
		assert_ok!(IdavollModule::add_member_by_onwer(RawOrigin::Signed(OWNER.clone()).into(),3,c));
		assert_ok!(IdavollModule::add_member_by_onwer(RawOrigin::Signed(OWNER.clone()).into(),4,c));
		assert_eq!(IdavollModule::get_count_members(org_id),5);
		// assign the tokens for voting
		assert_ok!(IdavollAsset::transfer(RawOrigin::Signed(OWNER.clone()).into(),asset_id.clone(),1,5));
		assert_ok!(IdavollAsset::transfer(RawOrigin::Signed(OWNER.clone()).into(),asset_id.clone(),2,10));
		assert_ok!(IdavollAsset::transfer(RawOrigin::Signed(OWNER.clone()).into(),asset_id.clone(),3,20));
		assert_ok!(IdavollAsset::transfer(RawOrigin::Signed(OWNER.clone()).into(),asset_id.clone(),4,25));
		assert_eq!(IdavollAsset::free_balance(asset_id,&OWNER.clone()),40);

		set_block_number(1);
		assert_eq!(get_block_number(),1 as u64);
		// vote to transfer the vault
		// make the proposal with the proposal id
		let call = make_transfer_proposal(30);
		let tmp_proposal = create_proposal_without_storage(org_id,5,call_to_vec(call.clone()));
		let proposal_id = IdavollModule::make_proposal_id(&tmp_proposal.clone());

		assert_ok!(IdavollModule::create_proposal(RawOrigin::Signed(OWNER.clone()).into(),c,
		5,tmp_proposal.detail.sub_param.clone(),call));
		// get the proposal from the storage
		assert_eq!(IdavollModule::get_proposal_by_id(proposal_id.clone()),Ok(tmp_proposal.clone()));

		// vote for the proposal
		assert_eq!(IdvBalances::free_balance(RECEIVER.clone()),0 as u64);
		// vote for the proposal by the same proposal_id
		assert_ok!(IdavollModule::vote_proposal(RawOrigin::Signed(OWNER.clone()).into(),proposal_id.clone(),20,true));
		assert_ok!(IdavollModule::vote_proposal(RawOrigin::Signed(1).into(),proposal_id.clone(),3,true));
		assert_ok!(IdavollModule::vote_proposal(RawOrigin::Signed(2).into(),proposal_id.clone(),8,true));
		assert_ok!(IdavollModule::vote_proposal(RawOrigin::Signed(3).into(),proposal_id.clone(),20,true));
		assert_ok!(IdavollModule::vote_proposal(RawOrigin::Signed(4).into(),proposal_id.clone(),20,true));

		// the vote result, the proposal was passed，60% 'yes' votes was passed
		// make sure the RECEIVER has the 'value' balance and the vault was reduce 'value' balance
		assert_eq!(IdvBalances::free_balance(RECEIVER.clone()),30);
		assert_eq!(IdavollAsset::vault_balance_of(org_id.clone()),Ok(170));
	});
}

#[test]
fn it_works_for_5_members_vote_fail() {
	new_test_ext().execute_with(|| {
		let c = IdavollModule::counter_of();
		let org_id = create_new_organization(OWNER.clone(),100);
		let asset_id = IdavollModule::get_token_id_by_oid(org_id.clone()).unwrap();
		assert_ne!(org_id,u128::MAX);
		assert_ok!(IdavollModule::deposit_to_origanization(RawOrigin::Signed(A).into(),c,200));
		assert_eq!(IdavollModule::get_count_members(org_id),1);
		assert_eq!(IdavollAsset::vault_balance_of(org_id),Ok(200 as u64));
		// add 4 members
		assert_ok!(IdavollModule::add_member_by_onwer(RawOrigin::Signed(OWNER.clone()).into(),1,c));
		assert_ok!(IdavollModule::add_member_by_onwer(RawOrigin::Signed(OWNER.clone()).into(),2,c));
		assert_ok!(IdavollModule::add_member_by_onwer(RawOrigin::Signed(OWNER.clone()).into(),3,c));
		assert_ok!(IdavollModule::add_member_by_onwer(RawOrigin::Signed(OWNER.clone()).into(),4,c));
		assert_eq!(IdavollModule::get_count_members(org_id),5);
		// assign the tokens for voting
		assert_ok!(IdavollAsset::transfer(RawOrigin::Signed(OWNER.clone()).into(),asset_id.clone(),1,5));
		assert_ok!(IdavollAsset::transfer(RawOrigin::Signed(OWNER.clone()).into(),asset_id.clone(),2,10));
		assert_ok!(IdavollAsset::transfer(RawOrigin::Signed(OWNER.clone()).into(),asset_id.clone(),3,20));
		assert_ok!(IdavollAsset::transfer(RawOrigin::Signed(OWNER.clone()).into(),asset_id.clone(),4,25));
		assert_eq!(IdavollAsset::free_balance(asset_id,&OWNER.clone()),40);

		set_block_number(1);
		assert_eq!(get_block_number(),1 as u64);
		// vote to transfer the vault
		// make the proposal with the proposal id
		let call = make_transfer_proposal(30);
		let tmp_proposal = create_proposal_without_storage(org_id,5,call_to_vec(call.clone()));
		let proposal_id = IdavollModule::make_proposal_id(&tmp_proposal.clone());

		assert_ok!(IdavollModule::create_proposal(RawOrigin::Signed(OWNER.clone()).into(),c,
		5,tmp_proposal.detail.sub_param.clone(),call));
		// get the proposal from the storage
		assert_eq!(IdavollModule::get_proposal_by_id(proposal_id.clone()),Ok(tmp_proposal.clone()));

		// vote for the proposal
		assert_eq!(IdvBalances::free_balance(RECEIVER.clone()),0 as u64);
		// vote for the proposal by the same proposal_id
		assert_ok!(IdavollModule::vote_proposal(RawOrigin::Signed(OWNER.clone()).into(),proposal_id.clone(),20,true));
		assert_ok!(IdavollModule::vote_proposal(RawOrigin::Signed(1).into(),proposal_id.clone(),3,true));
		assert_ok!(IdavollModule::vote_proposal(RawOrigin::Signed(2).into(),proposal_id.clone(),8,false));
		assert_ok!(IdavollModule::vote_proposal(RawOrigin::Signed(3).into(),proposal_id.clone(),20,true));
		assert_ok!(IdavollModule::vote_proposal(RawOrigin::Signed(4).into(),proposal_id.clone(),20,true));

		// the vote result, the proposal was passed，60% 'yes' votes was passed
		// make sure the RECEIVER has the 'value' balance and the vault was reduce 'value' balance
		assert_eq!(IdvBalances::free_balance(RECEIVER.clone()),0);
		assert_eq!(IdavollAsset::vault_balance_of(org_id.clone()),Ok(200));
	});
}