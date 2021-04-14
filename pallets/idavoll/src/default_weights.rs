// This file is part of Idavoll Network.

// Copyright (C) 2021 Idavoll Network.

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.


#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

impl crate::WeightInfo for () {
    fn create_origanization(b: u32) -> Weight {
        (100_000_000 as Weight)
            .saturating_add((10_000_000 as Weight).saturating_mul(b as Weight))
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(b as Weight)))
            .saturating_add(DbWeight::get().writes(1 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(b as Weight)))
    }
    fn create_proposal() -> Weight {
        (100_000_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn veto_proposal(b: u32, c: u32) -> Weight {
        (100_000_000 as Weight)
            .saturating_add((10_000_000 as Weight).saturating_mul(b as Weight))
            .saturating_add((10_000_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(b as Weight)))
            .saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(c as Weight)))
            .saturating_add(DbWeight::get().writes(1 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(b as Weight)))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(c as Weight)))
    }
    fn finish_proposal(b: u32, c: u32) -> Weight {
        (100_000_000 as Weight)
            .saturating_add((10_000_000 as Weight).saturating_mul(b as Weight))
            .saturating_add((10_000_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(b as Weight)))
            .saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(c as Weight)))
            .saturating_add(DbWeight::get().writes(1 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(b as Weight)))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(c as Weight)))
    }
}