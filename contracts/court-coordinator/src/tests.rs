use std::{marker::PhantomData, sync::{Mutex, MutexGuard, OnceLock}};

use cosmwasm_std::{from_json, testing::{BankQuerier, MockApi}, Response, Addr, Api, BlockInfo, CanonicalAddr, Coin, ContractInfo, Env, MemoryStorage, OwnedDeps, Querier, QuerierResult, QueryRequest, StdError, StdResult, SystemError, SystemResult, Timestamp, TransactionInfo, WasmQuery};
use crownfi_cw_common::{data_types::canonical_addr::SeiCanonicalAddr, extentions::timestamp::TimestampExtentions, storage::base::{set_global_storage, GlobalStorage}};
use sei_cosmwasm::SeiQueryWrapper;
use crate::error::CourtContractError;

#[derive(Default)]
struct SeiMockApi {
	inner_api: MockApi
}
impl Api for SeiMockApi {
	fn addr_validate(&self, human: &str) -> StdResult<Addr> {
		let canonical = self.addr_canonicalize(human)?;
		let normalized = self.addr_humanize(&canonical)?;
		if human != normalized {
			return Err(StdError::generic_err(
				"Invalid input: address not normalized",
			));
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
		self.inner_api.secp256k1_recover_pubkey(message_hash, signature, recovery_param)
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
	bank: BankQuerier
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
			QueryRequest::Wasm(msg) => {
				match msg {
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
				}
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

type SeiMockEnvDeps = OwnedDeps::<GlobalStorage, SeiMockApi, ThreadSafeMockSeiQuerier, SeiQueryWrapper>;

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
	MUTEX.get_or_init(|| {
		Mutex::new(build_env())
	})
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

mod contract_execution {
	use super::*;
	use cosmwasm_std::{coin, MessageInfo};
use cw2::{get_contract_version, ContractVersion};
	use crate::{msg::*, state::{app::{CourtAppConfigJsonable, TransactionProposalInfoJsonable}, user::{CourtUserStatsJsonable, CourtUserVoteInfoJsonable}}};
	mod helpers {
		use crate::msg::CourtQueryResponseDenom;

		use super::*;
		pub fn instantiate(
			env_deps: &mut (Env, SeiMockEnvDeps),
			msg_info: Option<MessageInfo>,
			msg: Option<CourtInstantiateMsg>
		) -> Result<Response<sei_cosmwasm::SeiMsg>, CourtContractError> {
			let msg_info = msg_info.unwrap_or(
				MessageInfo {
					sender: Addr::unchecked(ADMIN_ACCOUNT),
					funds: vec![]
				}
			);
			let msg = msg.unwrap_or(
				CourtInstantiateMsg {
					admin: msg_info.sender.clone(),
					shares_mint_amount: 1000000u128.into(),
					shares_mint_receiver: msg_info.sender.clone(),
					minimum_vote_proposal_percent: 10,
					minimum_vote_turnout_percent: 20,
					minimum_vote_pass_percent: 50,
					max_proposal_expiry_time_seconds: 7200,
					execution_expiry_time_seconds: 3600,
					vote_share_name: "Test Votes".into(),
					vote_share_symbol: "TST".into(),
					vote_share_description: "Test vortessadbjhk,sdfgvgjhlksdfjhgbksdv".into()
				}
			);
			let env = env_deps.0.clone();
			crate::contract::instantiate(
				env_deps.1.as_mut(),
				env,
				msg_info,
				msg
			)
		}
		pub fn execute(
			env_deps: &mut (Env, SeiMockEnvDeps),
			msg_info: Option<MessageInfo>,
			msg: CourtExecuteMsg
		)-> Result<Response<sei_cosmwasm::SeiMsg>, CourtContractError> {
			let msg_info = msg_info.unwrap_or(
				MessageInfo {
					sender: Addr::unchecked(RANDOM_ACCOUNT_5),
					funds: vec![]
				}
			);
			let env = env_deps.0.clone();
			crate::contract::execute(
				env_deps.1.as_mut(),
				env,
				msg_info,
				msg
			)
		}
		pub fn query_config(
			env_deps: &(Env, SeiMockEnvDeps),
		) -> Result<CourtAppConfigJsonable, CourtContractError> {
			let env = env_deps.0.clone();
			Ok(
				from_json(
					crate::contract::query(
						env_deps.1.as_ref().into_empty(),
						env,
						CourtQueryMsg::Config
					)?
				)?
			)
		}
		pub fn query_denom(
			env_deps: &(Env, SeiMockEnvDeps),
		) -> Result<CourtQueryResponseDenom, CourtContractError> {
			let env = env_deps.0.clone();
			Ok(
				from_json(
					crate::contract::query(
						env_deps.1.as_ref().into_empty(),
						env,
						CourtQueryMsg::Denom
					)?
				)?
			)
		}
		pub fn query_proposal_amount(
			env_deps: &(Env, SeiMockEnvDeps),
		) -> Result<u32, CourtContractError> {
			let env = env_deps.0.clone();
			Ok(
				from_json(
					crate::contract::query(
						env_deps.1.as_ref().into_empty(),
						env,
						CourtQueryMsg::ProposalAmount
					)?
				)?
			)
		}
		pub fn query_get_proposal(
			env_deps: &(Env, SeiMockEnvDeps),
			id: u32
		) -> Result<Option<CourtQueryResponseTransactionProposal>, CourtContractError> {
			let env = env_deps.0.clone();
			Ok(
				from_json(
					crate::contract::query(
						env_deps.1.as_ref().into_empty(),
						env,
						CourtQueryMsg::GetProposal { id }
					)?
				)?
			)
		}
		pub fn query_get_proposals(
			env_deps: &(Env, SeiMockEnvDeps),
			skip: Option<u32>,
			limit: Option<u32>,
			descending: bool
		) -> Result<Vec<CourtQueryResponseTransactionProposal>, CourtContractError> {
			let env = env_deps.0.clone();
			Ok(
				from_json(
					crate::contract::query(
						env_deps.1.as_ref().into_empty(),
						env,
						CourtQueryMsg::GetProposals { skip, limit, descending }
					)?
				)?
			)
		}
		pub fn query_user_stats(
			env_deps: &(Env, SeiMockEnvDeps),
			user: Addr
		) -> Result<Vec<CourtUserStatsJsonable>, CourtContractError> {
			let env = env_deps.0.clone();
			Ok(
				from_json(
					crate::contract::query(
						env_deps.1.as_ref().into_empty(),
						env,
						CourtQueryMsg::UserStats { user }
					)?
				)?
			)
		}
		pub fn query_user_vote_info(
			env_deps: &(Env, SeiMockEnvDeps),
			user: Addr,
			proposal_id: u32
		) -> Result<Vec<CourtUserVoteInfoJsonable>, CourtContractError> {
			let env = env_deps.0.clone();
			Ok(
				from_json(
					crate::contract::query(
						env_deps.1.as_ref().into_empty(),
						env,
						CourtQueryMsg::UserVoteInfo { user, proposal_id }
					)?
				)?
			)
		}
		pub fn query_get_users_with_active_proposals(
			env_deps: &(Env, SeiMockEnvDeps),
			after: Option<CourtQueryUserWithActiveProposal>,
			limit: Option<u32>,
			descending: bool
		) -> Result<Vec<CourtQueryUserWithActiveProposal>, CourtContractError> {
			let env = env_deps.0.clone();
			Ok(
				from_json(
					crate::contract::query(
						env_deps.1.as_ref().into_empty(),
						env,
						CourtQueryMsg::GetUsersWithActiveProposals { after, limit, descending }
					)?
				)?
			)
		}
		pub fn query_get_user_active_proposals(
			env_deps: &(Env, SeiMockEnvDeps),
			user: Addr,
			skip: Option<u32>,
			limit: Option<u32>,
			descending: bool
		) -> Result<Vec<u32>, CourtContractError> {
			let env = env_deps.0.clone();
			Ok(
				from_json(
					crate::contract::query(
						env_deps.1.as_ref().into_empty(),
						env,
						CourtQueryMsg::GetUserActiveProposals { user, skip, limit, descending }
					)?
				)?
			)
		}
		pub fn query_get_proposal_user_votes(
			env_deps: &(Env, SeiMockEnvDeps),
			proposal_id: u32,
			after: Option<Addr>,
			limit: Option<u32>,
			descending: bool
		) -> Result<Vec<CourtQueryResponseUserVote>, CourtContractError> {
			let env = env_deps.0.clone();
			Ok(
				from_json(
					crate::contract::query(
						env_deps.1.as_ref().into_empty(),
						env,
						CourtQueryMsg::GetProposalUserVotes { proposal_id, after, limit, descending }
					)?
				)?
			)
		}
	

		pub fn assert_only_authorized_instruction(
			env_deps: &mut (Env, SeiMockEnvDeps),
			msg_info: Option<MessageInfo>,
			msg: CourtExecuteMsg,
			authorized_users: &[&str],
			unauthorized_users: &[&str],
		) {

		}
		pub fn assert_unfunded_instruction(
			env_deps: &mut (Env, SeiMockEnvDeps),
			msg_info: Option<MessageInfo>,
			msg: CourtExecuteMsg
		) {
			let vote_shares_denom = query_denom(env_deps).unwrap().votes;
			let mut msg_info = msg_info.unwrap_or(
				MessageInfo {
					sender: Addr::unchecked(RANDOM_ACCOUNT_5),
					funds: vec![]
				}
			);
			msg_info.funds.push(coin(31337, "usei"));
			let execute_response = execute(
				env_deps,
				Some(msg_info.clone()),
				msg.clone()
			);
			assert!(execute_response.is_err_and(|err| {
				// Typo won't be fixed until we upgrade to cosmwasm-std 2.x
				err.to_string().contains("does no accept funds") ||
				err.to_string().contains("does not accept funds")
			}));
			msg_info.funds.pop();
			msg_info.funds.push(coin(31337, &vote_shares_denom));
			let execute_response = execute(
				env_deps,
				Some(msg_info.clone()),
				msg.clone()
			);
			assert!(execute_response.is_err_and(|err| {
				// Typo won't be fixed until we upgrade to cosmwasm-std 2.x
				err.to_string().contains("does no accept funds") ||
				err.to_string().contains("does not accept funds")
			}));
			msg_info.funds.push(coin(31337, &vote_shares_denom));
			let execute_response = execute(
				env_deps,
				Some(msg_info.clone()),
				msg.clone()
			);
			assert!(execute_response.is_err_and(|err| {
				// Typo won't be fixed until we upgrade to cosmwasm-std 2.x
				err.to_string().contains("does no accept funds") ||
				err.to_string().contains("does not accept funds")
			}));
		}
	}

	#[test]
	pub fn instantiate() {
		let mut env_deps = new_global_env();
		let contract_addr = &env_deps.0.contract.address.clone();
		let vote_shares_denom = format!("factory/{}/votes", contract_addr);

		let instantiate_response = helpers::instantiate(
			&mut env_deps,
			Some(
				MessageInfo {
					sender: Addr::unchecked(ADMIN_ACCOUNT),
					funds: vec![coin(1337, "usei")]
				}
			),
			None
		);

		// instantiate is unfunded
		assert!(instantiate_response.is_err_and(|err| {
			// Typo won't be fixed until we upgrade to cosmwasm-std 2.x
			err.to_string().contains("does no accept funds") ||
			err.to_string().contains("does not accept funds")
		}));
		let instantiate_response = helpers::instantiate(
			&mut env_deps,
			Some(
				MessageInfo {
					sender: Addr::unchecked(RANDOM_ACCOUNT_1),
					funds: vec![]
				}
			),
			Some(
				CourtInstantiateMsg {
					admin: Addr::unchecked(RANDOM_ACCOUNT_2),
					shares_mint_amount: 1404438u128.into(),
					shares_mint_receiver: Addr::unchecked(RANDOM_ACCOUNT_3),
					minimum_vote_proposal_percent: 27,
					minimum_vote_turnout_percent: 20,
					minimum_vote_pass_percent: 28,
					max_proposal_expiry_time_seconds: 3428,
					execution_expiry_time_seconds: 2506,
					vote_share_name: "Test vote tokens".into(),
					vote_share_symbol: "TVT".into(),
					vote_share_description: "Will everyone named Mike please stand".into()
				}
			)
		).unwrap();

		// Adheres to cw2
		assert!(
			get_contract_version(&env_deps.1.storage).is_ok_and(|info| {
				info.eq(&ContractVersion {
					contract: "court-coordinator-contract".into(),
					version: env!("CARGO_PKG_VERSION").into()
				})
			})
		);
		
		// Config is what's applied
		assert!(
			helpers::query_config(&env_deps).is_ok_and(|config| {
				config.eq(&CourtAppConfigJsonable {
					allow_new_proposals: true,
					minimum_vote_proposal_percent: 27,
					minimum_vote_turnout_percent: 20,
					minimum_vote_pass_percent: 28,
					max_proposal_expiry_time_seconds: 3428,
					execution_expiry_time_seconds: 2506,
					last_config_change_timestamp_ms: env_deps.0.block.time.millis(),
					admin: Addr::unchecked(RANDOM_ACCOUNT_2)
				})
			})
		);

		// Queried votes denom is correct
		assert!(
			helpers::query_denom(&env_deps).is_ok_and(|denoms| {
				denoms.votes == vote_shares_denom
			})
		);

		// Creates the votes denom
		assert!(
			instantiate_response.messages.iter().any(|sub_msg| {
				match &sub_msg.msg {
					cosmwasm_std::CosmosMsg::Custom(sub_msg) => {
						match sub_msg {
							sei_cosmwasm::SeiMsg::CreateDenom { subdenom } => {
								subdenom == "votes"
							},
							_ => {
								false
							}
						}
					}
					_ => false
				}
			})
		);

		// Mints the amount specified
		assert!(
			instantiate_response.messages.iter().any(|sub_msg| {
				match &sub_msg.msg {
					cosmwasm_std::CosmosMsg::Custom(sub_msg) => {
						match sub_msg {
							sei_cosmwasm::SeiMsg::MintTokens { amount } => {
								*amount == coin(1404438u128, &vote_shares_denom)
							},
							_ => {
								false
							}
						}
					}
					_ => false
				}
			})
		);

		// Sends the amount of tokens minted to the user specified
		assert!(
			instantiate_response.messages.iter().any(|sub_msg| {
				match &sub_msg.msg {
					cosmwasm_std::CosmosMsg::Bank(sub_msg) => {
						*sub_msg == cosmwasm_std::BankMsg::Send {
							to_address: RANDOM_ACCOUNT_3.into(),
							amount: vec![coin(1404438u128, &vote_shares_denom)]
						}
					}
					_ => false
				}
			})
		);

		// Registers the metadata
		assert!(
			instantiate_response.messages.iter().any(|sub_msg| {
				match &sub_msg.msg {
					cosmwasm_std::CosmosMsg::Custom(sub_msg) => {
						match sub_msg {
							sei_cosmwasm::SeiMsg::SetMetadata { metadata } => {
								*metadata == sei_cosmwasm::Metadata {
									description: "Will everyone named Mike please stand".into(),
									denom_units: vec![
										sei_cosmwasm::DenomUnit {
											denom: vote_shares_denom.clone(),
											exponent: 0,
											aliases: vec!["utvt".into(), "microtvt".into()],
										},
										sei_cosmwasm::DenomUnit {
											denom: "mtvt".into(),
											exponent: 3,
											aliases: vec!["millitvt".into()],
										},
										sei_cosmwasm::DenomUnit {
											denom: "tvt".into(),
											exponent: 6,
											aliases: vec![],
										}
									],
									base: vote_shares_denom.clone(),
									display: "tvt".into(),
									name: "Test vote tokens".into(),
									symbol: "TVT".into()
								}
							},
							_ => {
								false
							}
						}
					}
					_ => false
				}
			})
		);
	}

	#[test]
	pub fn admin_change_configs() {
		let mut env_deps = new_global_env();
		let contract_addr = &env_deps.0.contract.address.clone();
		let vote_shares_denom = format!("factory/{}/votes", contract_addr);

		helpers::instantiate(
			&mut env_deps,
			None,
			None
		).unwrap();
		let exec_msg = CourtExecuteMsg::Admin(
			CourtAdminExecuteMsg::ChangeConfig {
				minimum_vote_proposal_percent: None,
				minimum_vote_turnout_percent: None,
				minimum_vote_pass_percent: None,
				max_proposal_expiry_time_seconds: None,
				execution_expiry_time_seconds: None
			}
		);
	}
}
