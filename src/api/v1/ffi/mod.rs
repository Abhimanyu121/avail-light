pub mod types;
use crate::types::{Mode, RuntimeConfig, State};
use anyhow::anyhow;

use super::common::types::{
	ClientResponse, ConfidenceResponse, ExtrinsicsDataResponse, LatestBlockResponse,
};
use crate::api::v1::common;
use crate::api::v1::common::types::AppDataQuery;
use crate::api::v1::ffi::types::{FfiSafeAppDataQuery, FfiSafeConfidenceResponse, FfiSafeStatus};
use crate::light_client_commons::{run, DB, STATE};
use rocksdb::DB;
use std::{
	ffi::CString,
	sync::{Arc, Mutex},
};
use tokio::sync::mpsc::channel;
use tracing::error;

fn get_state() -> Arc<Mutex<State>> {
	match unsafe { STATE.clone() } {
		Some(state) => return state,
		_ => {
			panic!("Client not initialized")
		},
	}
}

fn get_db() -> Arc<DB> {
	match unsafe { DB.clone() } {
		Some(db) => return db,
		_ => {
			panic!("Client not initialized")
		},
	}
}
#[allow(non_snake_case)]
#[no_mangle]
#[tokio::main]
pub async unsafe extern "C" fn start_light_node() -> bool {
	let mut cfg: RuntimeConfig = RuntimeConfig::default();
	cfg.log_level = String::from("info");
	cfg.http_server_host = String::from("10.0.2.2");
	cfg.http_server_port = (7000, 8080);

	cfg.full_node_ws = [String::from("ws://10.0.2.2:9944")].to_vec();
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
	panic!("Error: {}", error);
	return false;
}

#[no_mangle]
pub extern "C" fn c_mode(app_id: u32) -> ClientResponse<Mode> {
	return common::mode(Some(app_id));
}
#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub extern "C" fn c_confidence(block_number: u32) -> ClientResponse<FfiSafeConfidenceResponse> {
	let db: Arc<DB> = get_db();
	let state: Arc<Mutex<State>> = get_state();

	let client_response: ClientResponse<ConfidenceResponse> =
		common::confidence(block_number, db, state);

	match client_response {
		ClientResponse::Normal(res) => {
			return ClientResponse::Normal(FfiSafeConfidenceResponse {
				block: res.block,
				confidence: res.confidence,
				serialised_confidence: CString::new(res.serialised_confidence.unwrap()).unwrap(),
			});
		},

		ClientResponse::Error(e) => {
			return ClientResponse::Error(e);
		},
		ClientResponse::InProcess => {
			return ClientResponse::InProcess;
		},
		ClientResponse::NotFound => {
			return ClientResponse::NotFound;
		},
		ClientResponse::NotFinalized => {
			return ClientResponse::NotFinalized;
		},
	}
}

#[no_mangle]
pub extern "C" fn c_status(app_id: u32) -> ClientResponse<FfiSafeStatus> {
	let db: Arc<DB> = get_db();
	let state: Arc<Mutex<State>> = get_state();
	let client_response = common::status(Some(app_id), state, db);
	match client_response {
		ClientResponse::Normal(res) => {
			return ClientResponse::Normal(FfiSafeStatus {
				block_num: res.block_num,
				app_id: res.app_id.unwrap(),
				confidence: res.confidence,
			});
		},

		ClientResponse::Error(e) => {
			return ClientResponse::Error(e);
		},
		ClientResponse::InProcess => {
			return ClientResponse::InProcess;
		},
		ClientResponse::NotFound => {
			return ClientResponse::NotFound;
		},
		ClientResponse::NotFinalized => {
			return ClientResponse::NotFinalized;
		},
	}
}

#[no_mangle]
pub extern "C" fn c_latest_block() -> ClientResponse<LatestBlockResponse> {
	let state: Arc<Mutex<State>> = get_state();
	let latest_block = common::latest_block(state);
	match latest_block {
		ClientResponse::Normal(res) => {
			panic!("res {}", res.latest_block)
		},
		ClientResponse::NotFound => panic!("Not found"),
		ClientResponse::NotFinalized => panic!("NotFinalized"),
		ClientResponse::InProcess => panic!("InProcess"),
		ClientResponse::Error(err) => panic!("err {}", err),
	}
}
#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub extern "C" fn c_appdata(
	block_num: u32,
	query: FfiSafeAppDataQuery,
	app_id: u32,
) -> ClientResponse<ExtrinsicsDataResponse> {
	let db: Arc<DB> = get_db();
	let state: Arc<Mutex<State>> = get_state();
	return common::appdata(
		block_num,
		AppDataQuery {
			decode: Some(query.decode),
		},
		db,
		Some(app_id),
		state,
	);
}
