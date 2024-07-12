use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult};
use crownfi_cw_common::{
	data_types::canonical_addr::SeiCanonicalAddr, env::MinimalEnvInfo, extentions::timestamp::TimestampExtentions,
	storage::item::StoredItem,
};
use cw2::set_contract_version;
use cw_utils::nonpayable;
use sei_cosmwasm::{SeiMsg, SeiQueryWrapper};

use crate::{
	error::CourtContractError,
	msg::{
		CourtAdminExecuteMsg, CourtExecuteMsg, CourtInstantiateMsg, CourtMigrateMsg, CourtQueryMsg, CourtQueryResponseDenom, CourtQueryResponseTotalSupply, CourtQueryResponseTransactionProposal, CourtQueryResponseUserVote, CourtQueryUserWithActiveProposal
	},
	proposed_msg::ProposedCourtMsgJsonable,
	state::{
		app::{
			get_transaction_proposal_info_vec, get_transaction_proposal_messages_vec, CourtAppConfig,
			CourtAppConfigJsonable,
		},
		user::{
			get_all_proposal_user_votes, get_all_user_active_proposal_ids, get_proposal_user_vote_store,
			get_user_active_proposal_id_set, get_user_stats_store, CourtUserStatsJsonable, CourtUserVoteInfoJsonable,
		},
	},
	workarounds::{mint_to_workaround, total_supply_workaround},
};

use self::{
	admin::AdminMsgExecutor,
	permissionless::{process_deactivate_votes, process_execute_proposal},
	shares::{votes_denom, VOTES_SUBDENOM},
	user::{process_propose_transaction, process_stake, process_unstake, process_vote},
};

pub mod admin;
pub mod permissionless;
pub mod shares;
pub mod user;

