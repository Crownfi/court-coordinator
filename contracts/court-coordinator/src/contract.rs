use cosmwasm_std::{entry_point, to_json_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult};
use crownfi_cw_common::{data_types::canonical_addr::SeiCanonicalAddr, env::ClonableEnvInfoMut, extentions::timestamp::TimestampExtentions, storage::item::StoredItem};
use cw2::set_contract_version;
use sei_cosmwasm::{SeiQueryWrapper, SeiMsg};

use crate::{error::CourtContractError, msg::{CourtAdminExecuteMsg, CourtExecuteMsg, CourtInstantiateMsg, CourtMigrateMsg, CourtQueryMsg, CourtQueryResponseDenom, CourtQueryResponseTransactionProposal, CourtQueryResponseUserVote}, state::{app::{get_transaction_proposal_info_vec, get_transaction_proposal_messages_vec, CourtAppConfig, CourtAppConfigJsonable}, user::{get_all_user_votes, get_user_stats_store, get_user_vote_info_store}}, workarounds::{mint_to_workaround, total_supply_workaround}};

use self::{shares::{VOTES_SUBDENOM, votes_denom}, admin::AdminMsgExecutor, user::{process_stake, process_unstake, process_vote, process_propose_transaction}, permissionless::{process_deactivate_votes, process_execute_proposal}};

pub mod shares;
pub mod admin;
pub mod permissionless;
pub mod user;

const CONTRACT_NAME: &str = "crates.io:court-coordinator-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
#[inline]
pub fn instantiate(
	deps: DepsMut<SeiQueryWrapper>,
	env: Env,
	msg_info: MessageInfo,
	msg: CourtInstantiateMsg,
) -> Result<Response<SeiMsg>, CourtContractError> {
	enforce_unfunded(&msg_info)?;
	set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
	CourtAppConfigJsonable {
		allow_new_proposals: true,
		minimum_vote_proposal_percent: msg.minimum_vote_proposal_percent,
		minimum_vote_turnout_percent: msg.minimum_vote_turnout_percent,
		minimum_vote_pass_percent: msg.minimum_vote_pass_percent,
		max_proposal_expiry_time_seconds: msg.max_proposal_expiry_time_seconds,
		execution_expiry_time_seconds: msg.execution_expiry_time_seconds,
		last_config_change_timestamp_ms: env.block.time.millis(),
		admin: msg.admin
	}.into_storable(deps.api)?.save(deps.storage)?;
	let new_denom = votes_denom(&env);
	Ok(
		mint_to_workaround(
			Response::new()
			.add_message(
				SeiMsg::CreateDenom {
					subdenom: VOTES_SUBDENOM.to_string()
				}
			),
			deps.storage,
			&new_denom,
			&msg.shares_mint_receiver,
			msg.shares_mint_amount.u128()
		)?
	)
}


