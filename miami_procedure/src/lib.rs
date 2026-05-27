use std::time::Instant;

use serde::{Deserialize, Serialize};

use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use spl_memo::build_memo;

#[cfg(target_arch = "wasm32")]
use zela_std::rpc_client::{RpcClient, RpcSendTransactionConfig};

#[cfg(not(target_arch = "wasm32"))]
pub use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::RpcSendTransactionConfig};

mod orca;
mod pyth;
mod util;

/// https://explorer.solana.com/address/EAKAcGHVpZUJUz1b8orHxsW8qHYMqJ3SfweKzhHDxSij
/// ========================================================================
/// pubkey: EAKAcGHVpZUJUz1b8orHxsW8qHYMqJ3SfweKzhHDxSij
/// ========================================================================
/// Save this seed phrase and your BIP39 passphrase to recover your new keypair:
/// slab render armor raw silver tail panic peanut heart hill inmate plastic
/// ========================================================================
const SIGNER_KEYPAIR: [u8; 64] = [
	47, 153, 169, 245, 0, 27, 253, 31, 175, 115, 199, 43, 238, 239, 190, 43, 92, 97, 174, 233, 214,
	113, 215, 169, 153, 219, 64, 124, 193, 33, 74, 118, 195, 138, 178, 129, 251, 162, 171, 49, 249,
	79, 50, 108, 104, 180, 222, 152, 127, 249, 215, 157, 157, 110, 49, 108, 203, 148, 6, 100, 142,
	6, 3, 208,
];
const PRICE_DIFF_BPS_THRESHOLD: f64 = 80.0; // 0.8%
const CEX_WEIGHT: f64 = 0.7;
const DEX_WEIGHT: f64 = 0.3;

#[derive(Debug, Deserialize)]
pub struct Input {
	pub pair: String,
	pub cex_price: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
	OracleUpdate,
	NoAction,
}

#[derive(Debug, Serialize)]
pub struct Output {
	pub explorer_link: Option<String>,
	pub ms_total: f64,
}

pub struct PropAMMReprice;

