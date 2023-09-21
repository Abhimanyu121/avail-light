use anyhow::{anyhow, Result};
use rocksdb::DB;
use std::net::Ipv4Addr;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::channel;

use crate::api::v1::common::types::{ClientResponse, ExtrinsicsDataResponse, LatestBlockResponse};
use crate::api::v1::ffi::types::{FfiSafeAppDataQuery, FfiSafeConfidenceResponse, FfiSafeStatus};
use crate::api::v1::ffi::{c_appdata, c_confidence, c_latest_block, c_mode, c_status};
use crate::light_client_commons::run;
use crate::types::{Mode, RuntimeConfig, SecretKey, State};
use tracing::error;

use crate::api::v1::ffi::EmbedState;
use crate::light_client_commons::{DB, STATE};

// #[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
#[tokio::main]
pub async unsafe extern "C" fn start_light_node() -> bool {
	// panic!("Paniced at first");

	let mut cfg: RuntimeConfig = RuntimeConfig::default();
	cfg.log_level = String::from("info");
	cfg.http_server_host = String::from("127.0.0.1");
	cfg.http_server_port = (7000, 8080);

	// cfg.secret_key = SecretKey( seed : "avail" );
	// cfg.libp2p_port = 37000;

	cfg.full_node_ws = [String::from("wss://kate.avail.tools/ws")].to_vec();
	cfg.app_id = Some(0);
	cfg.confidence = 92.0;
	cfg.avail_path =
		String::from("/data/user/0/com.example.avail_light_app/app_flutter/avail_path");
	cfg.bootstraps = [(
		String::from("12D3KooWMm1c4pzeLPGkkCJMAgFbsfQ8xmVDusg272icWsaNHWzN"),
		("/ip4/10.0.2.2/tcp/37000").parse().unwrap(),
	)]
	.to_vec();
	let (error_sender, mut error_receiver) = channel::<anyhow::Error>(1);

	let res = run(error_sender, cfg, false).await;

	if let Err(error) = res {
		error!("{error}");
		panic!("{}", error);
	// return Err(error);
	} else {
		let (state, db): (Arc<Mutex<State>>, Arc<DB>) = res.unwrap();
		STATE = Some(state);
		DB = Some(db);
		return true;
	};

	let error = match error_receiver.recv().await {
		Some(error) => error,
		None => anyhow!("Failed to receive error message"),
	};
	panic!("panic: {:?}", error);

	// Err(error)
	return false;
}

// #[cfg(target_os = "android")]
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn android_block_confidence(
	block_number: u32,
) -> ClientResponse<FfiSafeConfidenceResponse> {
	if STATE.is_some() && DB.is_some() {
		let embed_state: EmbedState = EmbedState::new(STATE.clone().unwrap(), DB.clone().unwrap());
		return c_confidence(block_number, &embed_state);
	} else {
		return ClientResponse::NotFound;
	}
}

// #[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn android_status(app_id: u32) -> ClientResponse<FfiSafeStatus> {
	if STATE.is_some() && DB.is_some() {
		let embed_state: EmbedState = EmbedState::new(STATE.clone().unwrap(), DB.clone().unwrap());
		return c_status(app_id, &embed_state);
	} else {
		return ClientResponse::NotFound;
	}
}
// #[cfg(target_os = "android")]
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn android_latest_block() -> u32 {
	if STATE.is_some() && DB.is_some() {
		let embed_state: EmbedState = EmbedState::new(STATE.clone().unwrap(), DB.clone().unwrap());
		let resp = c_latest_block(&embed_state);
		match resp {
			ClientResponse::Normal(c) => {
				return c.latest_block;
			},
			ClientResponse::Error(e) => {
				panic!("{}", e.to_string());
			},
			ClientResponse::InProcess => {
				panic!("In Process");
			},
			ClientResponse::NotFinalized => {
				panic!("Not Finalized");
			},
			ClientResponse::NotFound => {
				panic!("Not found");
			},
		}
	} else {
		panic!("Something went wrong");

		//return ClientResponse::NotFound;
	}
}

// #[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn android_appdata(
	block_num: u32,
	query: FfiSafeAppDataQuery,
	app_id: u32,
) -> ClientResponse<ExtrinsicsDataResponse> {
	if STATE.is_some() && DB.is_some() {
		let embed_state: EmbedState = EmbedState::new(STATE.clone().unwrap(), DB.clone().unwrap());
		return c_appdata(block_num, query, app_id, &embed_state);
	} else {
		return ClientResponse::NotFound;
	}
}

// #[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn android_mode(app_id: u32) -> ClientResponse<Mode> {
	if STATE.is_some() && DB.is_some() {
		return c_mode(app_id);
	} else {
		return ClientResponse::NotFound;
	}
}
