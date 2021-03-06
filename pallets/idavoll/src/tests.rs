/*
 * Copyright 2021 Idavoll Network
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */


use crate::{Error,mock::*};
use frame_support::{assert_ok,assert_noop};
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
			assert_ok!(IdavollModule::deposit_to_organization(RawOrigin::Signed(A).into(),c,10+i as u64));
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
		assert_ok!(IdavollModule::deposit_to_organization(RawOrigin::Signed(A).into(),c,10));
		assert_eq!(IdavollModule::get_count_members(org_id),1);
		assert_eq!(IdavollAsset::vault_balance_of(org_id.clone()),Ok(10));
		assert_eq!(IdavollAsset::free_balance(asset_id.clone(),&OWNER.clone()),100);
		assert_eq!(IdavollModule::get_local_balance(org_id.clone()),Ok(10));

		// the total issuance was 100
		assert_eq!(IdavollAsset::total_issuances(asset_id.clone()),100);
		assert_eq!(IdavollAsset::free_balance(asset_id.clone(),&OWNER.clone()),100);

		// add members in the organization by org_id and transfer the token for vote from the owner
		// add member '1' and '2' account by the owner of the organization
		assert_ok!(IdavollModule::add_member_and_assign_token(RawOrigin::Signed(OWNER.clone()).into(),1,c,10));
		assert_eq!(IdavollAsset::free_balance(asset_id.clone(),&OWNER.clone()),90);
		assert_eq!(IdavollAsset::free_balance(asset_id.clone(),&1),10);
		assert_eq!(IdavollModule::get_count_members(org_id),2);

		assert_ok!(IdavollModule::add_member_and_assign_token(RawOrigin::Signed(OWNER.clone()).into(),2,c,10));
		assert_eq!(IdavollAsset::free_balance(asset_id.clone(),&OWNER.clone()),80);
		assert_eq!(IdavollAsset::free_balance(asset_id.clone(),&2),10);
		assert_eq!(IdavollModule::get_count_members(org_id),3);

		// add member '3' and '4' account by the '1' account
		assert_ok!(IdavollModule::add_member_and_assign_token(RawOrigin::Signed(1).into(),3,c,3));
		assert_eq!(IdavollAsset::free_balance(asset_id.clone(),&1),7);
		assert_eq!(IdavollAsset::free_balance(asset_id.clone(),&3),3);
		assert_eq!(IdavollModule::get_count_members(org_id),4);

		assert_ok!(IdavollModule::add_member_and_assign_token(RawOrigin::Signed(1).into(),4,c,2));
		assert_eq!(IdavollAsset::free_balance(asset_id.clone(),&1),5);
		assert_eq!(IdavollAsset::free_balance(asset_id.clone(),&4),2);
		assert_eq!(IdavollModule::get_count_members(org_id),5);
	});
}

#[test]
fn only_members_can_create_proposals() {
	new_test_ext().execute_with(|| {
		let c = IdavollModule::counter_of();
		let org_id = create_new_organization(OWNER.clone(),100);
		assert_ne!(org_id,u128::MAX);
		assert_ok!(IdavollModule::deposit_to_organization(RawOrigin::Signed(A).into(),c,200));
		assert_eq!(IdavollModule::get_count_members(org_id),1);
		assert_eq!(IdavollAsset::vault_balance_of(org_id.clone()),Ok(200 as u64));

		set_block_number(1);
		assert_eq!(get_block_number(),1 as u64);
		// vote to transfer the vault
		// make the proposal with the proposal id
		let call = make_transfer_proposal(10);
		let tmp_proposal = create_proposal_without_storage(org_id,5,call_to_vec(call.clone()));
		// let proposal_id = IdavollModule::make_proposal_id(&tmp_proposal.clone());

		assert_noop!(IdavollModule::create_proposal(RawOrigin::Signed(A.clone()).into(),c,
		5,tmp_proposal.detail.sub_param.clone(),call),Error::<Test>::NotMemberInOrg);
	});
}

