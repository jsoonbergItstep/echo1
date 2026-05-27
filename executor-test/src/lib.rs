use std::str::FromStr;

use futures_util::StreamExt;
use zela_std::{CustomProcedure, JsonValue, RpcError, json};

#[derive(Debug, serde::Deserialize)]
pub struct MyParams(pub String, pub u32);

#[derive(serde::Serialize)]
pub struct MySuccessData {
	res: String,
}

pub struct TestProcedure;
impl CustomProcedure for TestProcedure {
	type Params = MyParams;
	type SuccessData = MySuccessData;
	type ErrorData = JsonValue;

	async fn run(params: Self::Params) -> Result<Self::SuccessData, RpcError<Self::ErrorData>> {
		match params.1 {
			// unconditionally panics
			1 => {
				panic!();
			}
			// opens an RPC websocket and waits for the first account notification, returns the notification
			2 => {
				log::trace!("Opening RPC Websocket");
				let sub = match zela_std::Subscription::new(
					"account",
					json!([params.0, { "commitment": "processed" }]),
				) {
					Ok(Ok(v)) => v,
					Ok(Err(RpcError {
						code,
						message,
						data,
					})) => {
						return Err(RpcError {
							code,
							message: format!("Failed to open rpc subscription: {message}"),
							data,
						});
					}
					Err(err) => {
						return Err(RpcError {
							code: 500,
							message: format!("Failed to open rpc subscription: {err}"),
							data: None,
						});
					}
				};

				let notif: JsonValue = match sub.recv() {
					Ok(v) => v,
					Err(err) => {
						return Err(RpcError {
							code: 500,
							message: format!("Failed to receive rpc notification: {err}"),
							data: None,
						});
					}
				};

				Ok(MySuccessData {
					res: notif.to_string(),
				})
			}
			// opens an RPC websocket using the PubsubClient interface and waits for the first account notification, returns the owner
			3 => {
				let pubsub = zela_std::rpc_client::PubsubClient::new();
				let mut sub = pubsub.account_subscribe(
					&solana_sdk::pubkey::Pubkey::from_str(&params.0).unwrap(),
					None
				).await.unwrap();
				let notif = sub.next().await.unwrap();
				log::trace!("Notification {notif:?}");

				Ok(MySuccessData {
					res: notif.value.owner,
				})
			}
			// sleeps for 5 seconds and reports the time information it gets from the system
			4 => {
				let before = std::time::SystemTime::now();
				std::thread::sleep(std::time::Duration::from_secs(5));
				let after = std::time::SystemTime::now();
				Ok(MySuccessData {
					res: format!(
						"Slept for {}s - from {} to {}",
						after.duration_since(before).unwrap().as_secs_f32(),
						before.duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs(),
						after.duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs()
					),
				})
			}
			// calls into the host with invalid JSON and reports the error
			5 => {
				let (code, message) = match zela_std::sys::rockawayx::zela::zela_host::call_rpc("foobar", b"invalid JSON") {
					Ok(Ok(data)) => (0, format!("Ok: {}", String::from_utf8_lossy(&data))),
					Ok(Err(err)) => (0, format!("Rpc err: {err:?}")),
					Err(err) => (err as i32, format!("Err: {:?}", std::io::Error::from_raw_os_error(err as i32)))
				};
				Err(RpcError {
					code,
					message,
					data: None
				})
			}
			// calls the RPC about account info and returns the owner
			_ => {
				log::trace!("Calling rpc {}", "getAccountInfo");
				let res: JsonValue = match zela_std::call_rpc(
					"getAccountInfo",
					json!([params.0, { "commitment": "confirmed" }]),
				) {
					Ok(Ok(v)) => v,
					Ok(Err(RpcError {
						code,
						message,
						data,
					})) => {
						return Err(RpcError {
							code,
							message: format!("Failed to call rpc: {message}"),
							data,
						});
					}
					Err(err) => {
						return Err(RpcError {
							code: 500,
							message: format!("Failed to call rpc: {err}"),
							data: None,
						});
					}
				};
				log::trace!("Got {res} from rpc");

				Ok(MySuccessData {
					res: res["value"]["owner"].as_str().unwrap_or("").to_owned(),
				})
			}
		}
	}

	const LOG_MAX_LEVEL: log::LevelFilter = log::LevelFilter::Trace;
}
zela_std::zela_custom_procedure!(TestProcedure);
