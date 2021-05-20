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

//! the idavoll-asset pallet is a base asset for DAO ,it will be used to token and finance module

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{fmt::Debug};
use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch,
                    traits::{Get,Currency, ReservableCurrency,ExistenceRequirement::AllowDeath},
                    Parameter,ensure};
use frame_system::ensure_signed;
use sp_runtime::{RuntimeDebug, ModuleId,
                 traits::{AtLeast32Bit,One,Zero,Member, AtLeast32BitUnsigned,
                          StaticLookup, Saturating, CheckedSub, CheckedAdd,
                          MaybeSerializeDeserialize,
}};
use codec::{Encode, Decode};

pub mod token;
pub mod finance;

/// The module configuration trait.
pub trait Trait: frame_system::Trait {
    /// The idv-asset's module id, used for deriving its sovereign account ID.
    type ModuleId: Get<ModuleId>;

    /// The staking balance.(for the local asset(idv))
    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    /// The units in which we record balances.
    type Balance: Member + Parameter + AtLeast32BitUnsigned + MaybeSerializeDeserialize + Default + Copy;

    /// The arithmetic type of asset identifier.
    type AssetId: Parameter + Member +MaybeSerializeDeserialize + Ord + AtLeast32Bit + Default + Copy;
}

/// the balance for local token(idv)
pub type LocalBalance<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
pub struct AssetDetails<
    Balance: Encode + Decode + Clone + Debug + Eq + PartialEq,
    AccountId: Encode + Decode + Clone + Debug + Eq + PartialEq,
> {
    /// Can First allocation the token.
    issuer: AccountId,
    /// Can be assigned when first created
    init:   bool,
    /// The total supply across all accounts.
    supply: Balance,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default)]
pub struct AccountAssetMetadata<Balance> {
    /// The free balance.
    free: Balance,
    /// The frozen balance.
    frozen: Balance,
}

impl<Balance: Saturating + Copy> AccountAssetMetadata<Balance> {
    /// Computes and return the total balance, including reserved funds.
    pub fn total(&self) -> Balance {
        self.free.saturating_add(self.frozen)
    }
    pub fn valid(&self) -> Balance {
        self.free
    }
}

