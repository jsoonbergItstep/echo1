use borsh::BorshDeserialize;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;

use crate::util::{SOL_DECIMALS, USDC_DECIMALS};

/// Canonical Orca Whirlpool SOL/USDC pool.
/// https://www.orca.so/pools/Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE
pub const SOL_USDC_POOL: Pubkey = pubkey!("Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE");

/// Structs source: https://github.com/orca-so/whirlpools/blob/main/programs/whirlpool/src/state/whirlpool.rs
const NUM_REWARDS: usize = 3;

#[allow(dead_code)]
#[derive(BorshDeserialize, Debug)]
pub struct WhirlpoolRewardInfo {
	pub mint: Pubkey,
	pub vault: Pubkey,
	pub extension: [u8; 32],
	pub emissions_per_second_x64: u128,
	pub growth_global_x64: u128,
}

#[allow(dead_code)]
#[derive(BorshDeserialize, Debug)]
pub struct Whirlpool {
	pub whirlpools_config: Pubkey,                        // 32
	pub whirlpool_bump: [u8; 1],                          // 1
	pub tick_spacing: u16,                                // 2
	pub fee_tier_index_seed: [u8; 2],                     // 2
	pub fee_rate: u16,                                    // 2
	pub protocol_fee_rate: u16,                           // 2
	pub liquidity: u128,                                  // 16
	pub sqrt_price: u128,                                 // 16  ← byte 65 after 8-byte discriminator
	pub tick_current_index: i32,                          // 4
	pub protocol_fee_owed_a: u64,                         // 8
	pub protocol_fee_owed_b: u64,                         // 8
	pub token_mint_a: Pubkey,                             // 32
	pub token_vault_a: Pubkey,                            // 32
	pub fee_growth_global_a: u128,                        // 16
	pub token_mint_b: Pubkey,                             // 32
	pub token_vault_b: Pubkey,                            // 32
	pub fee_growth_global_b: u128,                        // 16
	pub reward_last_updated_timestamp: u64,               // 8
	pub reward_infos: [WhirlpoolRewardInfo; NUM_REWARDS], // 384
}

/// Q64.64 sqrt_price → spot price, rescaled for the pair's decimals.
/// SOL = token_a (9 dec), USDC = token_b (6 dec).
pub fn decode_price(account_data: &[u8]) -> Result<f64, String> {
	if account_data.len() < 8 {
		return Err("Orca account data too short for discriminator".into());
	}
	let mut data: &[u8] = &account_data[8..];
	let whirlpool = Whirlpool::deserialize(&mut data)
		.map_err(|e| format!("Failed to decode Orca Whirlpool: {e}"))?;
	let sqrt = (whirlpool.sqrt_price as f64) / (2f64).powi(64);
	Ok(sqrt * sqrt * (10f64).powi(SOL_DECIMALS - USDC_DECIMALS))
}