pub const COURT_CONTRACT_NAME: &str = "court-coordinator-contract";
pub const COURT_CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
#[inline]
pub fn instantiate(
	deps: DepsMut<SeiQueryWrapper>,
	env: Env,
	msg_info: MessageInfo,
	msg: CourtInstantiateMsg,
) -> Result<Response<SeiMsg>, CourtContractError> {
	nonpayable(&msg_info)?;
	set_contract_version(deps.storage, COURT_CONTRACT_NAME, COURT_CONTRACT_VERSION)?;
	CourtAppConfig::try_from(&CourtAppConfigJsonable {
		allow_new_proposals: true,
		minimum_vote_proposal_percent: msg.minimum_vote_proposal_percent,
		minimum_vote_turnout_percent: msg.minimum_vote_turnout_percent,
		minimum_vote_pass_percent: msg.minimum_vote_pass_percent,
		max_proposal_expiry_time_seconds: msg.max_proposal_expiry_time_seconds,
		execution_expiry_time_seconds: msg.execution_expiry_time_seconds,
		last_config_change_timestamp_ms: env.block.time.millis(),
		admin: msg.admin,
	})?
	.save()?;
	let new_denom = votes_denom(&env);
	let vote_share_symbol_lowercase = msg.vote_share_symbol.to_ascii_lowercase();
	let vote_share_symbol_uppercase = msg.vote_share_symbol.to_ascii_uppercase();
	Ok(mint_to_workaround(
		Response::new()
			.add_message(SeiMsg::CreateDenom {
				subdenom: VOTES_SUBDENOM.to_string(),
			})
			.add_message(SeiMsg::SetMetadata {
				metadata: sei_cosmwasm::Metadata {
					description: msg.vote_share_description,
					denom_units: vec![
						sei_cosmwasm::DenomUnit {
							denom: new_denom.clone(),
							exponent: 0,
							aliases: vec![
								format!("u{vote_share_symbol_lowercase}"),
								format!("micro{vote_share_symbol_lowercase}"),
							],
						},
						sei_cosmwasm::DenomUnit {
							denom: format!("m{vote_share_symbol_lowercase}"),
							exponent: 3,
							aliases: vec![format!("milli{vote_share_symbol_lowercase}")],
						},
						sei_cosmwasm::DenomUnit {
							denom: vote_share_symbol_lowercase.clone(),
							exponent: 6,
							aliases: vec![],
						},
					],
					base: new_denom.clone(),
					display: vote_share_symbol_lowercase.clone(),
					name: msg.vote_share_name,
					symbol: vote_share_symbol_uppercase,
				},
			}),
		&new_denom,
		&msg.shares_mint_receiver,
		msg.shares_mint_amount.u128(),
	)?)
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
#[inline]
pub fn execute(
	deps: DepsMut<SeiQueryWrapper>,
	env: Env,
	msg_info: MessageInfo,
	msg: CourtExecuteMsg,
) -> Result<Response<SeiMsg>, CourtContractError> {
	let env_info = MinimalEnvInfo::from_deps_mut(deps, env);
	Ok(match msg {
		CourtExecuteMsg::Admin(admin_msg) => {
			//deps.querier.query(request)
			let mut admin_executor = AdminMsgExecutor::new(env_info, &msg_info)?;
			match admin_msg {
				CourtAdminExecuteMsg::ChangeConfig {
					minimum_vote_proposal_percent,
					minimum_vote_turnout_percent,
					minimum_vote_pass_percent,
					max_proposal_expiry_time_seconds,
					execution_expiry_time_seconds,
				} => admin_executor.process_change_config(
					&msg_info,
					minimum_vote_proposal_percent,
					minimum_vote_turnout_percent,
					minimum_vote_pass_percent,
					max_proposal_expiry_time_seconds,
					execution_expiry_time_seconds,
				)?,
				CourtAdminExecuteMsg::ChangeAdmin { admin } => admin_executor.process_change_admin(&msg_info, admin)?,
				CourtAdminExecuteMsg::AllowNewProposals { allowed } => {
					admin_executor.process_allow_new_proposals(&msg_info, allowed)?
				}
				CourtAdminExecuteMsg::MintShares { receiver, amount } => {
					admin_executor.process_mint_shares(&msg_info, receiver, amount)?
				}
			}
		}
		CourtExecuteMsg::Stake => process_stake(env_info, msg_info)?,
		CourtExecuteMsg::Unstake => process_unstake(env_info, msg_info)?,
		CourtExecuteMsg::Vote { id, vote } => process_vote(env_info, msg_info, id, vote)?,
		CourtExecuteMsg::DeactivateVotes { user, limit } => process_deactivate_votes(
			env_info,
			msg_info,
			user.and_then(|v| SeiCanonicalAddr::try_from(&v).ok()),
			limit,
		)?,
		CourtExecuteMsg::ProposeTransaction {
			msgs,
			expiry_time_seconds,
		} => process_propose_transaction(env_info, msg_info, msgs, expiry_time_seconds)?,
		CourtExecuteMsg::ExecuteProposal { id } => process_execute_proposal(env_info, msg_info, id)?,
	})
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
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

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
#[inline]
pub fn query(_deps: Deps, env: Env, msg: CourtQueryMsg) -> Result<Binary, CourtContractError> {
	Ok(match msg {
		CourtQueryMsg::Config => to_json_binary(&CourtAppConfigJsonable::try_from(
			CourtAppConfig::load_non_empty()?.as_ref(),
		)?)?,
		CourtQueryMsg::Denom => to_json_binary(&CourtQueryResponseDenom {
			votes: votes_denom(&env),
		})?,
		CourtQueryMsg::TotalSupply => to_json_binary(&CourtQueryResponseTotalSupply {
			votes: total_supply_workaround(&votes_denom(&env))
		})?,
		CourtQueryMsg::ProposalAmount => to_json_binary(&(get_transaction_proposal_info_vec().len() as u32))?,
		CourtQueryMsg::GetProposal { id } => {
			let app_config = CourtAppConfig::load_non_empty()?;
			let total_supply = total_supply_workaround(&votes_denom(&env));
			let proposal_msg_vec = get_transaction_proposal_messages_vec();
			to_json_binary(
				&get_transaction_proposal_info_vec()
					.get(id)?
					.map(|info| -> Result<_, StdError> {
						Ok(CourtQueryResponseTransactionProposal {
							proposal_id: id as u32,
							status: info.status(env.block.time.millis(), total_supply.u128(), &app_config),
							info: info.as_ref().try_into()?,
							messages: proposal_msg_vec
								.get(id as u32)?
								.unwrap_or_default()
								.into_inner()
								.into_iter()
								.map(|v| {
									let mut v_jsonable = ProposedCourtMsgJsonable::try_from(v)?;
									v_jsonable.make_pretty()?;
									Ok(v_jsonable)
								})
								.collect::<Result<Vec<_>, StdError>>()?,
						})
					})
					.transpose()?,
			)?
		}
		CourtQueryMsg::GetProposals {
			skip,
			limit,
			descending,
		} => {
			let app_config = CourtAppConfig::load_non_empty()?;
			let total_supply = total_supply_workaround(&votes_denom(&env));
			let proposal_msg_vec = get_transaction_proposal_messages_vec();

			let iter = get_transaction_proposal_info_vec()
				.into_iter()
				.enumerate()
				.map(|(index, info_result)| {
					let info = info_result?;
					Ok(CourtQueryResponseTransactionProposal {
						proposal_id: index as u32,
						status: info.status(env.block.time.millis(), total_supply.u128(), &app_config),
						info: info.as_ref().try_into()?,
						messages: proposal_msg_vec
							.get(index as u32)?
							.unwrap_or_default()
							.into_inner()
							.into_iter()
							.map(|v| {
								let mut v_jsonable = ProposedCourtMsgJsonable::try_from(v)?;
								v_jsonable.make_pretty()?;
								Ok(v_jsonable)
							})
							.collect::<Result<Vec<_>, StdError>>()?,
					})
				});
			to_json_binary(&if descending {
				iter.rev()
					.skip(skip.unwrap_or(0) as usize)
					.take(limit.unwrap_or(u32::MAX) as usize)
					.collect::<StdResult<Vec<CourtQueryResponseTransactionProposal>>>()?
			} else {
				iter.skip(skip.unwrap_or(0) as usize)
					.take(limit.unwrap_or(u32::MAX) as usize)
					.collect::<StdResult<Vec<CourtQueryResponseTransactionProposal>>>()?
			})?
		}
		CourtQueryMsg::UserStats { user } => {
			let user = SeiCanonicalAddr::try_from(&user)?;
			to_json_binary(&CourtUserStatsJsonable::try_from(
				get_user_stats_store().get(&user)?.unwrap_or_default().as_ref(),
			)?)?
		}
		CourtQueryMsg::UserVoteInfo { user, proposal_id } => {
			let user = SeiCanonicalAddr::try_from(&user)?;
			to_json_binary(&CourtUserVoteInfoJsonable::try_from(
				get_proposal_user_vote_store()
					.get(&(proposal_id, user))?
					.unwrap_or_default()
					.as_ref(),
			)?)?
		}
		CourtQueryMsg::GetUsersWithActiveProposals {
			after,
			limit,
			descending,
		} => {
			let iter = get_user_active_proposal_id_set()
				.iter_range(
					after
						.as_ref()
						.filter(|_| !descending)
						.map(|after| -> Result<_, StdError> { Ok(((&after.user).try_into()?, after.proposal_id)) })
						.transpose()?,
					after
						.as_ref()
						.filter(|_| descending)
						.map(|after| -> Result<_, StdError> { Ok(((&after.user).try_into()?, after.proposal_id)) })
						.transpose()?,
				)?
				.map(|(canon_addr, proposal_id)| -> Result<_, StdError> {
					Ok(CourtQueryUserWithActiveProposal {
						user: canon_addr.try_into()?,
						proposal_id,
					})
				});
			to_json_binary(&if descending {
				iter.rev()
					.take(limit.unwrap_or(u32::MAX) as usize)
					.collect::<Result<Vec<CourtQueryUserWithActiveProposal>, _>>()?
			} else if after.is_some() {
				// "start" is inclusive while "end" is exclusive
				iter.skip(1)
					.take(limit.unwrap_or(u32::MAX) as usize)
					.collect::<Result<Vec<CourtQueryUserWithActiveProposal>, _>>()?
			} else {
				iter.take(limit.unwrap_or(u32::MAX) as usize)
					.collect::<Result<Vec<CourtQueryUserWithActiveProposal>, _>>()?
			})?
		}
		CourtQueryMsg::GetUserActiveProposals {
			user,
			skip,
			limit,
			descending,
		} => {
			let iter = get_all_user_active_proposal_ids(SeiCanonicalAddr::try_from(user)?)?;
			to_json_binary(&if descending {
				iter.rev()
					.skip(skip.unwrap_or(0) as usize)
					.take(limit.unwrap_or(u32::MAX) as usize)
					.collect::<Vec<u32>>()
			} else {
				iter.skip(skip.unwrap_or(0) as usize)
					.take(limit.unwrap_or(u32::MAX) as usize)
					.collect::<Vec<u32>>()
			})?
		}
		CourtQueryMsg::GetProposalUserVotes {
			proposal_id,
			after,
			limit,
			descending,
		} => {
			let iter = get_all_proposal_user_votes(
				proposal_id,
				after
					.as_ref()
					.filter(|_| descending)
					.map(|addr| addr.try_into())
					.transpose()?,
				after
					.as_ref()
					.filter(|_| !descending)
					.map(|addr| addr.try_into())
					.transpose()?,
			)?
			.map(|(addr, info)| {
				Ok(CourtQueryResponseUserVote {
					user: addr.try_into()?,
					info: info.as_ref().try_into()?,
				})
			});
			to_json_binary(&if descending {
				iter.rev()
					.take(limit.unwrap_or(u32::MAX) as usize)
					.collect::<StdResult<Vec<CourtQueryResponseUserVote>>>()?
			} else if after.is_some() {
				// "start" is inclusive while "end" is exclusive
				iter.skip(1)
					.take(limit.unwrap_or(u32::MAX) as usize)
					.collect::<StdResult<Vec<CourtQueryResponseUserVote>>>()?
			} else {
				iter.take(limit.unwrap_or(u32::MAX) as usize)
					.collect::<StdResult<Vec<CourtQueryResponseUserVote>>>()?
			})?
		}
	})
}