decl_event! {
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
		<T as Trait>::Balance,
		<T as Trait>::AssetId,
	{
		/// Some assets were issued. \[asset_id, owner, total_supply\]
		Issued(AssetId, AccountId, Balance),
		/// Some assets were transferred. \[asset_id, from, to, amount\]
		Transferred(AssetId, AccountId, AccountId, Balance),
		/// Some assets were destroyed. \[asset_id, owner, balance\]
		Destroyed(AssetId, AccountId, Balance),
		/// Some assets were minted. \[asset_id, issuer, amount\]
		Minted(AssetId, AccountId, Balance),
		/// Some assets were burned. \[asset_id, issuer, amount\]
		Burned(AssetId, AccountId, Balance),
		/// Some assets were locked. \[asset_id, who, amount\]
		Locked(AssetId, AccountId, Balance),
		/// Some assets were unlocked. \[asset_id, who, amount\]
		UnLocked(AssetId, AccountId, Balance),
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Transfer amount should be non-zero
		AmountZero,
		/// Account balance must be greater than or equal to the transfer amount
		BalanceLow,
		/// Balance should be non-zero
		BalanceZero,
		/// The signing account has no permission to do the operation.
		NoPermission,
		/// The given asset ID is unknown.
		Unknown,
		/// A mint operation lead to an overflow.
		Overflow,
		/// Unknow the organization id
		UnknownOwnerID,
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as IdavollAsset {
		/// The number of units of assets held by any given account.
		pub Balances: map hasher(blake2_128_concat) (T::AssetId, T::AccountId) => AccountAssetMetadata<T::Balance>;
		/// record the balance of the local asset(idv) for every organization
		pub Finances get(fn finances): map hasher(blake2_128_concat) T::AccountId => LocalBalance<T>;
		/// The next asset identifier up for grabs.
		NextAssetId get(fn next_asset_id): T::AssetId;
        // pub Locks get(fn locks): double_map hasher(blake2_128_concat) (T::AssetId, T::AccountId), hasher(blake2_128_concat) LockIdentifier => T::Balance;
        /// The details of an asset.
        pub TotalSupply get(fn total_supply): map hasher(blake2_128_concat) T::AssetId => Option<AssetDetails<T::Balance,T::AccountId>>;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;
        /// The idv-asset's module id, used for deriving its sovereign account ID.
		const ModuleId: ModuleId = T::ModuleId::get();
		/// Move some assets from one holder to another.
		#[weight = 100_000]
		pub fn transfer(origin,
			#[compact] id: T::AssetId,
			target: <T::Lookup as StaticLookup>::Source,
			#[compact] amount: T::Balance
		) -> dispatch::DispatchResult{
			let origin = ensure_signed(origin)?;
			let target = T::Lookup::lookup(target)?;

			Self::base_transfer(id,&origin,&target,amount)
		}
	}
}

// The main implementation block for the module.
impl<T: Trait> Module<T> {
    // Public immutables

    /// Get the asset `id` free balance of `who`.
    pub fn free_balance(id: T::AssetId, who: &T::AccountId) -> T::Balance {
        <Balances<T>>::get((id, who)).free
    }
    /// Get the asset `id` total balance of `who`.
    pub fn total_balance(id: T::AssetId, who: &T::AccountId) -> T::Balance {
        <Balances<T>>::get((id, who)).total()
    }
    /// Get the total supply of an asset `id`.
    pub fn total_issuances(id: T::AssetId) -> T::Balance {
        match <TotalSupply<T>>::get(id) {
            Some(asset) => asset.supply,
            _ => Zero::zero()
        }
    }
    /// Issue a new class of fungible assets. There are, and will only ever be, `total`
    /// such assets and they'll all belong to the `origin` initially. It will have an
    /// identifier `AssetId` instance: this will be specified in the `Issued` event.
    fn create_token(owner: T::AccountId,total: T::Balance) -> T::AssetId {

        let id = Self::next_asset_id();
        <NextAssetId<T>>::mutate(|id| *id += One::one());

        let details = AssetDetails {
            issuer: owner.clone(),
            init: false,
            supply: total,
        };
        let meta = AccountAssetMetadata {
            free:    total,
            frozen: Zero::zero(),
        } ;
        <TotalSupply<T>>::insert(id, details);
        <Balances<T>>::insert((id, owner.clone()), meta);

        Self::deposit_event(RawEvent::Issued(id, owner, total));
        id
    }

    /// Move some assets from one holder to another.
    fn base_transfer(id: T::AssetId, origin: &T::AccountId,
                target: &T::AccountId, amount: T::Balance) -> dispatch::DispatchResult {

        ensure!(!amount.is_zero(), Error::<T>::AmountZero);
        Self::deposit_event(RawEvent::Transferred(id, origin.clone(), target.clone(), amount));
        if *origin == *target {
            return Ok(());
        }

        Balances::<T>::mutate((id, origin.clone()), |origin_account| -> dispatch::DispatchResult {
            ensure!(origin_account.free >= amount, Error::<T>::BalanceLow);
            origin_account.free = origin_account.free.checked_sub(&amount)
                .ok_or(Error::<T>::BalanceLow)?;
            Ok(())
        })?;

        Balances::<T>::mutate((id, target.clone()), |a| -> dispatch::DispatchResult {
            let amount = a.free.saturating_add(amount);
            a.free = amount;
            Ok(())
        })
    }
    fn base_mint(id: T::AssetId, issuer: &T::AccountId, amount: T::Balance) -> dispatch::DispatchResult {
        TotalSupply::<T>::try_mutate(id, |maybe_asset| {
            let details = maybe_asset.as_mut().ok_or(Error::<T>::Unknown)?;

            ensure!(*issuer == details.issuer, Error::<T>::NoPermission);
            details.supply = details.supply.checked_add(&amount).ok_or(Error::<T>::Overflow)?;

            Balances::<T>::try_mutate((id, issuer.clone()), |t| -> dispatch::DispatchResult {
                t.free.saturating_add(amount);
                Ok(())
            })?;
            Self::deposit_event(RawEvent::Minted(id, issuer.clone(), amount));
            Ok(())
        })
    }
    fn base_burn(id: T::AssetId, issuer: &T::AccountId, amount: T::Balance) -> dispatch::DispatchResult {
        TotalSupply::<T>::try_mutate(id, |maybe_asset| {
            let d = maybe_asset.as_mut().ok_or(Error::<T>::Unknown)?;
            ensure!(*issuer == d.issuer, Error::<T>::NoPermission);

            Balances::<T>::try_mutate((id, issuer.clone()), |maybe_account| -> dispatch::DispatchResult {
                ensure!(maybe_account.free >= amount, Error::<T>::BalanceLow);
                maybe_account.free = maybe_account.free.checked_sub(&amount)
                    .ok_or(Error::<T>::BalanceLow)?;
                Ok(())
            })?;

            d.supply = d.supply.saturating_sub(amount);

            Self::deposit_event(RawEvent::Burned(id, issuer.clone(), amount));
            Ok(())
        })
    }
    fn base_lock(id: T::AssetId, who: &T::AccountId, amount: T::Balance) -> dispatch::DispatchResult {
        Balances::<T>::try_mutate((id, who.clone()), |maybe_account| -> dispatch::DispatchResult {
            ensure!(maybe_account.free >= amount, Error::<T>::BalanceLow);
            maybe_account.free = maybe_account.free.checked_sub(&amount)
                .ok_or(Error::<T>::BalanceLow)?;
            maybe_account.frozen = maybe_account.frozen.saturating_add(amount);
            Ok(())
        })?;
        Self::deposit_event(RawEvent::Locked(id, who.clone(), amount));
        Ok(())
    }
    fn base_unlock(id: T::AssetId, who: &T::AccountId, amount: T::Balance) -> dispatch::DispatchResult {
        Balances::<T>::try_mutate((id, who.clone()), |maybe_account| -> dispatch::DispatchResult {
            ensure!(maybe_account.frozen >= amount, Error::<T>::BalanceLow);
            maybe_account.frozen = maybe_account.frozen.checked_sub(&amount)
                .ok_or(Error::<T>::BalanceLow)?;
            maybe_account.free = maybe_account.free.saturating_add(amount);
            Ok(())
        })?;
        Self::deposit_event(RawEvent::UnLocked(id, who.clone(), amount));
        Ok(())
    }


    pub fn vault_balance_of(oid: T::AccountId) -> Result<LocalBalance<T>, dispatch::DispatchError> {
        if Finances::<T>::contains_key(oid.clone()) {
            Ok(<Finances<T>>::get(oid.clone()))
        }else {
            Err(Error::<T>::UnknownOwnerID.into())
        }
    }
    /// transfer the balance to the organization's Vault from the members in the organization
    pub fn transfer_to_vault(oid: T::AccountId,who: T::AccountId,value: LocalBalance<T>) -> dispatch::DispatchResult {
        let balance = T::Currency::free_balance(&who);
        ensure!(balance >= value,Error::<T>::BalanceLow);
        let vault_account = Self::account_id();
        T::Currency::transfer(&who,&vault_account,value,AllowDeath)?;

        Finances::<T>::mutate(oid.clone(), |a| -> dispatch::DispatchResult {
            *a = a.saturating_add(value.clone());
            Ok(())
        })
    }
    /// transfer the balance to `to` from finance's Vault by Call<> function
    pub fn spend_organization_vault(oid: T::AccountId,to: T::AccountId,value: LocalBalance<T>) -> dispatch::DispatchResult {
        let vault_balance = Self::vault_balance_of(oid.clone())?;
        ensure!(vault_balance >= value,Error::<T>::BalanceLow);
        let vault_account = Self::account_id();
        T::Currency::transfer(&vault_account,&to,value,AllowDeath)?;
        Finances::<T>::try_mutate_exists(oid,|x|{
            let balance = x.as_mut().ok_or(Error::<T>::UnknownOwnerID)?;
            *x = Some(balance.saturating_sub(value));
            Ok(())
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use frame_support::{impl_outer_origin, assert_ok, assert_noop, parameter_types, weights::Weight};
    use sp_core::H256;
    use sp_runtime::{Perbill, traits::{BlakeTwo256, IdentityLookup}, testing::Header};


    impl_outer_origin! {
		pub enum Origin for Test where system = frame_system {}
	}

    type System = frame_system::Module<Test>;
    type IdvBalances = pallet_balances::Module<Test>;

    const A: u64 = 100;
    const B: u64 = 200;
    const ORGID: u64 = 1000;
    const ORGID2: u64 = 2000;

    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const MaximumBlockWeight: Weight = 1024;
		pub const MaximumBlockLength: u32 = 2 * 1024;
		pub const AvailableBlockRatio: Perbill = Perbill::one();
		pub const IdvAssetModuleId: ModuleId = ModuleId(*b"py/asset");
	}
    impl frame_system::Trait for Test {
        type BaseCallFilter = ();
        type Origin = Origin;
        type Index = u64;
        type Call = ();
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type DbWeight = ();
        type BlockExecutionWeight = ();
        type ExtrinsicBaseWeight = ();
        type MaximumExtrinsicWeight = MaximumBlockWeight;
        type AvailableBlockRatio = AvailableBlockRatio;
        type MaximumBlockLength = MaximumBlockLength;
        type Version = ();
        type PalletInfo = ();
        type AccountData = pallet_balances::AccountData<u64>;
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
    }

    parameter_types! {
	pub const ExistentialDeposit: u64 = 10;
    }
    impl pallet_balances::Trait for Test {
        type Balance = u64;
        type DustRemoval = ();
        type Event = ();
        type ExistentialDeposit = ExistentialDeposit;
        type AccountStore = System;
        type MaxLocks = ();
        type WeightInfo = ();
    }

    impl Trait for Test {
        type Event = ();
        type Balance = u64;
        type AssetId = u32;
        type Currency = IdvBalances;
        type ModuleId = IdvAssetModuleId;
    }
    type IdavollAsset = Module<Test>;

    fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
        let genesis = pallet_balances::GenesisConfig::<Test> {
            balances: vec![
                (A, 100),
                (B, 200),
            ],
        };
        genesis.assimilate_storage(&mut t).unwrap();
        t.into()
    }

    #[test]
    fn issuing_asset_units_to_issuer_should_work() {
        new_test_ext().execute_with(|| {
            assert_eq!(IdavollAsset::create_token(1, 100),0);
            assert_eq!(IdavollAsset::free_balance(0, &1), 100);
            assert_eq!(IdavollAsset::total_balance(0, &1), 100);
            assert_eq!(IdavollAsset::create_token(1, 100),1);
            assert_eq!(IdavollAsset::free_balance(1, &1), 100);
            assert_eq!(IdavollAsset::total_balance(1, &1), 100);
            assert_eq!(IdavollAsset::create_token(2, 300),2);
            assert_eq!(IdavollAsset::free_balance(2, &2), 300);
            assert_eq!(IdavollAsset::total_balance(2, &2), 300);
        });
    }

    #[test]
    fn base_transfer_should_work() {
        new_test_ext().execute_with(|| {
            assert_eq!(IdavollAsset::create_token(1, 100),0);
            assert_eq!(IdavollAsset::free_balance(0, &1), 100);
            assert_eq!(IdavollAsset::total_balance(0, &1), 100);
            assert_ok!(IdavollAsset::transfer(Origin::signed(1), 0, 2, 50));
            assert_eq!(IdavollAsset::free_balance(0, &1), 50);
            assert_eq!(IdavollAsset::free_balance(0, &2), 50);
            assert_ok!(IdavollAsset::transfer(Origin::signed(2), 0, 3, 31));
            assert_eq!(IdavollAsset::free_balance(0, &1), 50);
            assert_eq!(IdavollAsset::free_balance(0, &2), 19);
            assert_eq!(IdavollAsset::free_balance(0, &3), 31);
            assert_eq!(IdavollAsset::total_issuances(0), 100);
        });
    }

    #[test]
    fn burn_and_mint_should_work() {
        new_test_ext().execute_with(|| {
            assert_eq!(IdavollAsset::create_token(1, 100),0);
            assert_eq!(IdavollAsset::free_balance(0, &1), 100);
            assert_eq!(IdavollAsset::total_balance(0, &1), 100);
            assert_ok!(IdavollAsset::transfer(Origin::signed(1), 0, 2, 50));
            assert_ok!(IdavollAsset::base_mint(0, &1,100));
            assert_eq!(IdavollAsset::total_issuances(0), 200);
            assert_eq!(IdavollAsset::base_mint(0,&2,100), Err(Error::<Test>::NoPermission.into()));
            assert_ok!(IdavollAsset::base_burn(0, &1,20));
            assert_eq!(IdavollAsset::total_issuances(0), 180);
            assert_eq!(IdavollAsset::base_burn(0,&2,100), Err(Error::<Test>::NoPermission.into()));
            assert_eq!(IdavollAsset::base_burn(0,&1,1000), Err(Error::<Test>::BalanceLow.into()));
        });
    }

    #[test]
    fn lock_and_unlock_should_not_work() {
        new_test_ext().execute_with(|| {
            assert_eq!(IdavollAsset::create_token(1, 100),0);
            assert_eq!(IdavollAsset::free_balance(0, &1), 100);
            assert_ok!(IdavollAsset::transfer(Origin::signed(1), 0, 2, 40));
            assert_eq!(IdavollAsset::free_balance(0, &1), 60);
            assert_eq!(IdavollAsset::free_balance(0, &2), 40);
            assert_ok!(IdavollAsset::base_lock(0, &1,20));
            assert_eq!(IdavollAsset::free_balance(0, &1), 40);
            assert_eq!(IdavollAsset::total_balance(0, &1), 60);
            assert_ok!(IdavollAsset::base_unlock(0, &1,20));
            assert_eq!(IdavollAsset::free_balance(0, &1), 60);
            assert_eq!(IdavollAsset::total_balance(0, &1), 60);
            assert_ok!(IdavollAsset::base_lock(0, &1,60));
            assert_eq!(IdavollAsset::free_balance(0, &1), 0);
            assert_eq!(IdavollAsset::total_balance(0, &1), 60);
            assert_ok!(IdavollAsset::base_unlock(0, &1,20));
            assert_eq!(IdavollAsset::free_balance(0, &1), 20);
            assert_eq!(IdavollAsset::total_balance(0, &1), 60);
            assert_noop!(IdavollAsset::base_lock(0, &2, 50), Error::<Test>::BalanceLow);
            assert_noop!(IdavollAsset::base_unlock(0, &2, 50), Error::<Test>::BalanceLow);
        });
    }

    #[test]
    fn Vault_transfer_and_balance_should_not_work() {
        new_test_ext().execute_with(|| {
            assert_noop!(IdavollAsset::vault_balance_of(ORGID),Error::<Test>::UnknownOwnerID);
            assert_ok!(IdavollAsset::transfer_to_vault(ORGID, A,30));
            assert_eq!(IdavollAsset::vault_balance_of(ORGID),Ok(30));
            assert_eq!(IdvBalances::free_balance(IdavollAsset::account_id()),30);
            assert_eq!(IdvBalances::free_balance(A),70);
            assert_noop!(IdavollAsset::transfer_to_vault(ORGID, 1, 50), Error::<Test>::BalanceLow);
            assert_noop!(IdavollAsset::transfer_to_vault(ORGID, A, 100), Error::<Test>::BalanceLow);
            assert_ok!(IdavollAsset::transfer_to_vault(ORGID2, B,60));
            assert_eq!(IdavollAsset::vault_balance_of(ORGID2),Ok(60));
            assert_eq!(IdvBalances::free_balance(IdavollAsset::account_id()),90);
            assert_eq!(IdvBalances::free_balance(B),140);
            // spend the Vault
            assert_noop!(IdavollAsset::spend_organization_vault(ORGID, 1,60),Error::<Test>::BalanceLow);
            assert_noop!(IdavollAsset::spend_organization_vault(2, 1,60),Error::<Test>::UnknownOwnerID);
            assert_ok!(IdavollAsset::spend_organization_vault(ORGID, 1,10));
            assert_eq!(IdavollAsset::vault_balance_of(ORGID),Ok(20));
            assert_eq!(IdvBalances::free_balance(IdavollAsset::account_id()),80);
            assert_eq!(IdvBalances::free_balance(1),10);

            assert_ok!(IdavollAsset::spend_organization_vault(ORGID2, 2,30));
            assert_eq!(IdavollAsset::vault_balance_of(ORGID2),Ok(30));
            assert_eq!(IdvBalances::free_balance(IdavollAsset::account_id()),50);
            assert_eq!(IdvBalances::free_balance(2),30);
        });
    }
}