#[cfg_attr(not(feature = "library"), entry_point)]
#[inline]
pub fn execute(
	deps: DepsMut<SeiQueryWrapper>,
	env: Env,
	msg_info: MessageInfo,
	msg: CourtExecuteMsg,
) -> Result<Response<SeiMsg>, CourtContractError> {
	Ok(
		match msg {
			CourtExecuteMsg::Admin(admin_msg) => {
				//deps.querier.query(request)
				let mut admin_executor = AdminMsgExecutor::new(ClonableEnvInfoMut::new(deps, env), &msg_info)?;
				match admin_msg {
					CourtAdminExecuteMsg::ChangeConfig {
						minimum_vote_proposal_percent,
						minimum_vote_turnout_percent,
						minimum_vote_pass_percent,
						max_proposal_expiry_time_seconds,
						execution_expiry_time_seconds,
						admin
					} => {
						admin_executor.process_change_config(
							&msg_info,
							minimum_vote_proposal_percent,
							minimum_vote_turnout_percent,
							minimum_vote_pass_percent,
							max_proposal_expiry_time_seconds,
							execution_expiry_time_seconds,
							admin
						)?
					},
					CourtAdminExecuteMsg::AllowNewProposals { allowed } => {
						admin_executor.process_allow_new_proposals(&msg_info, allowed)?
					},
					CourtAdminExecuteMsg::MintShares { receiver, amount } => {
						admin_executor.process_mint_shares(&msg_info, receiver, amount)?
					},
				}
			},
			CourtExecuteMsg::Stake => {
				process_stake(ClonableEnvInfoMut::new(deps, env), msg_info)?
			},
			CourtExecuteMsg::Unstake => {
				process_unstake(ClonableEnvInfoMut::new(deps, env), msg_info)?
			},
			CourtExecuteMsg::Vote { id, approval } => {
				process_vote(ClonableEnvInfoMut::new(deps, env), msg_info, id, approval)?
			},
			CourtExecuteMsg::DeactivateVotes { user, limit } => {
				let env_info = ClonableEnvInfoMut::new(deps, env);
				process_deactivate_votes(
					env_info.clone(),
					msg_info,
					user.and_then(|v| {SeiCanonicalAddr::from_addr_using_api(&v, *env_info.api).ok()}),
					limit
				)?
			},
			CourtExecuteMsg::ProposeTransaction { msgs, expiry_time_seconds } => {
				let env_info = ClonableEnvInfoMut::new(deps, env);
				process_propose_transaction(
					env_info.clone(),
					msg_info,
					msgs.into_iter().map(|v| {v.into_storable(*env_info.api)}).collect::<Result<_, _>>()?,
					expiry_time_seconds
				)?
			},
			CourtExecuteMsg::ExecuteProposal { id } => {
				process_execute_proposal(ClonableEnvInfoMut::new(deps, env), msg_info, id)?
			}
		}
	)
}

#[cfg_attr(not(feature = "library"), entry_point)]
#[inline]
pub fn migrate(
    _deps: DepsMut<SeiQueryWrapper>,
   _env: Env,
   _msg: CourtMigrateMsg,
) -> Result<Response<SeiMsg>, CourtContractError> {
	// let contract_version = get_contract_version(deps.storage)?;
	//contract_version.
	todo!();
	// cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
}


