use std::{
	marker::PhantomData,
	sync::{Mutex, MutexGuard, OnceLock},
};

use crate::error::CourtContractError;
use cosmwasm_std::{
	from_json,
	testing::{BankQuerier, MockApi},
	Addr, Api, BlockInfo, CanonicalAddr, Coin, ContractInfo, Env, MemoryStorage, OwnedDeps, Querier, QuerierResult,
	QueryRequest, Response, StdError, StdResult, SystemError, SystemResult, Timestamp, TransactionInfo, WasmQuery,
};
use crownfi_cw_common::{
	data_types::canonical_addr::SeiCanonicalAddr,
	extentions::timestamp::TimestampExtentions,
	storage::base::{set_global_storage, GlobalStorage},
};
use sei_cosmwasm::SeiQueryWrapper;

#[derive(Default)]
struct SeiMockApi {
	inner_api: MockApi,
}
impl Api for SeiMockApi {
	fn addr_validate(&self, human: &str) -> StdResult<Addr> {
		let canonical = self.addr_canonicalize(human)?;
		let normalized = self.addr_humanize(&canonical)?;
		if human != normalized {
			return Err(StdError::generic_err("Invalid input: address not normalized"));
		}
		Ok(normalized)
	}
	fn addr_canonicalize(&self, human: &str) -> StdResult<CanonicalAddr> {
		Ok(SeiCanonicalAddr::try_from(human)?.as_slice().to_vec().into())
	}
	fn addr_humanize(&self, canonical: &CanonicalAddr) -> StdResult<Addr> {
		SeiCanonicalAddr::try_from(canonical)?.try_into()
	}
	fn secp256k1_verify(
		&self,
		message_hash: &[u8],
		signature: &[u8],
		public_key: &[u8],
	) -> Result<bool, cosmwasm_std::VerificationError> {
		self.inner_api.ed25519_verify(message_hash, signature, public_key)
	}
	fn secp256k1_recover_pubkey(
		&self,
		message_hash: &[u8],
		signature: &[u8],
		recovery_param: u8,
	) -> Result<Vec<u8>, cosmwasm_std::RecoverPubkeyError> {
		self.inner_api
			.secp256k1_recover_pubkey(message_hash, signature, recovery_param)
	}
	fn ed25519_verify(
		&self,
		message: &[u8],
		signature: &[u8],
		public_key: &[u8],
	) -> Result<bool, cosmwasm_std::VerificationError> {
		self.inner_api.ed25519_verify(message, signature, public_key)
	}
	fn ed25519_batch_verify(
		&self,
		messages: &[&[u8]],
		signatures: &[&[u8]],
		public_keys: &[&[u8]],
	) -> Result<bool, cosmwasm_std::VerificationError> {
		self.inner_api.ed25519_batch_verify(messages, signatures, public_keys)
	}
	fn debug(&self, message: &str) {
		self.inner_api.debug(message)
	}
}

struct ThreadSafeMockSeiQuerier {
	bank: BankQuerier,
}

impl ThreadSafeMockSeiQuerier {
	pub fn new(balances: &[(&str, &[Coin])]) -> Self {
		ThreadSafeMockSeiQuerier {
			bank: BankQuerier::new(balances),
		}
	}
	// set a new balance for the given address and return the old balance
	/*
	pub fn update_balance(
		&mut self,
		addr: impl Into<String>,
		balance: Vec<Coin>,
	) -> Option<Vec<Coin>> {
		self.bank.update_balance(addr, balance)
	}

	pub fn set_denom_metadata(&mut self, denom_metadata: &[DenomMetadata]) {
		self.bank.set_denom_metadata(denom_metadata);
	}
	*/
	pub fn handle_query(&self, request: &QueryRequest<SeiQueryWrapper>) -> QuerierResult {
		// Bare-minimum for now
		match request {
			QueryRequest::Bank(bank_query) => self.bank.query(bank_query),
			QueryRequest::Custom(_) => SystemResult::Err(SystemError::UnsupportedRequest {
				kind: "SeiQuery".to_string(),
			}),
			QueryRequest::Wasm(msg) => match msg {
				WasmQuery::Smart { contract_addr, .. } => SystemResult::Err(SystemError::NoSuchContract {
					addr: contract_addr.clone(),
				}),
				WasmQuery::Raw { contract_addr, .. } => SystemResult::Err(SystemError::NoSuchContract {
					addr: contract_addr.clone(),
				}),
				WasmQuery::ContractInfo { contract_addr, .. } => SystemResult::Err(SystemError::NoSuchContract {
					addr: contract_addr.clone(),
				}),
				_ => SystemResult::Err(SystemError::UnsupportedRequest {
					kind: "WasmQuery::_ (unmatched)".to_string(),
				}),
			},
			_ => SystemResult::Err(SystemError::UnsupportedRequest {
				kind: "_ (unmatched)".to_string(),
			}),
		}
	}
}
impl Default for ThreadSafeMockSeiQuerier {
	fn default() -> Self {
		ThreadSafeMockSeiQuerier::new(&[])
	}
}

