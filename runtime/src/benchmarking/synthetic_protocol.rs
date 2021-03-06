use super::utils::{dollars, set_ausd_balance, set_price};
use crate::{
	AccountId, BaseLiquidityPoolsForSynthetic, Oracle, Price, Runtime, SyntheticLiquidityPools, SyntheticProtocol,
};

use frame_support::traits::ChangeMembers;
use frame_system::RawOrigin;
use sp_runtime::{DispatchError, DispatchResult, FixedPointNumber};
use sp_std::prelude::*;

use frame_benchmarking::account;
use orml_benchmarking::runtime_benchmarks;

use module_primitives::CurrencyId::FEUR;
use module_primitives::*;

const SEED: u32 = 0;
const MAX_TRADER_INDEX: u32 = 1000;
const MAX_POOL_OWNER_INDEX: u32 = 1000;
const MAX_DOLLARS: u32 = 1000;

fn create_pool(p: u32) -> Result<AccountId, DispatchError> {
	let owner: AccountId = account("owner", p, SEED);
	BaseLiquidityPoolsForSynthetic::create_pool(RawOrigin::Signed(owner.clone()).into())?;

	SyntheticLiquidityPools::set_spread(RawOrigin::Signed(owner.clone()).into(), 0, FEUR, 0, 0)?;
	SyntheticLiquidityPools::set_synthetic_enabled(RawOrigin::Signed(owner.clone()).into(), 0, FEUR, true)?;

	Ok(owner)
}

fn add_liquidity(owner: &AccountId, liquidity: Balance) -> DispatchResult {
	set_ausd_balance(owner, liquidity + dollars(1u128))?;
	BaseLiquidityPoolsForSynthetic::deposit_liquidity(RawOrigin::Signed(owner.clone()).into(), 0, liquidity)
}

fn set_up_oracle() {
	<Oracle as ChangeMembers<_>>::change_members_sorted(
		&vec![],
		&vec![],
		&vec![AccountId::from([100u8; 32]), AccountId::from([101u8; 32])],
	);
}

runtime_benchmarks! {
	{ Runtime, synthetic_protocol }

	_ {
		let t in 1 .. MAX_TRADER_INDEX => ();
		let p in 1 .. MAX_POOL_OWNER_INDEX => ();
		let d in 100 .. MAX_DOLLARS => ();
	}

	mint {
		let p in ...;
		let t in ...;
		let d in ...;

		let owner = create_pool(p)?;
		let trader: AccountId = account("trader", t, SEED);

		let balance = dollars(d);
		set_ausd_balance(&trader, balance + dollars(1u128))?;

		add_liquidity(&owner, balance)?;

		set_up_oracle();
		set_price(vec![(CurrencyId::FEUR, Price::saturating_from_integer(1))])?;
	}: _(RawOrigin::Signed(trader), 0, FEUR, balance, Price::saturating_from_integer(2))

	redeem {
		let p in ...;
		let t in ...;
		let d in ...;

		let owner = create_pool(p)?;
		let trader: AccountId = account("trader", t, SEED);

		let balance = dollars(d);
		set_ausd_balance(&trader, balance + dollars(1u128))?;

		add_liquidity(&owner, balance)?;

		set_up_oracle();
		set_price(vec![(CurrencyId::FEUR, Price::saturating_from_integer(1))])?;
		SyntheticProtocol::mint(RawOrigin::Signed(trader.clone()).into(), 0, FEUR, balance, Price::saturating_from_integer(2))?;
	}: _(RawOrigin::Signed(trader), 0, FEUR, balance / 2, Price::zero())

	liquidate {
		let p in ...;
		let t in ...;
		let d in ...;

		let owner = create_pool(p)?;
		let trader: AccountId = account("trader", t, SEED);

		let balance = dollars(d);
		set_ausd_balance(&trader, balance + dollars(1u128))?;

		add_liquidity(&owner, balance)?;

		set_up_oracle();
		set_price(vec![(CurrencyId::FEUR, Price::saturating_from_integer(1))])?;
		SyntheticProtocol::mint(RawOrigin::Signed(trader.clone()).into(), 0, FEUR, balance, Price::saturating_from_integer(2))?;

		set_price(vec![(CurrencyId::FEUR, Price::saturating_from_rational(12, 10))])?;
	}: _(RawOrigin::Signed(trader), 0, FEUR, balance / 2)

	add_collateral {
		let p in ...;
		let t in ...;
		let d in ...;

		let _ = create_pool(p)?;
		let trader: AccountId = account("trader", t, SEED);

		let balance = dollars(d);
		set_ausd_balance(&trader, balance + dollars(1u128))?;
	}: _(RawOrigin::Signed(trader), 0, FEUR, balance)

	withdraw_collateral {
		let p in ...;
		let t in ...;
		let d in ...;

		let owner = create_pool(p)?;
		let trader: AccountId = account("trader", t, SEED);

		let balance = dollars(d);
		set_ausd_balance(&trader, balance + dollars(1u128))?;

		add_liquidity(&owner, balance)?;

		set_up_oracle();
		set_price(vec![(CurrencyId::FEUR, Price::saturating_from_integer(1))])?;
		SyntheticProtocol::mint(RawOrigin::Signed(trader.clone()).into(), 0, FEUR, balance, Price::saturating_from_integer(2))?;

		set_price(vec![(CurrencyId::FEUR, Price::saturating_from_rational(1, 2))])?;
	}: _(RawOrigin::Signed(owner), 0, FEUR)
}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::assert_ok;

	fn new_test_ext() -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		pallet_membership::GenesisConfig::<Runtime, pallet_membership::Instance3> {
			members: vec![AccountId::from([100u8; 32]), AccountId::from([101u8; 32])],
			phantom: Default::default(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		t.into()
	}

	#[test]
	fn mint() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_mint());
		});
	}

	#[test]
	fn redeem() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_redeem());
		});
	}

	#[test]
	fn liquidate() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_liquidate());
		});
	}

	#[test]
	fn add_collateral() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_add_collateral());
		});
	}

	#[test]
	fn withdraw_collateral() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_withdraw_collateral());
		});
	}
}