#[cfg_attr(not(feature = "library"), entry_point)]
#[inline]
pub fn query(deps: Deps, env: Env, msg: CourtQueryMsg) -> Result<Binary, CourtContractError> {
	Ok(
		match msg {
			CourtQueryMsg::Config => {
				to_json_binary(
					&CourtAppConfig::load_non_empty(deps.storage)?.into_jsonable(deps.api)?
				)?
			},
			CourtQueryMsg::Denom => {
				to_json_binary(
					&CourtQueryResponseDenom {
						votes: votes_denom(&env)
					}
				)?
			},
			CourtQueryMsg::ProposalAmount => {
				to_json_binary(
					&(get_transaction_proposal_info_vec(
						deps.storage
					)?.len() as u32)
				)?
			}
			CourtQueryMsg::GetProposal { id } => {
				let app_config = CourtAppConfig::load_non_empty(deps.storage)?;
				let total_supply = total_supply_workaround(deps.storage, &votes_denom(&env));
				let proposal_msg_vec = get_transaction_proposal_messages_vec(
					deps.storage
				)?;
				to_json_binary(
					&get_transaction_proposal_info_vec(
						deps.storage
					)?.get(id)?.map(|info| -> Result<_, StdError> {
						Ok(
							CourtQueryResponseTransactionProposal {
								proposal_id: id as u32,
								status: info.status(
									env.block.time.millis(),
									total_supply.u128(),
									&app_config
								),
								info: info.into_jsonable(deps.api)?,
								messages: proposal_msg_vec.get(id as u32)?
									.unwrap_or_default()
									.into_iter()
									.map(|v| {v.into_jsonable(deps.api)})
									.collect::<Result<Vec<_>, _>>()?
							}
						)
					}).transpose()?
				)?
			},
			CourtQueryMsg::GetProposals { skip, limit, descending } => {
				let app_config = CourtAppConfig::load_non_empty(deps.storage)?;
				let total_supply = total_supply_workaround(deps.storage, &votes_denom(&env));
				let proposal_msg_vec = get_transaction_proposal_messages_vec(
					deps.storage
				)?;
				to_json_binary(
					&if descending {
						get_transaction_proposal_info_vec(
							deps.storage
						)?.into_iter()
							.enumerate()
							.rev()
							.skip(skip as usize)
							.take(limit as usize)
							.map(|(index, info_result)| {
								let info = info_result?;
								Ok(
									CourtQueryResponseTransactionProposal {
										proposal_id: index as u32,
										status: info.status(
											env.block.time.millis(),
											total_supply.u128(),
											&app_config
										),
										info: info.into_jsonable(deps.api)?,
										messages: proposal_msg_vec.get(index as u32)?
											.unwrap_or_default()
											.into_iter()
											.map(|v| {v.into_jsonable(deps.api)})
											.collect::<Result<Vec<_>, _>>()?
									}
								)
							}).collect::<StdResult<Vec<CourtQueryResponseTransactionProposal>>>()?
					}else{
						get_transaction_proposal_info_vec(
							deps.storage
						)?.into_iter()
							.enumerate()
							.skip(skip as usize)
							.take(limit as usize)
							.map(|(index, info_result)| {
								let info = info_result?;
								Ok(
									CourtQueryResponseTransactionProposal {
										proposal_id: index as u32,
										status: info.status(
											env.block.time.millis(),
											total_supply.u128(),
											&app_config
										),
										info: info.into_jsonable(deps.api)?,
										messages: proposal_msg_vec.get(index as u32)?
											.unwrap_or_default()
											.into_iter()
											.map(|v| {v.into_jsonable(deps.api)})
											.collect::<Result<Vec<_>, _>>()?
									}
								)
							}).collect::<StdResult<Vec<CourtQueryResponseTransactionProposal>>>()?
					}
				)?
			},
			CourtQueryMsg::UserStats { user } => {
				let user = SeiCanonicalAddr::from_addr_using_api(&user, deps.api)?;
				to_json_binary(
					&get_user_stats_store(
						deps.storage
					)?.get(&user)?.unwrap_or_default().into_jsonable(deps.api)?
				)?
			},
			CourtQueryMsg::UserVoteInfo { user, proposal_id } => {
				let user = SeiCanonicalAddr::from_addr_using_api(&user, deps.api)?;
				to_json_binary(
					&get_user_vote_info_store(
						deps.storage
					)?.get(&(user, proposal_id))?.map(|v| {v.into_jsonable(deps.api).ok()})
				)?
			},
			CourtQueryMsg::GetUserVotes { user, skip, limit, descending, .. } => {
				let user = SeiCanonicalAddr::from_addr_using_api(&user, deps.api)?;
				to_json_binary(
					&if descending {
						get_all_user_votes(deps.storage, user)?
							.rev()
							.skip(skip as usize)
							.take(limit as usize)
							.map(|(proposal_id, info)| {
								Ok(
									CourtQueryResponseUserVote { proposal_id, info: info.into_jsonable(deps.api)? }
								)
							})
							.collect::<Result<Vec<CourtQueryResponseUserVote>, StdError>>()?
					}else{
						get_all_user_votes(deps.storage, user)?
							.skip(skip as usize)
							.take(limit as usize)
							.map(|(proposal_id, info)| {
								Ok(
									CourtQueryResponseUserVote { proposal_id, info: info.into_jsonable(deps.api)? }
								)
							})
							.collect::<Result<Vec<CourtQueryResponseUserVote>, StdError>>()?
					}
				)?
			},
		}
	)
}

pub fn enforce_unfunded(msg_info: &MessageInfo) -> Result<(), CourtContractError> {
	if msg_info.funds.len() > 0 {
		return Err(
			CourtContractError::UnexpectedFunds(msg_info.funds[0].denom.clone())
		)
	}
	Ok(())
}
pub fn enforce_single_payment<'msg>(msg_info: &'msg MessageInfo, expected_denom: &'_ String) -> Result<&'msg Coin, CourtContractError> {
    if msg_info.funds.len() == 0 {
		return Err(
			CourtContractError::TokenMissing(expected_denom.clone())
		);
	}
	if msg_info.funds[0].denom != *expected_denom {
		return Err(
			CourtContractError::UnexpectedFunds(msg_info.funds[0].denom.clone())
		);
	}
	if msg_info.funds.len() > 1 {
		return Err(
			CourtContractError::UnexpectedFunds(msg_info.funds[1].denom.clone())
		);
	}
	return Ok(&msg_info.funds[0])
}