impl Querier for ThreadSafeMockSeiQuerier {
	fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
		let request: QueryRequest<SeiQueryWrapper> = match from_json(bin_request) {
			Ok(v) => v,
			Err(e) => {
				return SystemResult::Err(SystemError::InvalidRequest {
					error: format!("Parsing query request: {e}"),
					request: bin_request.into(),
				})
			}
		};
		self.handle_query(&request)
	}
}

type SeiMockEnvDeps = OwnedDeps<GlobalStorage, SeiMockApi, ThreadSafeMockSeiQuerier, SeiQueryWrapper>;

//static STORAGE_MUTEX: Arc<Mutex<Env>> = Arc::new(Mutex::new(mock_env()));

fn build_env() -> (Env, SeiMockEnvDeps) {
	let env = Env {
		block: BlockInfo {
			height: 9000,
			time: Timestamp::from_millis(1712517600000),
			chain_id: "sei-chain".to_string(),
		},
		transaction: Some(TransactionInfo { index: 3 }),
		contract: ContractInfo {
			address: Addr::unchecked("sei1235xjueqd9ejqargv5sxxmmww3exzcm5ypskgerjv4ehxgrvdakqu2lr7g"),
		},
	};
	let deps = SeiMockEnvDeps {
		api: Default::default(),
		querier: ThreadSafeMockSeiQuerier::new(&[(env.contract.address.as_str(), &[Coin::new(1000000, "usei")])]),
		storage: GlobalStorage {},
		custom_query_type: PhantomData,
	};
	(env, deps)
}
fn global_env() -> &'static Mutex<(Env, SeiMockEnvDeps)> {
	static MUTEX: OnceLock<Mutex<(Env, SeiMockEnvDeps)>> = OnceLock::new();
	MUTEX.get_or_init(|| Mutex::new(build_env()))
}
fn new_global_env() -> MutexGuard<'static, (Env, SeiMockEnvDeps)> {
	let mut env = global_env().lock().unwrap();
	set_global_storage(Box::new(MemoryStorage::new()));
	*env = build_env();
	env
}

const RANDOM_CONTRACT: &str = "sei1g9hx7argv4ezqcm0de68yctrwssxzerywfjhxueqv9uhjmrdv9hsptdzgf";

const ADMIN_ACCOUNT: &str = "sei1w35x2grddaehggrswf5hv6tvv4nk2eppfl8p44";
const SHARES_HOLDER_ACCOUNT_1: &str = "sei12pexjanfd3jkgem9vss8xetwv3jhygp392q3xy";
const SHARES_HOLDER_ACCOUNT_2: &str = "sei12pexjanfd3jkgem9vss8xetwv3jhygpjte48gm";
const SHARES_HOLDER_ACCOUNT_3: &str = "sei12pexjanfd3jkgem9vss8xetwv3jhygpnk0pj4f";
const SHARES_HOLDER_ACCOUNT_4: &str = "sei12pexjanfd3jkgem9vss8xetwv3jhygp5hkkz5v";
const SHARES_HOLDER_ACCOUNT_5: &str = "sei12pexjanfd3jkgem9vss8xetwv3jhygp42qzhf7";

const RANDOM_ACCOUNT_1: &str = "sei12fskuer0d5s8qmr9vgsxummjd45k2gp3u6r9dc";
const RANDOM_ACCOUNT_2: &str = "sei12fskuer0d5s8qmr9vgsxummjd45k2gpjjfknr8";
const RANDOM_ACCOUNT_3: &str = "sei12fskuer0d5s8qmr9vgsxummjd45k2gpn0lzx74";
const RANDOM_ACCOUNT_4: &str = "sei12fskuer0d5s8qmr9vgsxummjd45k2gp5wx4kls";
const RANDOM_ACCOUNT_5: &str = "sei12fskuer0d5s8qmr9vgsxummjd45k2gp4nsprzz";
const RANDOM_EVM_ACCOUNT_1: &str = "0x69207370696c6C206d79206472696E6b20545F54";

mod contract_execution;