#[test]
fn it_works_for_first_dao() {
	new_test_ext().execute_with(|| {
		let c = IdavollModule::counter_of();
		let org_id = create_new_organization(OWNER.clone(),100);
		assert_ne!(org_id,u128::MAX);
		assert_ok!(IdavollModule::deposit_to_organization(RawOrigin::Signed(A).into(),c,200));
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
fn it_works_for_dao_of_token_balance_change() {
	new_test_ext().execute_with(|| {
		let c = IdavollModule::counter_of();
		let org_id = create_new_organization(OWNER.clone(),100);
		assert_ne!(org_id,u128::MAX);
		assert_ok!(IdavollModule::deposit_to_organization(RawOrigin::Signed(A).into(),c,200));
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

		// vote for the proposal by the same proposal_id
		// voting on the proposal and check the token balance of the asset_id was changed
		// owner has all tokens of the asset_id by create the organization
		let asset_id = IdavollModule::get_token_id_by_oid(org_id.clone()).unwrap();
		assert_eq!(IdavollAsset::free_balance(asset_id,&OWNER.clone()),100);
		// the vault was 0
		assert_eq!(IdvBalances::free_balance(RECEIVER.clone()),0 as u64);

		// the owner voting on the proposal by 20 powers, it all locked 20 balance in the organization_id and proposal_id
		assert_ok!(IdavollModule::vote_proposal(RawOrigin::Signed(OWNER.clone()).into(),proposal_id.clone(),20,true));
		assert_eq!(IdavollAsset::free_balance(asset_id,&OWNER.clone()),80);
		assert_eq!(IdavollAsset::total_balance(asset_id,&OWNER.clone()),100);
		// the owner voting on the proposal by 10 powers, it all locked 30 balance in the organization_id and proposal_id
		assert_ok!(IdavollModule::vote_proposal(RawOrigin::Signed(OWNER.clone()).into(),proposal_id.clone(),10,true));
		assert_eq!(IdavollAsset::free_balance(asset_id,&OWNER.clone()),70);
		assert_eq!(IdavollAsset::total_balance(asset_id,&OWNER.clone()),100);
		// the owner voting on the proposal by 30 powers, it all locked 60 balance in the organization_id and proposal_id
		assert_ok!(IdavollModule::vote_proposal(RawOrigin::Signed(OWNER.clone()).into(),proposal_id.clone(),30,true));
		assert_eq!(IdavollAsset::free_balance(asset_id,&OWNER.clone()),40);
		assert_eq!(IdavollAsset::total_balance(asset_id,&OWNER.clone()),100);
		// the owner voting on the proposal by 20 powers, it all locked 80 balance in the organization_id and proposal_id
		// now the 'yes' vote was 80% of the all, it will pass the proposal, it will close the proposal and unlocked the user's
		// balance. now the user(owner) has 100 balance ot the token
		assert_ok!(IdavollModule::vote_proposal(RawOrigin::Signed(OWNER.clone()).into(),proposal_id.clone(),20,true));
		assert_eq!(IdavollAsset::free_balance(asset_id,&OWNER.clone()),100);
		assert_eq!(IdavollAsset::total_balance(asset_id,&OWNER.clone()),100);

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
		assert_ok!(IdavollModule::deposit_to_organization(RawOrigin::Signed(A).into(),c,200));
		assert_eq!(IdavollModule::get_count_members(org_id),1);
		assert_eq!(IdavollAsset::vault_balance_of(org_id),Ok(200 as u64));
		// add 4 members and assign the tokens for voting
		assert_ok!(IdavollModule::add_member_and_assign_token(RawOrigin::Signed(OWNER.clone()).into(),1,c,5));
		assert_ok!(IdavollModule::add_member_and_assign_token(RawOrigin::Signed(OWNER.clone()).into(),2,c,10));
		assert_ok!(IdavollModule::add_member_and_assign_token(RawOrigin::Signed(OWNER.clone()).into(),3,c,20));
		assert_ok!(IdavollModule::add_member_and_assign_token(RawOrigin::Signed(OWNER.clone()).into(),4,c,25));

		assert_eq!(IdavollModule::get_count_members(org_id),5);
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
		assert_ok!(IdavollModule::deposit_to_organization(RawOrigin::Signed(A).into(),c,200));
		assert_eq!(IdavollModule::get_count_members(org_id),1);
		assert_eq!(IdavollAsset::vault_balance_of(org_id),Ok(200 as u64));
		// add 4 members and assign the tokens for voting
		assert_ok!(IdavollModule::add_member_and_assign_token(RawOrigin::Signed(OWNER.clone()).into(),1,c,5));
		assert_ok!(IdavollModule::add_member_and_assign_token(RawOrigin::Signed(OWNER.clone()).into(),2,c,10));
		assert_ok!(IdavollModule::add_member_and_assign_token(RawOrigin::Signed(OWNER.clone()).into(),3,c,20));
		assert_ok!(IdavollModule::add_member_and_assign_token(RawOrigin::Signed(OWNER.clone()).into(),4,c,25));

		assert_eq!(IdavollModule::get_count_members(org_id),5);
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
