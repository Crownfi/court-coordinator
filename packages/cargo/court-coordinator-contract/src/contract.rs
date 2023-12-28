use cosmwasm_std::{MessageInfo, Coin, entry_point, DepsMut, Env, Response, coin, BankMsg, Deps, Binary, to_json_binary, StdResult};
use crownfi_cw_common::{extentions::timestamp::TimestampExtentions, storage::{map::StoredMapIter, MaybeMutableStorage, item::StoredItem}, env::ClonableEnvInfoMut};
use cw2::set_contract_version;
use sei_cosmwasm::{SeiQueryWrapper, SeiMsg};

use crate::{error::CourtContractError, msg::{InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg, QueryResponseDenom, QueryResponseTransactionProposal, QueryResponseUserVote, AdminExecuteMsg}, state::{app::{CourtAppConfig, get_transaction_proposal_stored_vec}, user::{get_user_stats_store, get_user_vote_info_store, CourtUserVoteInfo, get_all_user_vote_info_iter}}, workarounds::{total_supply_workaround, mint_to_workaround}};

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
	msg: InstantiateMsg,
) -> Result<Response<SeiMsg>, CourtContractError> {
	enforce_unfunded(&msg_info)?;
	set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
	CourtAppConfig {
		allow_new_proposals: true,
		minimum_vote_proposal_percent: msg.minimum_vote_proposal_percent,
		minimum_vote_turnout_percent: msg.minimum_vote_turnout_percent,
		minimum_vote_pass_percent: msg.minimum_vote_pass_percent,
		max_expiry_time_seconds: msg.max_expiry_time_seconds, 
		last_config_change_timestamp_ms: env.block.time.millis(),
		admin: msg.admin
	}.save(deps.storage)?;
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
	msg: ExecuteMsg,
) -> Result<Response<SeiMsg>, CourtContractError> {
	Ok(
		match msg {
			ExecuteMsg::Admin(admin_msg) => {
				//deps.querier.query(request)
				let mut admin_executor = AdminMsgExecutor::new(ClonableEnvInfoMut::new(deps, env), &msg_info)?;
				match admin_msg {
					AdminExecuteMsg::ChangeConfig {
						minimum_vote_proposal_percent,
						minimum_vote_turnout_percent,
						minimum_vote_pass_percent,
						max_expiry_time_seconds,
						admin
					} => {
						admin_executor.process_change_config(
							&msg_info,
							minimum_vote_proposal_percent,
							minimum_vote_turnout_percent,
							minimum_vote_pass_percent,
							max_expiry_time_seconds,
							admin
						)?
					},
					AdminExecuteMsg::AllowNewProposals { allowed } => {
						admin_executor.process_allow_new_proposals(&msg_info, allowed)?
					},
					AdminExecuteMsg::MintShares { receiver, amount } => {
						admin_executor.process_mint_shares(&msg_info, receiver, amount)?
					},
				}
			},
			ExecuteMsg::Stake => {
				process_stake(ClonableEnvInfoMut::new(deps, env), msg_info)?
			},
			ExecuteMsg::Unstake => {
				process_unstake(ClonableEnvInfoMut::new(deps, env), msg_info)?
			},
			ExecuteMsg::Vote { id, approval } => {
				process_vote(ClonableEnvInfoMut::new(deps, env), msg_info, id, approval)?
			},
			ExecuteMsg::DeactivateVotes { user, limit } => {
				process_deactivate_votes(ClonableEnvInfoMut::new(deps, env), msg_info, user, limit)?
			},
			ExecuteMsg::ProposeTransaction { msgs, expiry_time_seconds } => {
				process_propose_transaction(ClonableEnvInfoMut::new(deps, env), msg_info, msgs, expiry_time_seconds)?
			},
			ExecuteMsg::ExecuteProposal { id } => {
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
   _msg: MigrateMsg,
) -> Result<Response<SeiMsg>, CourtContractError> {
	// let contract_version = get_contract_version(deps.storage)?;
	//contract_version.
	todo!();
	// cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
}


#[cfg_attr(not(feature = "library"), entry_point)]
#[inline]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, CourtContractError> {
	Ok(
		match msg {
			QueryMsg::Config => {
				to_json_binary(
					&CourtAppConfig::load_non_empty(deps.storage)?
				)?
			},
			QueryMsg::Denom => {
				to_json_binary(
					&QueryResponseDenom {
						votes: votes_denom(&env)
					}
				)?
			},
			QueryMsg::ProposalInfo { id } => {
				to_json_binary(
					&get_transaction_proposal_stored_vec(
						deps.storage
					)?.get(id)?
				)?
			},
			QueryMsg::GetProposals { skip, limit, descending } => {
				let app_config = CourtAppConfig::load_non_empty(deps.storage)?;
				let total_supply = total_supply_workaround(deps.storage, &votes_denom(&env));
				to_json_binary(
					&if descending {
						get_transaction_proposal_stored_vec(
							deps.storage
						)?.into_iter()
							.enumerate()
							.rev()
							.skip(skip as usize)
							.take(limit as usize)
							.map(|(index, info_result)| {
								let info = info_result?;
								Ok(
									QueryResponseTransactionProposal {
									proposal_id: index as u32,
									status: info.status(
										env.block.time.millis(),
										&total_supply,
										&app_config
									),
									info
								}
								)
							}).collect::<StdResult<Vec<QueryResponseTransactionProposal>>>()?
					}else{
						get_transaction_proposal_stored_vec(
							deps.storage
						)?.into_iter()
							.enumerate()
							.skip(skip as usize)
							.take(limit as usize)
							.map(|(index, info_result)| {
								let info = info_result?;
								Ok(
									QueryResponseTransactionProposal {
									proposal_id: index as u32,
									status: info.status(
										env.block.time.millis(),
										&total_supply,
										&app_config
									),
									info
								}
								)
							}).collect::<StdResult<Vec<QueryResponseTransactionProposal>>>()?
					}
				)?
			},
			QueryMsg::UserStats { user } => {
				to_json_binary(
					&get_user_stats_store(
						deps.storage
					)?.get(&user)?.unwrap_or_default()
				)?
			},
			QueryMsg::UserVoteInfo { user, proposal_id } => {
				to_json_binary(
					&get_user_vote_info_store(
						deps.storage
					)?.get(&(user, proposal_id))?
				)?
			},
			QueryMsg::GetUserVotes { user, skip, limit, descending } => {
				to_json_binary(
					&if descending {
						get_all_user_vote_info_iter(deps.storage, user)?
							.rev()
							.skip(skip as usize)
							.take(limit as usize)
							.map(|(proposal_id, info)| {
								QueryResponseUserVote { proposal_id, info }
							})
							.collect::<Vec<QueryResponseUserVote>>()
					}else{
						get_all_user_vote_info_iter(deps.storage, user)?
							.skip(skip as usize)
							.take(limit as usize)
							.map(|(proposal_id, info)| {
								QueryResponseUserVote { proposal_id, info }
							})
							.collect::<Vec<QueryResponseUserVote>>()
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
