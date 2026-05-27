use borsh::BorshDeserialize;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;

/// Pyth SOL/USD price-feed account on Solana mainnet.
pub const SOL_USD_FEED: Pubkey = pubkey!("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");

/// Struct Source: https://github.com/pyth-network/pyth-crosschain/blob/3564fb58b46ad55a228b55e68920745ef027d8ef/target_chains/solana/pyth_solana_receiver_sdk/src/price_update.rs#L51
#[allow(dead_code)]
#[derive(BorshDeserialize, Debug)]
pub enum VerificationLevel {
	Partial { num_signatures: u8 },
	Full,
}

#[allow(dead_code)]
#[derive(BorshDeserialize, Debug)]
pub struct PriceFeedMessage {
	pub feed_id: [u8; 32],
	pub price: i64,
	pub conf: u64,
	pub exponent: i32,
	pub publish_time: i64,
	pub prev_publish_time: i64,
	pub ema_price: i64,
	pub ema_conf: u64,
}

#[allow(dead_code)]
#[derive(BorshDeserialize, Debug)]
pub struct PriceUpdateV2 {
	pub write_authority: [u8; 32],
	pub verification_level: VerificationLevel,
	pub price_message: PriceFeedMessage,
	pub posted_slot: u64,
}

impl PriceUpdateV2 {
	pub fn get_price(&self) -> f64 {
		let price = self.price_message.price as f64;
		price * (10f64).powi(self.price_message.exponent)
	}
}

pub fn decode_price(account_data: &[u8]) -> Result<f64, String> {
	if account_data.len() < 8 {
		return Err("Pyth account data too short for anchor discriminator".into());
	}
	// Skip 8-byte anchor discriminator, then borsh-decode.
	let mut data: &[u8] = &account_data[8..];
	let update = PriceUpdateV2::deserialize(&mut data)
		.map_err(|e| format!("Failed to decode Pyth price: {e}"))?;
	Ok(update.get_price())
}
