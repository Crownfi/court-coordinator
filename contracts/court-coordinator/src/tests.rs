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
const RANDOM_EVM_ACCOUNT_1: &str = "0x69207370696c6C206d79206472696E6b20545F54";

mod contract_execution {
	use super::*;
	use cosmwasm_std::{coin, MessageInfo};
	use cw2::{get_contract_version, ContractVersion};
	use helpers::{get_known_vote_supply, new_env_and_instantiate};
	use crate::{msg::*, proposed_msg::ProposedCourtMsgJsonable, state::{app::{CourtAppConfigJsonable, TransactionProposalInfoJsonable, TransactionProposalStatus, TransactionProposalExecutionStatus}, user::{CourtUserStatsJsonable, CourtUserVoteInfoJsonable, CourtUserVoteStatus}}};
	mod helpers {
		

use super::*;
		pub fn new_env_and_instantiate(msg: Option<CourtInstantiateMsg>) -> MutexGuard<'static, (Env, SeiMockEnvDeps)> {
			let mut env_deps = new_global_env();
			instantiate(&mut env_deps, None, msg.clone()).unwrap();
			assert_eq!(
				query_config(&env_deps).unwrap(),
				// Config is what's applied
				CourtAppConfigJsonable {
					allow_new_proposals: true,
					minimum_vote_proposal_percent: msg.as_ref().map(|msg| {msg.minimum_vote_proposal_percent}).unwrap_or(10),
					minimum_vote_turnout_percent: msg.as_ref().map(|msg| {msg.minimum_vote_turnout_percent}).unwrap_or(20),
					minimum_vote_pass_percent: msg.as_ref().map(|msg| {msg.minimum_vote_pass_percent}).unwrap_or(50),
					max_proposal_expiry_time_seconds: msg.as_ref().map(|msg| {msg.max_proposal_expiry_time_seconds}).unwrap_or(7200),
					execution_expiry_time_seconds: msg.as_ref().map(|msg| {msg.execution_expiry_time_seconds}).unwrap_or(3600),
					last_config_change_timestamp_ms: env_deps.0.block.time.millis(),
					admin: Addr::unchecked(ADMIN_ACCOUNT)
				}
			);
			env_deps
		}

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
					shares_mint_receiver: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
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
			user: &str
		) -> Result<CourtUserStatsJsonable, CourtContractError> {
			let env = env_deps.0.clone();
			Ok(
				from_json(
					crate::contract::query(
						env_deps.1.as_ref().into_empty(),
						env,
						CourtQueryMsg::UserStats { user: Addr::unchecked(user) }
					)?
				)?
			)
		}
		pub fn query_user_vote_info(
			env_deps: &(Env, SeiMockEnvDeps),
			user: &str,
			proposal_id: u32
		) -> Result<CourtUserVoteInfoJsonable, CourtContractError> {
			let env = env_deps.0.clone();
			Ok(
				from_json(
					crate::contract::query(
						env_deps.1.as_ref().into_empty(),
						env,
						CourtQueryMsg::UserVoteInfo { user: Addr::unchecked(user), proposal_id }
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
			user: &str,
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
						CourtQueryMsg::GetUserActiveProposals {
							user: Addr::unchecked(user),
							skip,
							limit,
							descending
						}
					)?
				)?
			)
		}
		pub fn query_get_proposal_user_votes(
			env_deps: &(Env, SeiMockEnvDeps),
			proposal_id: u32,
			after: Option<&str>,
			limit: Option<u32>,
			descending: bool
		) -> Result<Vec<CourtQueryResponseUserVote>, CourtContractError> {
			let env = env_deps.0.clone();
			Ok(
				from_json(
					crate::contract::query(
						env_deps.1.as_ref().into_empty(),
						env,
						CourtQueryMsg::GetProposalUserVotes {
							proposal_id,
							after: after.map(|addr| {Addr::unchecked(addr)}),
							limit,
							descending
						}
					)?
				)?
			)
		}

		pub fn assert_only_authorized_instruction(
			env_deps: &mut (Env, SeiMockEnvDeps),
			funds: &[Coin],
			unauthorized_users: &[&str],
			authorized_users: &[&str],
			msg: CourtExecuteMsg
		) {
			for user_str in unauthorized_users.iter() {
				let execute_response = execute(
					env_deps,
					Some(MessageInfo {
						sender: Addr::unchecked(user_str.to_string()),
						funds: funds.into(),
					}),
					msg.clone()
				);
				assert!(execute_response.is_err_and(|err| {
					// Typo won't be fixed until we upgrade to cosmwasm-std 2.x
					err.to_string().starts_with("Permission denied:")
				}));
			}
			for user_str in authorized_users.iter() {
				let execute_response = execute(
					env_deps,
					Some(MessageInfo {
						sender: Addr::unchecked(user_str.to_string()),
						funds: funds.into(),
					}),
					msg.clone()
				);
				assert!(execute_response.is_ok());
			}
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
			assert!(execute_response.is_err());
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
			assert!(execute_response.is_err());
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
			assert!(execute_response.is_err());
			assert!(execute_response.is_err_and(|err| {
				// Typo won't be fixed until we upgrade to cosmwasm-std 2.x
				err.to_string().contains("does no accept funds") ||
				err.to_string().contains("does not accept funds")
			}));
		}

		pub fn assert_must_pay(
			env_deps: &mut (Env, SeiMockEnvDeps),
			sender: &str,
			msg: CourtExecuteMsg,
			accepted_denom: &str
		) {
			let random_denom_1 = format!("factory/{}/ayylmao", RANDOM_CONTRACT);			
			let random_denom_2 = format!("factory/{}/ayylmao", env_deps.0.contract.address);

			let mut msg_info = MessageInfo {
				sender: Addr::unchecked(sender),
				funds: vec![]
			};
			
			let execute_response = execute(
				env_deps,
				Some(msg_info.clone()),
				msg.clone()
			);
			assert!(execute_response.is_err());
			assert!(execute_response.is_err_and(|err| {
				err.to_string().contains("No funds sent")
			}));

			msg_info.funds.push(coin(0, accepted_denom));
			let execute_response = execute(
				env_deps,
				Some(msg_info.clone()),
				msg.clone()
			);
			assert!(execute_response.is_err());
			assert!(execute_response.is_err_and(|err| {
				err.to_string().contains("No funds sent")
			}));

			msg_info.funds.pop();

			msg_info.funds.push(coin(31337u128, &random_denom_1));
			let execute_response = execute(
				env_deps,
				Some(msg_info.clone()),
				msg.clone()
			);
			assert!(execute_response.is_err());
			assert!(execute_response.is_err_and(|err| {
				// Must send reserve token
				err.to_string().contains("Must send") && err.to_string().contains(accepted_denom)
			}));


			msg_info.funds.pop();

			msg_info.funds.push(coin(31337u128, &random_denom_2));
			let execute_response = execute(
				env_deps,
				Some(msg_info.clone()),
				msg.clone()
			);
			assert!(execute_response.is_err());
			assert!(execute_response.is_err_and(|err| {
				// Must send reserve token
				err.to_string().contains("Must send") && err.to_string().contains(accepted_denom)
			}));

			msg_info.funds.push(coin(31337u128, &random_denom_1));
			let execute_response = execute(
				env_deps,
				Some(msg_info.clone()),
				msg.clone()
			);
			assert!(execute_response.is_err());
			assert!(execute_response.is_err_and(|err| {
				// Must send reserve token
				err.to_string().contains("more than one denomination")
			}));

			msg_info.funds.pop();
			msg_info.funds.pop();
			msg_info.funds.push(coin(1337u128, accepted_denom));
			msg_info.funds.push(coin(1337u128, &random_denom_1));

			let execute_response = execute(
				env_deps,
				Some(msg_info.clone()),
				msg.clone()
			);
			assert!(execute_response.is_err());
			assert!(execute_response.is_err_and(|err| {
				// Must send reserve token
				err.to_string().contains("more than one denomination")
			}));

			msg_info.funds.pop();

			let execute_response = execute(
				env_deps,
				Some(msg_info.clone()),
				msg.clone()
			);
			assert!(execute_response.is_ok());
		}

		// Sei's cosmwasm is too outdated for contracts to know the total supply of a token. So... workaround!
		pub fn set_known_vote_supply(
			env_deps: &mut (Env, SeiMockEnvDeps),
			amount: u128
		) {
			use cosmwasm_std::Storage;

			let vote_shares_denom = format!("factory/{}/votes", &env_deps.0.contract.address);
			env_deps.1.storage.set(vote_shares_denom.as_bytes(), &amount.to_le_bytes());
		}

		// Contract stores the number of tokens it minted here
		pub fn get_known_vote_supply(
			env_deps: &(Env, SeiMockEnvDeps)
		) -> u128 {
			use cosmwasm_std::Storage;

			let vote_shares_denom = format!("factory/{}/votes", &env_deps.0.contract.address);
			env_deps.1.storage.get(
				vote_shares_denom.as_bytes()
			).and_then(|bytes| {
				Some(<u128>::from_le_bytes(bytes.try_into().ok()?))
			}).unwrap_or_default()
		}

		pub fn execute_change_admin(
			env_deps: &mut (Env, SeiMockEnvDeps),
			sender: Option<&str>,
			new_admin: &str
		) {
			execute(
				env_deps,
				Some(
					MessageInfo { sender: Addr::unchecked(sender.unwrap_or(ADMIN_ACCOUNT)), funds: vec![] }
				),
				CourtExecuteMsg::Admin(CourtAdminExecuteMsg::ChangeAdmin { admin: Addr::unchecked(new_admin) })
			).unwrap();
		}

		pub fn execute_allow_new_proposals(
			env_deps: &mut (Env, SeiMockEnvDeps),
			sender: Option<&str>,
			allowed: bool
		) {
			execute(
				env_deps,
				Some(
					MessageInfo { sender: Addr::unchecked(sender.unwrap_or(ADMIN_ACCOUNT)), funds: vec![] }
				),
				CourtExecuteMsg::Admin(CourtAdminExecuteMsg::AllowNewProposals { allowed })
			).unwrap();
		}

		pub fn execute_stake_votes(
			env_deps: &mut (Env, SeiMockEnvDeps),
			sender: &str,
			amount: u128
		) {
			let vote_shares_denom = query_denom(&env_deps).unwrap().votes;
			let previous_stake_amount = query_user_stats(&env_deps, sender).unwrap().staked_votes.u128();
			execute(
				env_deps,
				Some(
					MessageInfo { sender: Addr::unchecked(sender), funds: vec![coin(amount, &vote_shares_denom)] }
				),
				CourtExecuteMsg::Stake
			).unwrap();

			let stake_amount_increase = query_user_stats(&env_deps, sender).unwrap().staked_votes.u128()
				.saturating_sub(previous_stake_amount);
			assert_eq!(amount, stake_amount_increase);
		}
	
		pub fn execute_propose_transaction(
			env_deps: &mut (Env, SeiMockEnvDeps),
			sender: &str,
			msgs: Vec<ProposedCourtMsgJsonable>,
			expiry_time_seconds: u32
		) {
			let user_staked_votes = query_user_stats(&env_deps, sender).unwrap().staked_votes;
			let new_proposal_id = query_proposal_amount(&env_deps).unwrap();
			let execute_result = execute(
				env_deps,
				Some(
					MessageInfo { sender: Addr::unchecked(sender), funds: vec![] }
				),
				CourtExecuteMsg::ProposeTransaction { msgs, expiry_time_seconds }
			).unwrap();

			assert_eq!(execute_result.messages.len(), 0);
			assert_eq!(execute_result.events, vec![
				cosmwasm_std::Event::new("proposal")
					.add_attribute("proposal_id", new_proposal_id.to_string())
					.add_attribute("proposer", sender),
				cosmwasm_std::Event::new("vote")
					.add_attribute("proposal_id", new_proposal_id.to_string())
					.add_attribute("voter", sender)
					.add_attribute("votes", user_staked_votes)
					.add_attribute("vote", "approve")
			]);
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

		assert_eq!(get_known_vote_supply(&env_deps), 1404438u128);

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
	pub fn admin_change_config_unfunded_check() {
		let mut env_deps = new_env_and_instantiate(None);

		helpers::assert_unfunded_instruction(
			&mut env_deps,
			Some(MessageInfo { sender: Addr::unchecked(ADMIN_ACCOUNT), funds: vec![] }),
			CourtExecuteMsg::Admin(
				CourtAdminExecuteMsg::ChangeConfig {
					minimum_vote_proposal_percent: None,
					minimum_vote_turnout_percent: None,
					minimum_vote_pass_percent: None,
					max_proposal_expiry_time_seconds: None,
					execution_expiry_time_seconds: None
				}
			)
		)
	}

	#[test]
	pub fn admin_change_config_authorized_check() {
		let mut env_deps = new_env_and_instantiate(None);
		
		helpers::assert_only_authorized_instruction(
			&mut env_deps,
			&[],
			&[
				RANDOM_ACCOUNT_1,
				RANDOM_ACCOUNT_2,
				RANDOM_ACCOUNT_3,
				RANDOM_ACCOUNT_4,
				RANDOM_ACCOUNT_5,
				SHARES_HOLDER_ACCOUNT_1,
				SHARES_HOLDER_ACCOUNT_2,
				SHARES_HOLDER_ACCOUNT_3,
				SHARES_HOLDER_ACCOUNT_4,
				SHARES_HOLDER_ACCOUNT_5
			],
			&[ADMIN_ACCOUNT],
			CourtExecuteMsg::Admin(
				CourtAdminExecuteMsg::ChangeConfig {
					minimum_vote_proposal_percent: None,
					minimum_vote_turnout_percent: None,
					minimum_vote_pass_percent: None,
					max_proposal_expiry_time_seconds: None,
					execution_expiry_time_seconds: None
				}
			)
		);
	}

	#[test]
	pub fn admin_change_config_correct() {
		// Nothing changes if nothing is specified to change
		let mut env_deps = helpers::new_env_and_instantiate(None);
		helpers::execute(
			&mut env_deps,
			Some(
				MessageInfo {
					sender: Addr::unchecked(ADMIN_ACCOUNT),
					funds: vec![]
				}
			),
			CourtExecuteMsg::Admin(
				CourtAdminExecuteMsg::ChangeConfig {
					minimum_vote_proposal_percent: None,
					minimum_vote_turnout_percent: None,
					minimum_vote_pass_percent: None,
					max_proposal_expiry_time_seconds: None,
					execution_expiry_time_seconds: None
				}
			)
		).unwrap();
		assert_eq!(
			helpers::query_config(&env_deps).unwrap(),
			// Config is what's applied
			CourtAppConfigJsonable {
				allow_new_proposals: true,
				minimum_vote_proposal_percent: 10,
				minimum_vote_turnout_percent: 20,
				minimum_vote_pass_percent: 50,
				max_proposal_expiry_time_seconds: 7200,
				execution_expiry_time_seconds: 3600,
				last_config_change_timestamp_ms: env_deps.0.block.time.millis(),
				admin: Addr::unchecked(ADMIN_ACCOUNT)
			}
		);

		drop(env_deps); // Must drop otherwise we deadlock
		let mut env_deps = helpers::new_env_and_instantiate(None);
		helpers::execute(
			&mut env_deps,
			Some(
				MessageInfo {
					sender: Addr::unchecked(ADMIN_ACCOUNT),
					funds: vec![]
				}
			),
			CourtExecuteMsg::Admin(
				CourtAdminExecuteMsg::ChangeConfig {
					minimum_vote_proposal_percent: Some(69),
					minimum_vote_turnout_percent: None,
					minimum_vote_pass_percent: None,
					max_proposal_expiry_time_seconds: None,
					execution_expiry_time_seconds: None
				}
			)
		).unwrap();
		assert_eq!(
			helpers::query_config(&env_deps).unwrap(),
			// Config is what's applied
			CourtAppConfigJsonable {
				allow_new_proposals: true,
				minimum_vote_proposal_percent: 69,
				minimum_vote_turnout_percent: 20,
				minimum_vote_pass_percent: 50,
				max_proposal_expiry_time_seconds: 7200,
				execution_expiry_time_seconds: 3600,
				last_config_change_timestamp_ms: env_deps.0.block.time.millis(),
				admin: Addr::unchecked(ADMIN_ACCOUNT)
			}
		);

		drop(env_deps); // Must drop otherwise we deadlock
		let mut env_deps = helpers::new_env_and_instantiate(None);
		helpers::execute(
			&mut env_deps,
			Some(
				MessageInfo {
					sender: Addr::unchecked(ADMIN_ACCOUNT),
					funds: vec![]
				}
			),
			CourtExecuteMsg::Admin(
				CourtAdminExecuteMsg::ChangeConfig {
					minimum_vote_proposal_percent: None,
					minimum_vote_turnout_percent: Some(69),
					minimum_vote_pass_percent: None,
					max_proposal_expiry_time_seconds: None,
					execution_expiry_time_seconds: None
				}
			)
		).unwrap();
		assert_eq!(
			helpers::query_config(&env_deps).unwrap(),
			// Config is what's applied
			CourtAppConfigJsonable {
				allow_new_proposals: true,
				minimum_vote_proposal_percent: 10,
				minimum_vote_turnout_percent: 69,
				minimum_vote_pass_percent: 50,
				max_proposal_expiry_time_seconds: 7200,
				execution_expiry_time_seconds: 3600,
				last_config_change_timestamp_ms: env_deps.0.block.time.millis(),
				admin: Addr::unchecked(ADMIN_ACCOUNT)
			}
		);

		drop(env_deps); // Must drop otherwise we deadlock
		let mut env_deps = helpers::new_env_and_instantiate(None);
		helpers::execute(
			&mut env_deps,
			Some(
				MessageInfo {
					sender: Addr::unchecked(ADMIN_ACCOUNT),
					funds: vec![]
				}
			),
			CourtExecuteMsg::Admin(
				CourtAdminExecuteMsg::ChangeConfig {
					minimum_vote_proposal_percent: None,
					minimum_vote_turnout_percent: None,
					minimum_vote_pass_percent: Some(69),
					max_proposal_expiry_time_seconds: None,
					execution_expiry_time_seconds: None
				}
			)
		).unwrap();
		assert_eq!(
			helpers::query_config(&env_deps).unwrap(),
			// Config is what's applied
			CourtAppConfigJsonable {
				allow_new_proposals: true,
				minimum_vote_proposal_percent: 10,
				minimum_vote_turnout_percent: 20,
				minimum_vote_pass_percent: 69,
				max_proposal_expiry_time_seconds: 7200,
				execution_expiry_time_seconds: 3600,
				last_config_change_timestamp_ms: env_deps.0.block.time.millis(),
				admin: Addr::unchecked(ADMIN_ACCOUNT)
			}
		);

		drop(env_deps); // Must drop otherwise we deadlock
		let mut env_deps = helpers::new_env_and_instantiate(None);
		helpers::execute(
			&mut env_deps,
			Some(
				MessageInfo {
					sender: Addr::unchecked(ADMIN_ACCOUNT),
					funds: vec![]
				}
			),
			CourtExecuteMsg::Admin(
				CourtAdminExecuteMsg::ChangeConfig {
					minimum_vote_proposal_percent: None,
					minimum_vote_turnout_percent: None,
					minimum_vote_pass_percent: None,
					max_proposal_expiry_time_seconds: Some(69),
					execution_expiry_time_seconds: None
				}
			)
		).unwrap();
		assert_eq!(
			helpers::query_config(&env_deps).unwrap(),
			// Config is what's applied
			CourtAppConfigJsonable {
				allow_new_proposals: true,
				minimum_vote_proposal_percent: 10,
				minimum_vote_turnout_percent: 20,
				minimum_vote_pass_percent: 50,
				max_proposal_expiry_time_seconds: 69,
				execution_expiry_time_seconds: 3600,
				last_config_change_timestamp_ms: env_deps.0.block.time.millis(),
				admin: Addr::unchecked(ADMIN_ACCOUNT)
			}
		);

		drop(env_deps); // Must drop otherwise we deadlock
		let mut env_deps = helpers::new_env_and_instantiate(None);
		helpers::execute(
			&mut env_deps,
			Some(
				MessageInfo {
					sender: Addr::unchecked(ADMIN_ACCOUNT),
					funds: vec![]
				}
			),
			CourtExecuteMsg::Admin(
				CourtAdminExecuteMsg::ChangeConfig {
					minimum_vote_proposal_percent: None,
					minimum_vote_turnout_percent: None,
					minimum_vote_pass_percent: None,
					max_proposal_expiry_time_seconds: None,
					execution_expiry_time_seconds: Some(69)
				}
			)
		).unwrap();
		assert_eq!(
			helpers::query_config(&env_deps).unwrap(),
			// Config is what's applied
			CourtAppConfigJsonable {
				allow_new_proposals: true,
				minimum_vote_proposal_percent: 10,
				minimum_vote_turnout_percent: 20,
				minimum_vote_pass_percent: 50,
				max_proposal_expiry_time_seconds: 7200,
				execution_expiry_time_seconds: 69,
				last_config_change_timestamp_ms: env_deps.0.block.time.millis(),
				admin: Addr::unchecked(ADMIN_ACCOUNT)
			}
		);
		env_deps.0.block.time = env_deps.0.block.time.plus_minutes(1);

		helpers::execute(
			&mut env_deps,
			Some(
				MessageInfo {
					sender: Addr::unchecked(ADMIN_ACCOUNT),
					funds: vec![]
				}
			),
			CourtExecuteMsg::Admin(
				CourtAdminExecuteMsg::ChangeConfig {
					minimum_vote_proposal_percent: None,
					minimum_vote_turnout_percent: None,
					minimum_vote_pass_percent: Some(69),
					max_proposal_expiry_time_seconds: None,
					execution_expiry_time_seconds: None
				}
			)
		).unwrap();
		assert_eq!(
			helpers::query_config(&env_deps).unwrap(),
			// Config is what's applied
			CourtAppConfigJsonable {
				allow_new_proposals: true,
				minimum_vote_proposal_percent: 10,
				minimum_vote_turnout_percent: 20,
				minimum_vote_pass_percent: 69,
				max_proposal_expiry_time_seconds: 7200,
				execution_expiry_time_seconds: 69,
				last_config_change_timestamp_ms: env_deps.0.block.time.millis(),
				admin: Addr::unchecked(ADMIN_ACCOUNT)
			}
		);
	}

	#[test]
	pub fn admin_change_config_only_when_no_pending_proposals() {
		let mut env_deps = helpers::new_env_and_instantiate(None);
		
		// Sanity tests for current config we're testing against
		assert_eq!(helpers::query_config(&env_deps).unwrap().minimum_vote_proposal_percent, 10);
		assert_eq!(helpers::query_config(&env_deps).unwrap().max_proposal_expiry_time_seconds, 7200);
		assert_eq!(helpers::get_known_vote_supply(&env_deps), 1000000);

		helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 100000);
		helpers::execute_propose_transaction(
			&mut env_deps,
			SHARES_HOLDER_ACCOUNT_1,
			vec![ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_ACCOUNT_1.into(),
				denom: "usei".into(),
				amount: 1337u128.into()
			}],
			1200
		);
		let execute_response = helpers::execute(
			&mut env_deps,
			Some(
				MessageInfo { sender: Addr::unchecked(ADMIN_ACCOUNT), funds: vec![] }
			),
			CourtExecuteMsg::Admin(
				CourtAdminExecuteMsg::ChangeConfig {
					minimum_vote_proposal_percent: None,
					minimum_vote_turnout_percent: None,
					minimum_vote_pass_percent: None,
					max_proposal_expiry_time_seconds: None,
					execution_expiry_time_seconds: None
				}
			)
		);
		assert!(execute_response.is_err_and(|err| {
			err.to_string().contains("votes must not be tied to any proposals")
		}));
		env_deps.0.block.time = env_deps.0.block.time.plus_days(69);

		helpers::execute(
			&mut env_deps,
			None,
			CourtExecuteMsg::DeactivateVotes { user: Some(Addr::unchecked(SHARES_HOLDER_ACCOUNT_1)), limit: None }
		).unwrap();
		helpers::execute(
			&mut env_deps,
			Some(
				MessageInfo { sender: Addr::unchecked(ADMIN_ACCOUNT), funds: vec![] }
			),
			CourtExecuteMsg::Admin(
				CourtAdminExecuteMsg::ChangeConfig {
					minimum_vote_proposal_percent: None,
					minimum_vote_turnout_percent: None,
					minimum_vote_pass_percent: None,
					max_proposal_expiry_time_seconds: None,
					execution_expiry_time_seconds: None
				}
			)
		).unwrap();
	}

	#[test]
	pub fn admin_change_admin_unfunded_check() {
		let mut env_deps = new_env_and_instantiate(None);

		helpers::assert_unfunded_instruction(
			&mut env_deps,
			Some(MessageInfo { sender: Addr::unchecked(ADMIN_ACCOUNT), funds: vec![] }),
			CourtExecuteMsg::Admin(
				CourtAdminExecuteMsg::ChangeAdmin { admin: Addr::unchecked(RANDOM_ACCOUNT_2) }
			)
		);
	}

	#[test]
	pub fn admin_change_admin_authorized_check() {
		let mut env_deps = new_env_and_instantiate(None);
		
		helpers::assert_only_authorized_instruction(
			&mut env_deps,
			&[],
			&[
				RANDOM_ACCOUNT_1,
				RANDOM_ACCOUNT_2,
				RANDOM_ACCOUNT_3,
				RANDOM_ACCOUNT_4,
				RANDOM_ACCOUNT_5,
				SHARES_HOLDER_ACCOUNT_1,
				SHARES_HOLDER_ACCOUNT_2,
				SHARES_HOLDER_ACCOUNT_3,
				SHARES_HOLDER_ACCOUNT_4,
				SHARES_HOLDER_ACCOUNT_5
			],
			&[ADMIN_ACCOUNT],
			CourtExecuteMsg::Admin(
				CourtAdminExecuteMsg::ChangeAdmin { admin: Addr::unchecked(RANDOM_ACCOUNT_2) }
			)
		);
	}

	#[test]
	pub fn admin_change_admin_cannot_be_self_while_voting_disabled() {
		let mut env_deps = new_env_and_instantiate(None);
		let contract_addr = &env_deps.0.contract.address.clone();
		helpers::execute_allow_new_proposals(
			&mut env_deps,
			None,
			false
		);
		let execute_response = helpers::execute(
			&mut env_deps,
			Some(
				MessageInfo { sender: Addr::unchecked(ADMIN_ACCOUNT), funds: vec![] }
			),
			CourtExecuteMsg::Admin(
				CourtAdminExecuteMsg::ChangeAdmin { admin: contract_addr.clone() }
			)
		);
		assert!(execute_response.is_err());
		assert!(execute_response.is_err_and(|err| {
			err.to_string().contains("may result in this contract becoming unusable")
		}));
	}

	#[test]
	pub fn admin_change_admin_new_guy_can_do_things() {
		let mut env_deps = new_env_and_instantiate(None);
		helpers::execute_change_admin(
			&mut env_deps,
			None,
			RANDOM_ACCOUNT_1
		);
		helpers::assert_only_authorized_instruction(
			&mut env_deps,
			&[],
			&[
				ADMIN_ACCOUNT,
				RANDOM_ACCOUNT_2,
				RANDOM_ACCOUNT_3,
				RANDOM_ACCOUNT_4,
				RANDOM_ACCOUNT_5,
				SHARES_HOLDER_ACCOUNT_1,
				SHARES_HOLDER_ACCOUNT_2,
				SHARES_HOLDER_ACCOUNT_3,
				SHARES_HOLDER_ACCOUNT_4,
				SHARES_HOLDER_ACCOUNT_5
			],
			&[RANDOM_ACCOUNT_1],
			CourtExecuteMsg::Admin(
				CourtAdminExecuteMsg::ChangeConfig {
					minimum_vote_proposal_percent: Some(69),
					minimum_vote_turnout_percent: Some(69),
					minimum_vote_pass_percent: Some(69),
					max_proposal_expiry_time_seconds: None,
					execution_expiry_time_seconds: None
				}
			)
		);
	}
	
	#[test]
	pub fn admin_disallow_new_proposals_unfunded_check() {
		let mut env_deps = new_env_and_instantiate(None);

		helpers::assert_unfunded_instruction(
			&mut env_deps,
			Some(MessageInfo { sender: Addr::unchecked(ADMIN_ACCOUNT), funds: vec![] }),
			CourtExecuteMsg::Admin(
				CourtAdminExecuteMsg::AllowNewProposals { allowed: false }
			)
		);
	}

	#[test]
	pub fn admin_disallow_new_proposals_authorized_check() {
		let mut env_deps = new_env_and_instantiate(None);
		
		helpers::assert_only_authorized_instruction(
			&mut env_deps,
			&[],
			&[
				RANDOM_ACCOUNT_1,
				RANDOM_ACCOUNT_2,
				RANDOM_ACCOUNT_3,
				RANDOM_ACCOUNT_4,
				RANDOM_ACCOUNT_5,
				SHARES_HOLDER_ACCOUNT_1,
				SHARES_HOLDER_ACCOUNT_2,
				SHARES_HOLDER_ACCOUNT_3,
				SHARES_HOLDER_ACCOUNT_4,
				SHARES_HOLDER_ACCOUNT_5
			],
			&[ADMIN_ACCOUNT],
			CourtExecuteMsg::Admin(
				CourtAdminExecuteMsg::AllowNewProposals { allowed: false }
			)
		);
	}

	#[test]
	pub fn admin_disallow_new_proposals_cant_be_self() {
		let mut env_deps = new_env_and_instantiate(None);
		let contract_addr = &env_deps.0.contract.address.clone();
		helpers::execute_change_admin(
			&mut env_deps,
			None,
			contract_addr.as_str()
		);

		let execute_response = helpers::execute(
			&mut env_deps,
			Some(
				MessageInfo { sender: contract_addr.clone(), funds: vec![] }
			),
			CourtExecuteMsg::Admin(
				CourtAdminExecuteMsg::AllowNewProposals { allowed: false }
			)
		);
		assert!(execute_response.is_err());
		assert!(execute_response.is_err_and(|err| {
			err.to_string().contains("may result in this contract becoming unusable")
		}));
	}

	#[test]
	pub fn admin_disallow_new_proposals_blocks_proposals() {
		let mut env_deps = new_env_and_instantiate(Some(
			CourtInstantiateMsg {
				admin: Addr::unchecked(ADMIN_ACCOUNT),
				shares_mint_amount: 1000000u128.into(),
				shares_mint_receiver: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
				minimum_vote_proposal_percent: 10,
				minimum_vote_turnout_percent: 10,
				minimum_vote_pass_percent: 50,
				max_proposal_expiry_time_seconds: 86400,
				execution_expiry_time_seconds: 86400,
				vote_share_name: "Test vote token".into(),
				vote_share_symbol: "TVT".into(),
				vote_share_description: "Test vote token".into()
			}
		));
		// let vote_shares_denom = helpers::query_denom(&env_deps).unwrap().votes;
		helpers::execute_stake_votes(
			&mut env_deps,
			SHARES_HOLDER_ACCOUNT_2,
			200000u128
		);
		helpers::execute_allow_new_proposals(&mut env_deps, None, false);
		let execute_response = helpers::execute(
			&mut env_deps,
			Some(
				MessageInfo { sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2), funds: vec![] }
			),
			CourtExecuteMsg::ProposeTransaction {
				msgs: vec![
					ProposedCourtMsgJsonable::SendCoin {
						to: RANDOM_ACCOUNT_1.into(),
						denom: "usei".into(),
						amount: 31337u128.into() }
				],
				expiry_time_seconds: 3600
			}
		);
		assert!(
			execute_response.is_err_and(|err| {
				err.to_string().contains("proposals currently aren't being accepted")
			})
		);
		helpers::execute_allow_new_proposals(&mut env_deps, None, true);
		helpers::execute(
			&mut env_deps,
			Some(
				MessageInfo { sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2), funds: vec![] }
			),
			CourtExecuteMsg::ProposeTransaction {
				msgs: vec![
					ProposedCourtMsgJsonable::SendCoin {
						to: RANDOM_ACCOUNT_1.into(),
						denom: "usei".into(),
						amount: 31337u128.into() }
				],
				expiry_time_seconds: 3600
			}
		).unwrap();
	}

	#[test]
	pub fn admin_mint_shares_unfunded_check() {
		let mut env_deps = new_env_and_instantiate(None);

		helpers::assert_unfunded_instruction(
			&mut env_deps,
			Some(MessageInfo { sender: Addr::unchecked(ADMIN_ACCOUNT), funds: vec![] }),
			CourtExecuteMsg::Admin(
				CourtAdminExecuteMsg::MintShares { receiver: Addr::unchecked(RANDOM_ACCOUNT_1), amount: 31337u128.into() }
			)
		);
	}

	#[test]
	pub fn admin_mint_shares_authorized_check() {
		let mut env_deps = new_env_and_instantiate(None);
		
		helpers::assert_only_authorized_instruction(
			&mut env_deps,
			&[],
			&[
				RANDOM_ACCOUNT_1,
				RANDOM_ACCOUNT_2,
				RANDOM_ACCOUNT_3,
				RANDOM_ACCOUNT_4,
				RANDOM_ACCOUNT_5,
				SHARES_HOLDER_ACCOUNT_1,
				SHARES_HOLDER_ACCOUNT_2,
				SHARES_HOLDER_ACCOUNT_3,
				SHARES_HOLDER_ACCOUNT_4,
				SHARES_HOLDER_ACCOUNT_5
			],
			&[ADMIN_ACCOUNT],
			CourtExecuteMsg::Admin(
				CourtAdminExecuteMsg::MintShares { receiver: Addr::unchecked(RANDOM_ACCOUNT_1), amount: 31337u128.into() }
			)
		);
	}

	#[test]
	pub fn admin_mint_shares_minted_check() {
		let mut env_deps = new_env_and_instantiate(None);
		// Just a sanity check since that's what new_env_and_instantiate does by default
		assert_eq!(get_known_vote_supply(&env_deps), 1000000u128);
		let vote_shares_denom = helpers::query_denom(&env_deps).unwrap().votes;

		let execute_reponse = helpers::execute(
			&mut env_deps,
			Some(
				MessageInfo { sender: Addr::unchecked(ADMIN_ACCOUNT), funds: vec![] }
			),
			CourtExecuteMsg::Admin(
				CourtAdminExecuteMsg::MintShares { receiver: Addr::unchecked(RANDOM_ACCOUNT_2), amount: 31337u128.into() }
			)
		).unwrap();

		// Mints the amount specified
		assert!(
			execute_reponse.messages.iter().any(|sub_msg| {
				match &sub_msg.msg {
					cosmwasm_std::CosmosMsg::Custom(sub_msg) => {
						match sub_msg {
							sei_cosmwasm::SeiMsg::MintTokens { amount } => {
								*amount == coin(31337u128, &vote_shares_denom)
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
			execute_reponse.messages.iter().any(|sub_msg| {
				match &sub_msg.msg {
					cosmwasm_std::CosmosMsg::Bank(sub_msg) => {
						*sub_msg == cosmwasm_std::BankMsg::Send {
							to_address: RANDOM_ACCOUNT_2.into(),
							amount: vec![coin(31337u128, &vote_shares_denom)]
						}
					}
					_ => false
				}
			})
		);
		assert_eq!(get_known_vote_supply(&env_deps), 1031337u128);
	}

	#[test]
	pub fn user_stake_votes_correct_tokens_check() {
		let mut env_deps = new_env_and_instantiate(None);
		let contract_addr = &env_deps.0.contract.address.clone();
		let vote_shares_denom = format!("factory/{}/votes", contract_addr);

		helpers::assert_must_pay(
			&mut env_deps,
			SHARES_HOLDER_ACCOUNT_1,
			CourtExecuteMsg::Stake,
			&vote_shares_denom
		);
	}

	#[test]
	pub fn user_stake_votes_info_updated() {
		let mut env_deps = new_env_and_instantiate(None);
		let contract_addr = &env_deps.0.contract.address.clone();
		let vote_shares_denom = format!("factory/{}/votes", contract_addr);

		assert_eq!(
			helpers::query_user_stats(
				&mut env_deps,
				SHARES_HOLDER_ACCOUNT_1
			).unwrap(),
			CourtUserStatsJsonable {
				staked_votes: 0u128.into()
			}
		);

		let execute_response = helpers::execute(
			&mut env_deps,
			Some(
				MessageInfo { sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), funds: vec![coin(31337, &vote_shares_denom)] }
			),
			CourtExecuteMsg::Stake
		).unwrap();

		assert_eq!(
			helpers::query_user_stats(
				&mut env_deps,
				SHARES_HOLDER_ACCOUNT_1
			).unwrap(),
			CourtUserStatsJsonable {
				staked_votes: 31337u128.into()
			}
		);
		assert_eq!(
			execute_response.events,
			vec![
				cosmwasm_std::Event::new("stake")
					.add_attribute("user", SHARES_HOLDER_ACCOUNT_1)
					.add_attribute("user_new_votes", 31337.to_string())
					.add_attribute("user_total_votes", 31337.to_string())
			]
		);


		let execute_response = helpers::execute(
			&mut env_deps,
			Some(
				MessageInfo { sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), funds: vec![coin(8663, &vote_shares_denom)] }
			),
			CourtExecuteMsg::Stake
		).unwrap();

		assert_eq!(
			helpers::query_user_stats(
				&mut env_deps,
				SHARES_HOLDER_ACCOUNT_1
			).unwrap(),
			CourtUserStatsJsonable {
				staked_votes: 40000u128.into()
			}
		);
		assert_eq!(
			execute_response.events,
			vec![
				cosmwasm_std::Event::new("stake")
					.add_attribute("user", SHARES_HOLDER_ACCOUNT_1)
					.add_attribute("user_new_votes", 8663.to_string())
					.add_attribute("user_total_votes", 40000.to_string())
			]
		);
	}

	#[test]
	pub fn user_unstake_votes_unfunded_check() {
		let mut env_deps = new_env_and_instantiate(None);

		helpers::assert_unfunded_instruction(
			&mut env_deps,
			Some(MessageInfo { sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), funds: vec![] }),
			CourtExecuteMsg::Unstake
		);
	}

	#[test]
	pub fn user_unstake_votes_must_have_staked_check() {
		let mut env_deps = new_env_and_instantiate(None);
		assert!(
			helpers::execute(
				&mut env_deps,
				Some(MessageInfo { sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), funds: vec![] }),
				CourtExecuteMsg::Unstake
			).is_err_and(|err| {
				err.to_string() == "No user votes staked"
			})
		);
	}

	#[test]
	pub fn user_unstake_votes_only_when_not_in_pending_proposals() {
		let mut env_deps = new_env_and_instantiate(None);
		// Sanity tests for current config we're testing against
		assert_eq!(helpers::query_config(&env_deps).unwrap().minimum_vote_proposal_percent, 10);
		assert_eq!(helpers::query_config(&env_deps).unwrap().max_proposal_expiry_time_seconds, 7200);
		assert_eq!(helpers::get_known_vote_supply(&env_deps), 1000000);

		helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 100000);
		helpers::execute_propose_transaction(
			&mut env_deps,
			SHARES_HOLDER_ACCOUNT_1,
			vec![ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_ACCOUNT_1.into(),
				denom: "usei".into(),
				amount: 1337u128.into()
			}],
			1200
		);
		assert!(
			helpers::execute(
				&mut env_deps,
				Some(MessageInfo { sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), funds: vec![] }),
				CourtExecuteMsg::Unstake
			).is_err_and(|err| {
				err.to_string() == "Staked votes must not be tied to any proposals"
			})
		);
	}

	#[test]
	pub fn user_unstake_votes_tokens_actually_returned() {
		let mut env_deps = new_env_and_instantiate(None);
		let vote_shares_denom = helpers::query_denom(&env_deps).unwrap().votes;
		let user1_stake_amount_1 = 2147u128;
		let user1_stake_amount_2 = 2563u128;
		let user1_stake_amount_total = 4710u128;

		let user2_stake_amount_1 = 567u128;
		let user2_stake_amount_2 = 2531u128;
		let user2_stake_amount_total = 3098u128;

		helpers::execute_stake_votes(
			&mut env_deps,
			SHARES_HOLDER_ACCOUNT_1,
			user1_stake_amount_1
		);
		helpers::execute_stake_votes(
			&mut env_deps,
			SHARES_HOLDER_ACCOUNT_2,
			user2_stake_amount_1
		);
		helpers::execute_stake_votes(
			&mut env_deps,
			SHARES_HOLDER_ACCOUNT_1,
			user1_stake_amount_2
		);
		helpers::execute_stake_votes(
			&mut env_deps,
			SHARES_HOLDER_ACCOUNT_2,
			user2_stake_amount_2
		);

		let execute_response = helpers::execute(
			&mut env_deps,
			Some(
				MessageInfo { sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2), funds: vec![] }
			),
			CourtExecuteMsg::Unstake
		).unwrap();

		// Sends the staked amount of tokens back to sender
		assert!(
			execute_response.messages.iter().any(|sub_msg| {
				match &sub_msg.msg {
					cosmwasm_std::CosmosMsg::Bank(sub_msg) => {
						*sub_msg == cosmwasm_std::BankMsg::Send {
							to_address: SHARES_HOLDER_ACCOUNT_2.into(),
							amount: vec![coin(user2_stake_amount_total, &vote_shares_denom)]
						}
					}
					_ => false
				}
			})
		);
		assert_eq!(
			execute_response.events,
			vec![
				cosmwasm_std::Event::new("unstake")
					.add_attribute("user", SHARES_HOLDER_ACCOUNT_2)
					.add_attribute("user_total_votes", user2_stake_amount_total.to_string())
			]
		);
		assert_eq!(
			helpers::query_user_stats(
				&mut env_deps,
				SHARES_HOLDER_ACCOUNT_2
			).unwrap(),
			CourtUserStatsJsonable {
				staked_votes: 0u128.into()
			}
		);
		assert_eq!(
			helpers::query_user_stats(
				&mut env_deps,
				SHARES_HOLDER_ACCOUNT_1
			).unwrap(),
			CourtUserStatsJsonable {
				staked_votes: user1_stake_amount_total.into()
			}
		);

		let execute_response = helpers::execute(
			&mut env_deps,
			Some(
				MessageInfo { sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), funds: vec![] }
			),
			CourtExecuteMsg::Unstake
		).unwrap();

		// Sends the staked amount of tokens back to sender
		assert!(
			execute_response.messages.iter().any(|sub_msg| {
				match &sub_msg.msg {
					cosmwasm_std::CosmosMsg::Bank(sub_msg) => {
						*sub_msg == cosmwasm_std::BankMsg::Send {
							to_address: SHARES_HOLDER_ACCOUNT_1.into(),
							amount: vec![coin(user1_stake_amount_total, &vote_shares_denom)]
						}
					}
					_ => false
				}
			})
		);
		assert_eq!(
			execute_response.events,
			vec![
				cosmwasm_std::Event::new("unstake")
					.add_attribute("user", SHARES_HOLDER_ACCOUNT_1)
					.add_attribute("user_total_votes", user1_stake_amount_total.to_string())
			]
		);
		assert_eq!(
			helpers::query_user_stats(
				&mut env_deps,
				SHARES_HOLDER_ACCOUNT_1
			).unwrap(),
			CourtUserStatsJsonable {
				staked_votes: 0u128.into()
			}
		);
	}

	#[test]
	pub fn user_propose_transaction_unfunded_check() {
		let mut env_deps = new_env_and_instantiate(None);
		helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 500000);
		helpers::assert_unfunded_instruction(
			&mut env_deps,
			Some(MessageInfo { sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), funds: vec![] }),
			CourtExecuteMsg::ProposeTransaction {
				msgs: vec![ProposedCourtMsgJsonable::SendCoin {
					to: RANDOM_ACCOUNT_1.into(),
					denom: "usei".into(),
					amount: 1337u128.into()
				}],
				expiry_time_seconds: 86400
			}
		);
	}

	#[test]
	pub fn user_propose_transaction_non_empty_check() {
		let mut env_deps = new_env_and_instantiate(None);
		helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 500000);
		let execute_result = helpers::execute(
			&mut env_deps,
			Some(MessageInfo { sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), funds: vec![] }),
			CourtExecuteMsg::ProposeTransaction {
				msgs: vec![],
				expiry_time_seconds: 86400
			}
		);
		assert!(
			execute_result.is_err_and(|err| {
				err.to_string().contains("Proposal must have at least one message")
			})
		);
	}

	#[test]
	pub fn user_propose_transaction_expire_time_check() {
		let mut env_deps = new_env_and_instantiate(None);
		// Sanity tests for current config we're testing against
		assert_eq!(helpers::query_config(&env_deps).unwrap().minimum_vote_proposal_percent, 10);
		assert_eq!(helpers::query_config(&env_deps).unwrap().max_proposal_expiry_time_seconds, 7200);
		assert_eq!(helpers::get_known_vote_supply(&env_deps), 1000000);

		helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 500000);
		let execute_result = helpers::execute(
			&mut env_deps,
			Some(MessageInfo { sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), funds: vec![] }),
			CourtExecuteMsg::ProposeTransaction {
				msgs: vec![ProposedCourtMsgJsonable::SendCoin {
					to: RANDOM_ACCOUNT_1.into(),
					denom: "usei".into(),
					amount: 1337u128.into()
				}],
				expiry_time_seconds: 7201
			}
		);
		assert!(
			execute_result.is_err_and(|err| {
				err.to_string().contains("Proposal takes too long to expire")
			})
		);
		helpers::execute(
			&mut env_deps,
			Some(MessageInfo { sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), funds: vec![] }),
			CourtExecuteMsg::ProposeTransaction {
				msgs: vec![ProposedCourtMsgJsonable::SendCoin {
					to: RANDOM_ACCOUNT_1.into(),
					denom: "usei".into(),
					amount: 1337u128.into()
				}],
				expiry_time_seconds: 7200
			}
		).unwrap();

	}

	#[test]
	pub fn user_propose_transaction_minimum_vote_check() {
		let mut env_deps = new_env_and_instantiate(None);
		// Sanity tests for current config we're testing against
		assert_eq!(helpers::query_config(&env_deps).unwrap().minimum_vote_proposal_percent, 10);
		assert_eq!(helpers::query_config(&env_deps).unwrap().max_proposal_expiry_time_seconds, 7200);
		assert_eq!(helpers::get_known_vote_supply(&env_deps), 1000000);

		helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 99999);
		let execute_result = helpers::execute(
			&mut env_deps,
			Some(MessageInfo { sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), funds: vec![] }),
			CourtExecuteMsg::ProposeTransaction {
				msgs: vec![ProposedCourtMsgJsonable::SendCoin {
					to: RANDOM_ACCOUNT_1.into(),
					denom: "usei".into(),
					amount: 1337u128.into()
				}],
				expiry_time_seconds: 1200
			}
		);
		assert!(
			execute_result.is_err_and(|err| {
				err.to_string().contains("User doesn't have enough staked votes to submit a proposal")
			})
		);
		helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 1);

		helpers::execute_propose_transaction(
			&mut env_deps,
			SHARES_HOLDER_ACCOUNT_1,
			vec![ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_ACCOUNT_1.into(),
				denom: "usei".into(),
				amount: 1337u128.into()
			}],
			1200
		);
		
		
		assert_eq!(helpers::query_get_proposal(&env_deps, 0), Ok(Some(
			CourtQueryResponseTransactionProposal {
				proposal_id: 0,
				status: TransactionProposalStatus::Pending,
				info: TransactionProposalInfoJsonable {
					proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
					votes_for: 100000u128.into(),
					votes_against: 0u128.into(),
					votes_abstain: 0u128.into(),
					execution_status: TransactionProposalExecutionStatus::NotExecuted,
					expiry_timestamp_ms: env_deps.0.block.time.plus_seconds(1200).millis()
				},
				messages: vec![ProposedCourtMsgJsonable::SendCoin {
					to: RANDOM_ACCOUNT_1.into(),
					denom: "usei".into(),
					amount: 1337u128.into()
				}]
			}
		)));
		assert_eq!(helpers::query_get_proposal_user_votes(&env_deps, 0, None, None, false), Ok(
			vec![
				CourtQueryResponseUserVote {
					user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
					info: CourtUserVoteInfoJsonable { active_votes: 100000u128.into(), vote: CourtUserVoteStatus::Approve }
				}
			]
		));
		assert_eq!(helpers::query_get_proposals(&env_deps, None, None, false), Ok(
			vec![
				CourtQueryResponseTransactionProposal {
					proposal_id: 0,
					status: TransactionProposalStatus::Pending,
					info: TransactionProposalInfoJsonable {
						proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
						votes_for: 100000u128.into(),
						votes_against: 0u128.into(),
						votes_abstain: 0u128.into(),
						execution_status: TransactionProposalExecutionStatus::NotExecuted,
						expiry_timestamp_ms: env_deps.0.block.time.plus_seconds(1200).millis()
					},
					messages: vec![ProposedCourtMsgJsonable::SendCoin {
						to: RANDOM_ACCOUNT_1.into(),
						denom: "usei".into(),
						amount: 1337u128.into()
					}]
				}
			]
		));
		assert_eq!(helpers::query_proposal_amount(&env_deps), Ok(1));
		assert_eq!(helpers::query_get_user_active_proposals(&env_deps, SHARES_HOLDER_ACCOUNT_1, None, None, false), Ok(vec![0]));
		assert_eq!(helpers::query_get_user_active_proposals(&env_deps, SHARES_HOLDER_ACCOUNT_2, None, None, false), Ok(vec![]));
		assert_eq!(helpers::query_get_users_with_active_proposals(&env_deps, None, None, false), Ok(vec![
			CourtQueryUserWithActiveProposal {user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), proposal_id: 0}]
		));
		assert_eq!(helpers::query_user_vote_info(&env_deps, SHARES_HOLDER_ACCOUNT_1, 0), Ok(CourtUserVoteInfoJsonable { active_votes: 100000u128.into(), vote: CourtUserVoteStatus::Approve }));
		assert_eq!(helpers::query_user_vote_info(&env_deps, SHARES_HOLDER_ACCOUNT_2, 0), Ok(CourtUserVoteInfoJsonable { active_votes: 0u128.into(), vote: CourtUserVoteStatus::Abstain }));
		assert_eq!(helpers::query_user_vote_info(&env_deps, SHARES_HOLDER_ACCOUNT_1, 1), Ok(CourtUserVoteInfoJsonable { active_votes: 0u128.into(), vote: CourtUserVoteStatus::Abstain }));

	}

	#[test]
	pub fn user_propose_transaction_multiple() {
		let mut env_deps = new_env_and_instantiate(None);
		// Sanity tests for current config we're testing against
		assert_eq!(helpers::query_config(&env_deps).unwrap().minimum_vote_proposal_percent, 10);
		assert_eq!(helpers::query_config(&env_deps).unwrap().max_proposal_expiry_time_seconds, 7200);
		assert_eq!(helpers::get_known_vote_supply(&env_deps), 1000000);

		helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 100000);
		helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_2, 150000);

		helpers::execute_propose_transaction(
			&mut env_deps,
			SHARES_HOLDER_ACCOUNT_1,
			vec![ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_ACCOUNT_1.into(),
				denom: "usei".into(),
				amount: 1337u128.into()
			}],
			1200
		);
		helpers::execute_propose_transaction(
			&mut env_deps,
			SHARES_HOLDER_ACCOUNT_1,
			vec![ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_ACCOUNT_3.into(),
				denom: "usei".into(),
				amount: 1338u128.into()
			}],
			1300
		);
		helpers::execute_propose_transaction(
			&mut env_deps,
			SHARES_HOLDER_ACCOUNT_2,
			vec![ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_ACCOUNT_2.into(),
				denom: "usei".into(),
				amount: 1339u128.into()
			}],
			1400
		);

		assert_eq!(helpers::query_get_proposal(&env_deps, 0), Ok(Some(
			CourtQueryResponseTransactionProposal {
				proposal_id: 0,
				status: TransactionProposalStatus::Pending,
				info: TransactionProposalInfoJsonable {
					proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
					votes_for: 100000u128.into(),
					votes_against: 0u128.into(),
					votes_abstain: 0u128.into(),
					execution_status: TransactionProposalExecutionStatus::NotExecuted,
					expiry_timestamp_ms: env_deps.0.block.time.plus_seconds(1200).millis()
				},
				messages: vec![ProposedCourtMsgJsonable::SendCoin {
					to: RANDOM_ACCOUNT_1.into(),
					denom: "usei".into(),
					amount: 1337u128.into()
				}]
			}
		)));
		assert_eq!(helpers::query_get_proposal(&env_deps, 1), Ok(Some(
			CourtQueryResponseTransactionProposal {
				proposal_id: 1,
				status: TransactionProposalStatus::Pending,
				info: TransactionProposalInfoJsonable {
					proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
					votes_for: 100000u128.into(),
					votes_against: 0u128.into(),
					votes_abstain: 0u128.into(),
					execution_status: TransactionProposalExecutionStatus::NotExecuted,
					expiry_timestamp_ms: env_deps.0.block.time.plus_seconds(1300).millis()
				},
				messages: vec![ProposedCourtMsgJsonable::SendCoin {
					to: RANDOM_ACCOUNT_3.into(),
					denom: "usei".into(),
					amount: 1338u128.into()
				}]
			}
		)));
		assert_eq!(helpers::query_get_proposal(&env_deps, 2), Ok(Some(
			CourtQueryResponseTransactionProposal {
				proposal_id: 2,
				status: TransactionProposalStatus::Pending,
				info: TransactionProposalInfoJsonable {
					proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
					votes_for: 150000u128.into(),
					votes_against: 0u128.into(),
					votes_abstain: 0u128.into(),
					execution_status: TransactionProposalExecutionStatus::NotExecuted,
					expiry_timestamp_ms: env_deps.0.block.time.plus_seconds(1400).millis()
				},
				messages: vec![ProposedCourtMsgJsonable::SendCoin {
					to: RANDOM_ACCOUNT_2.into(),
					denom: "usei".into(),
					amount: 1339u128.into()
				}]
			}
		)));
		assert_eq!(helpers::query_get_proposal_user_votes(&env_deps, 0, None, None, false), Ok(
			vec![
				CourtQueryResponseUserVote {
					user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
					info: CourtUserVoteInfoJsonable { active_votes: 100000u128.into(), vote: CourtUserVoteStatus::Approve }
				}
			]
		));
		assert_eq!(helpers::query_get_proposal_user_votes(&env_deps, 1, None, None, false), Ok(
			vec![
				CourtQueryResponseUserVote {
					user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
					info: CourtUserVoteInfoJsonable { active_votes: 100000u128.into(), vote: CourtUserVoteStatus::Approve }
				}
			]
		));
		assert_eq!(helpers::query_get_proposal_user_votes(&env_deps, 2, None, None, false), Ok(
			vec![
				CourtQueryResponseUserVote {
					user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
					info: CourtUserVoteInfoJsonable { active_votes: 150000u128.into(), vote: CourtUserVoteStatus::Approve }
				}
			]
		));

		assert_eq!(helpers::query_get_proposals(&env_deps, None, None, false), Ok(
			vec![
				CourtQueryResponseTransactionProposal {
					proposal_id: 0,
					status: TransactionProposalStatus::Pending,
					info: TransactionProposalInfoJsonable {
						proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
						votes_for: 100000u128.into(),
						votes_against: 0u128.into(),
						votes_abstain: 0u128.into(),
						execution_status: TransactionProposalExecutionStatus::NotExecuted,
						expiry_timestamp_ms: env_deps.0.block.time.plus_seconds(1200).millis()
					},
					messages: vec![ProposedCourtMsgJsonable::SendCoin {
						to: RANDOM_ACCOUNT_1.into(),
						denom: "usei".into(),
						amount: 1337u128.into()
					}]
				},
				CourtQueryResponseTransactionProposal {
					proposal_id: 1,
					status: TransactionProposalStatus::Pending,
					info: TransactionProposalInfoJsonable {
						proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
						votes_for: 100000u128.into(),
						votes_against: 0u128.into(),
						votes_abstain: 0u128.into(),
						execution_status: TransactionProposalExecutionStatus::NotExecuted,
						expiry_timestamp_ms: env_deps.0.block.time.plus_seconds(1300).millis()
					},
					messages: vec![ProposedCourtMsgJsonable::SendCoin {
						to: RANDOM_ACCOUNT_3.into(),
						denom: "usei".into(),
						amount: 1338u128.into()
					}]
				},
				CourtQueryResponseTransactionProposal {
					proposal_id: 2,
					status: TransactionProposalStatus::Pending,
					info: TransactionProposalInfoJsonable {
						proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
						votes_for: 150000u128.into(),
						votes_against: 0u128.into(),
						votes_abstain: 0u128.into(),
						execution_status: TransactionProposalExecutionStatus::NotExecuted,
						expiry_timestamp_ms: env_deps.0.block.time.plus_seconds(1400).millis()
					},
					messages: vec![ProposedCourtMsgJsonable::SendCoin {
						to: RANDOM_ACCOUNT_2.into(),
						denom: "usei".into(),
						amount: 1339u128.into()
					}]
				}
			]
		));
		assert_eq!(helpers::query_get_proposals(&env_deps, Some(1), Some(1), false), Ok(
			vec![
				CourtQueryResponseTransactionProposal {
					proposal_id: 1,
					status: TransactionProposalStatus::Pending,
					info: TransactionProposalInfoJsonable {
						proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
						votes_for: 100000u128.into(),
						votes_against: 0u128.into(),
						votes_abstain: 0u128.into(),
						execution_status: TransactionProposalExecutionStatus::NotExecuted,
						expiry_timestamp_ms: env_deps.0.block.time.plus_seconds(1300).millis()
					},
					messages: vec![ProposedCourtMsgJsonable::SendCoin {
						to: RANDOM_ACCOUNT_3.into(),
						denom: "usei".into(),
						amount: 1338u128.into()
					}]
				}
			]
		));
		assert_eq!(helpers::query_get_proposals(&env_deps, None, None, true), Ok(
			vec![
				CourtQueryResponseTransactionProposal {
					proposal_id: 2,
					status: TransactionProposalStatus::Pending,
					info: TransactionProposalInfoJsonable {
						proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
						votes_for: 150000u128.into(),
						votes_against: 0u128.into(),
						votes_abstain: 0u128.into(),
						execution_status: TransactionProposalExecutionStatus::NotExecuted,
						expiry_timestamp_ms: env_deps.0.block.time.plus_seconds(1400).millis()
					},
					messages: vec![ProposedCourtMsgJsonable::SendCoin {
						to: RANDOM_ACCOUNT_2.into(),
						denom: "usei".into(),
						amount: 1339u128.into()
					}]
				},
				CourtQueryResponseTransactionProposal {
					proposal_id: 1,
					status: TransactionProposalStatus::Pending,
					info: TransactionProposalInfoJsonable {
						proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
						votes_for: 100000u128.into(),
						votes_against: 0u128.into(),
						votes_abstain: 0u128.into(),
						execution_status: TransactionProposalExecutionStatus::NotExecuted,
						expiry_timestamp_ms: env_deps.0.block.time.plus_seconds(1300).millis()
					},
					messages: vec![ProposedCourtMsgJsonable::SendCoin {
						to: RANDOM_ACCOUNT_3.into(),
						denom: "usei".into(),
						amount: 1338u128.into()
					}]
				},
				CourtQueryResponseTransactionProposal {
					proposal_id: 0,
					status: TransactionProposalStatus::Pending,
					info: TransactionProposalInfoJsonable {
						proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
						votes_for: 100000u128.into(),
						votes_against: 0u128.into(),
						votes_abstain: 0u128.into(),
						execution_status: TransactionProposalExecutionStatus::NotExecuted,
						expiry_timestamp_ms: env_deps.0.block.time.plus_seconds(1200).millis()
					},
					messages: vec![ProposedCourtMsgJsonable::SendCoin {
						to: RANDOM_ACCOUNT_1.into(),
						denom: "usei".into(),
						amount: 1337u128.into()
					}]
				},
			]
		));
		assert_eq!(helpers::query_get_proposals(&env_deps, Some(2), None, true), Ok(
			vec![
				CourtQueryResponseTransactionProposal {
					proposal_id: 0,
					status: TransactionProposalStatus::Pending,
					info: TransactionProposalInfoJsonable {
						proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
						votes_for: 100000u128.into(),
						votes_against: 0u128.into(),
						votes_abstain: 0u128.into(),
						execution_status: TransactionProposalExecutionStatus::NotExecuted,
						expiry_timestamp_ms: env_deps.0.block.time.plus_seconds(1200).millis()
					},
					messages: vec![ProposedCourtMsgJsonable::SendCoin {
						to: RANDOM_ACCOUNT_1.into(),
						denom: "usei".into(),
						amount: 1337u128.into()
					}]
				},
			]
		));
		assert_eq!(helpers::query_proposal_amount(&env_deps), Ok(3));
		assert_eq!(helpers::query_get_user_active_proposals(&env_deps, SHARES_HOLDER_ACCOUNT_1, None, None, false), Ok(vec![0, 1]));
		assert_eq!(helpers::query_get_user_active_proposals(&env_deps, SHARES_HOLDER_ACCOUNT_2, None, None, false), Ok(vec![2]));
		assert_eq!(helpers::query_get_user_active_proposals(&env_deps, SHARES_HOLDER_ACCOUNT_1, None, Some(1), false), Ok(vec![0]));
		assert_eq!(helpers::query_get_user_active_proposals(&env_deps, SHARES_HOLDER_ACCOUNT_1, None, Some(1), true), Ok(vec![1]));
		assert_eq!(helpers::query_get_user_active_proposals(&env_deps, SHARES_HOLDER_ACCOUNT_1, Some(1), None, false), Ok(vec![1]));
		assert_eq!(helpers::query_get_user_active_proposals(&env_deps, SHARES_HOLDER_ACCOUNT_1, Some(1), None, true), Ok(vec![0]));
		assert_eq!(helpers::query_get_users_with_active_proposals(&env_deps, None, None, false), Ok(vec![
			CourtQueryUserWithActiveProposal {user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), proposal_id: 0},
			CourtQueryUserWithActiveProposal {user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), proposal_id: 1},
			CourtQueryUserWithActiveProposal {user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2), proposal_id: 2}
		]));
		assert_eq!(helpers::query_get_users_with_active_proposals(&env_deps, None, None, true), Ok(vec![
			CourtQueryUserWithActiveProposal {user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2), proposal_id: 2},
			CourtQueryUserWithActiveProposal {user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), proposal_id: 1},
			CourtQueryUserWithActiveProposal {user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), proposal_id: 0}
		]));
		assert_eq!(helpers::query_get_users_with_active_proposals(&env_deps, None, Some(1), false), Ok(vec![
			CourtQueryUserWithActiveProposal {user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), proposal_id: 0}
		]));
		assert_eq!(helpers::query_get_users_with_active_proposals(&env_deps, Some(CourtQueryUserWithActiveProposal {user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), proposal_id: 0}), None, false), Ok(vec![
			CourtQueryUserWithActiveProposal {user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), proposal_id: 1},
			CourtQueryUserWithActiveProposal {user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2), proposal_id: 2}
		]));
		assert_eq!(helpers::query_get_users_with_active_proposals(&env_deps, Some(CourtQueryUserWithActiveProposal {user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2), proposal_id: 2}), Some(1), true), Ok(vec![
			CourtQueryUserWithActiveProposal {user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), proposal_id: 1},
		]));
		
		assert_eq!(helpers::query_user_vote_info(&env_deps, SHARES_HOLDER_ACCOUNT_1, 0), Ok(CourtUserVoteInfoJsonable { active_votes: 100000u128.into(), vote: CourtUserVoteStatus::Approve }));
		assert_eq!(helpers::query_user_vote_info(&env_deps, SHARES_HOLDER_ACCOUNT_2, 0), Ok(CourtUserVoteInfoJsonable { active_votes: 0u128.into(), vote: CourtUserVoteStatus::Abstain }));
		assert_eq!(helpers::query_user_vote_info(&env_deps, SHARES_HOLDER_ACCOUNT_1, 1), Ok(CourtUserVoteInfoJsonable { active_votes: 100000u128.into(), vote: CourtUserVoteStatus::Approve }));
		assert_eq!(helpers::query_user_vote_info(&env_deps, SHARES_HOLDER_ACCOUNT_2, 1), Ok(CourtUserVoteInfoJsonable { active_votes: 0u128.into(), vote: CourtUserVoteStatus::Abstain }));
		assert_eq!(helpers::query_user_vote_info(&env_deps, SHARES_HOLDER_ACCOUNT_1, 2), Ok(CourtUserVoteInfoJsonable { active_votes: 0u128.into(), vote: CourtUserVoteStatus::Abstain }));
		assert_eq!(helpers::query_user_vote_info(&env_deps, SHARES_HOLDER_ACCOUNT_2, 2), Ok(CourtUserVoteInfoJsonable { active_votes: 150000u128.into(), vote: CourtUserVoteStatus::Approve }));
	}

	#[test]
	pub fn user_propose_transaction_token_recipiant_check() {
		let mut env_deps = new_env_and_instantiate(None);
		// Sanity tests for current config we're testing against
		assert_eq!(helpers::query_config(&env_deps).unwrap().minimum_vote_proposal_percent, 10);
		assert_eq!(helpers::query_config(&env_deps).unwrap().max_proposal_expiry_time_seconds, 7200);
		assert_eq!(helpers::get_known_vote_supply(&env_deps), 1000000);

		helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 100000);
		let execute_result = helpers::execute(
			&mut env_deps,
			Some(MessageInfo { sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), funds: vec![] }),
			CourtExecuteMsg::ProposeTransaction {
				msgs: vec![ProposedCourtMsgJsonable::SendCoin {
					to: RANDOM_EVM_ACCOUNT_1.into(),
					denom: "usei".into(),
					amount: 1337u128.into()
				}],
				expiry_time_seconds: 1200
			}
		);
		assert!(
			execute_result.is_err_and(|err| {
				err.to_string().contains("an address beginning with \"sei1\" is required")
			})
		);

		let execute_result = helpers::execute(
			&mut env_deps,
			Some(MessageInfo { sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), funds: vec![] }),
			CourtExecuteMsg::ProposeTransaction {
				msgs: vec![ProposedCourtMsgJsonable::SendCoin {
					to: RANDOM_ACCOUNT_1.into(),
					denom: format!("erc20/{}", RANDOM_EVM_ACCOUNT_1).into(),
					amount: 1337u128.into()
				}],
				expiry_time_seconds: 1200
			}
		);
		assert!(
			execute_result.is_err_and(|err| {
				err.to_string().contains("an address beginning with \"0x\" is required")
			})
		);
		helpers::execute(
			&mut env_deps,
			Some(MessageInfo { sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), funds: vec![] }),
			CourtExecuteMsg::ProposeTransaction {
				msgs: vec![ProposedCourtMsgJsonable::SendCoin {
					to: RANDOM_EVM_ACCOUNT_1.into(),
					denom: format!("erc20/{}", RANDOM_EVM_ACCOUNT_1).into(),
					amount: 1337u128.into()
				}],
				expiry_time_seconds: 1200
			}
		).unwrap();
	}

}
