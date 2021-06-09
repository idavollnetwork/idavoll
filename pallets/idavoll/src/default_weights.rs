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


#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

impl crate::WeightInfo for () {
    fn create_organization(b: u32) -> Weight {
        (100_000_000_u64)
            .saturating_add((10_000_000_u64).saturating_mul(b as Weight))
            .saturating_add(DbWeight::get().reads(1_u64))
            .saturating_add(DbWeight::get().reads((1_u64).saturating_mul(b as Weight)))
            .saturating_add(DbWeight::get().writes(1_u64))
            .saturating_add(DbWeight::get().writes((1_u64).saturating_mul(b as Weight)))
    }
    fn deposit_to_organization() -> Weight {
        (100_000_000_u64)
    }
    fn create_proposal() -> Weight {
        (600_000_000_u64)
            .saturating_add(DbWeight::get().reads(1_u64))
            .saturating_add(DbWeight::get().writes(1_u64))
    }
    fn veto_proposal() -> Weight {
        (200_000_000_u64)
            .saturating_add(DbWeight::get().reads((100_u64).saturating_mul(10_u64)))
            .saturating_add(DbWeight::get().writes((500_u64).saturating_mul(10_u64)))
    }
    fn add_member_by_onwer() -> Weight {
        (200_000_000_u64)
            .saturating_add(DbWeight::get().reads((100_u64).saturating_mul(10_u64)))
            .saturating_add(DbWeight::get().writes((500_u64).saturating_mul(10_u64)))
    }
}