impl PropAMMReprice {
	pub async fn run(input: &Input, rpc: &RpcClient) -> Result<Output, String> {
		let t_start = Instant::now();

		log::info!("[CEX] pair={}, price={}", input.pair, input.cex_price);
		// 0. Phase - Validate input
		if input.pair != "SOL/USDC" {
			return Err(format!(
				"Input Error: Only supported pair is 'SOL/USDC', pair={}",
				input.pair
			));
		}
		if input.cex_price < 0.0 {
			return Err(format!(
				"Input Error: CEX price can not be negative, price={}",
				input.cex_price
			));
		}

		// 1. Phase - Onchain read
		let t_read = Instant::now();
		let accounts = rpc
			.get_multiple_accounts(&[pyth::SOL_USD_FEED, orca::SOL_USDC_POOL])
			.await
			.map_err(|e| format!("Failed to fetch oracle accounts: {e}"))?;
		let ms_read = t_read.elapsed().as_secs_f64() * 1000.0;

		let pyth_acc = accounts[0]
			.as_ref()
			.ok_or_else(|| "Pyth account missing".to_string())?;
		let orca_acc = accounts[1]
			.as_ref()
			.ok_or_else(|| "Orca whirlpool missing".to_string())?;

		let pyth_price = pyth::decode_price(&pyth_acc.data)?;
		let orca_price = orca::decode_price(&orca_acc.data)?;

		log::info!("[DEX] pyth={pyth_price}, orca={orca_price} ms_read={ms_read}");
		let dex_prices = [pyth_price, orca_price];

		// 2. Phase - Compute / Decision
		// Equal-weight dex prices
		let dex_weighted = dex_prices.iter().sum::<f64>() / (dex_prices.len() as f64);
		if !dex_weighted.is_finite() || dex_weighted <= 0.0 {
			return Err(format!(
				"Oracle Error: dex_weighted price is not a positive finite value, dex_weighted={dex_weighted}"
			));
		}
		let new_fair_price = CEX_WEIGHT * input.cex_price + DEX_WEIGHT * dex_weighted;
		let diff_bps = ((input.cex_price - dex_weighted).abs() / dex_weighted) * 10_000.0;

		let should_update = diff_bps > PRICE_DIFF_BPS_THRESHOLD;
		log::info!(
			"[DECIDE] diff={diff_bps:.2} bps, fair={new_fair_price}, should_update={should_update}"
		);

		if !should_update {
			let ms_total = t_start.elapsed().as_secs_f64() * 1000.0;
			return Ok(Output {
				explorer_link: None,
				ms_total,
			});
		}

		// 3. Phase - Build update transaction
		let signer = Keypair::try_from(&SIGNER_KEYPAIR[..])
			.map_err(|e| format!("Invalid signer keypair: {e}"))?;
		log::info!("[BUILD] signer_pubkey={}", signer.pubkey());

		let t_blockhash = Instant::now();
		let blockhash = rpc
			.get_latest_blockhash()
			.await
			.map_err(|e| format!("Failed to get latest block hash: {e}"))?;
		let ms_blockhash = t_blockhash.elapsed().as_secs_f64() * 1000.0;
		log::info!("[BUILD] latest_blockhash={blockhash}, ms_blockhash={ms_blockhash}");

		// Compute priority fee
		// Could get fees for specific target address
		// Could just hard code the fee
		let t_priority_fee = Instant::now();
		let fees = rpc
			.get_recent_prioritization_fees(&[])
			.await
			.map_err(|e| format!("Failed to get priority fees: {e}"))?;
		let ms_priority_fee = t_priority_fee.elapsed().as_secs_f64() * 1000.0;

		let max_recent_priority_fee = fees
			.iter()
			.map(|f| f.prioritization_fee)
			.max()
			.unwrap_or(1_000) // 1_000 micro lamports
			.min(25_000); // Cap at 25_000 micro-lamports/CU → 0.000005 SOL (5_000 lamports) at 200_000 CU
		let compute_unit_price_ix =
			ComputeBudgetInstruction::set_compute_unit_price(max_recent_priority_fee);
		// Max CU transaction is allowed to consume (Solana default = 200_000)
		let compute_unit_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(200_000);
		let priority_fee_lamports = (max_recent_priority_fee * 200_000) / 1_000_000;
		log::info!(
			"[BUILD] Received {} recent fees, max_recent_priority_fee={max_recent_priority_fee}, ms_priority_fee={ms_priority_fee}",
			fees.len()
		);

		let memo = format!("{}: {new_fair_price}", input.pair);
		let tx = Transaction::new_signed_with_payer(
			&[
				compute_unit_limit_ix,
				compute_unit_price_ix,
				build_memo(memo.as_bytes(), &[&signer.pubkey()]),
			],
			Some(&signer.pubkey()),
			&[&signer],
			blockhash,
		);
		log::info!("[BUILD] memo={memo}, priority_fee_lamports={priority_fee_lamports}");

		let t_submit = Instant::now();
		let sig = rpc
			.send_transaction_with_config(
				&tx,
				RpcSendTransactionConfig {
					skip_preflight: true,
					..Default::default()
				},
			)
			.await
			.map_err(|e| format!("Failed to send update transaction: {e}"))?;
		let ms_submit = t_submit.elapsed().as_secs_f64() * 1000.0;
		log::info!("[SUBMIT] sig={sig} ms_submit={ms_submit} memo={memo}");

		let ms_total = t_start.elapsed().as_secs_f64() * 1000.0;
		Ok(Output {
			explorer_link: Some(format!("https://solscan.io/tx/{}", sig.to_string())),
			ms_total,
		})
	}
}

#[cfg(target_arch = "wasm32")]
mod zela {
	use super::*;
	use zela_std::{CustomProcedure, RpcError, zela_custom_procedure};

	impl CustomProcedure for PropAMMReprice {
		type Params = Input;
		type ErrorData = ();
		type SuccessData = Output;
		const LOG_MAX_LEVEL: log::LevelFilter = log::LevelFilter::Info;

		async fn run(params: Self::Params) -> Result<Self::SuccessData, RpcError<Self::ErrorData>> {
			let rpc = RpcClient::new();
			Self::run(&params, &rpc).await.map_err(|message| RpcError {
				code: 1,
				message,
				data: None,
			})
		}
	}
	zela_custom_procedure!(PropAMMReprice);
}

#[cfg(test)]
mod tests {
	use super::*;

	use solana_client::nonblocking::rpc_client::RpcClient;
	use solana_sdk::commitment_config::CommitmentConfig;

	const RPC_URL: &str = "https://api.mainnet.solana.com";

	fn init_logger() {
		let _ = env_logger::builder()
			.is_test(true)
			.parse_env(env_logger::Env::new().default_filter_or("info,miami_procedure=debug"))
			.try_init();
	}

	#[tokio::test]
	async fn test() {
		init_logger();

		let rpc = RpcClient::new_with_commitment(RPC_URL.to_owned(), CommitmentConfig::confirmed());

		let input = &(Input {
			pair: "SOL/USDC".to_owned(),
			cex_price: 100.0,
		});

		let res = PropAMMReprice::run(input, &rpc).await.unwrap();
		println!("{res:#?}")
	}
}
