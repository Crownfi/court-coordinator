use cosmwasm_std::{MessageInfo, Response, Event, Uint128, BankMsg, StdError, CosmosMsg};
use crownfi_cw_common::{env::ClonableEnvInfoMut, extentions::timestamp::TimestampExtentions};
use sei_cosmwasm::{SeiMsg, SeiQueryWrapper};

use crate::{error::CourtContractError, state::{user::{get_user_stats_store_mut, CourtUserVoteInfo, get_all_user_vote_info_iter, get_user_vote_info_store_mut}, app::{get_transaction_proposal_stored_vec_mut, CourtAppConfig, TransactionProposalStatus, TransactionProposalInfo}}, workarounds::total_supply_workaround};

use super::{enforce_single_payment, shares::{votes_denom, votes_coin}, enforce_unfunded};


pub fn process_stake(
	env_info: ClonableEnvInfoMut<SeiQueryWrapper>,
	msg_info: MessageInfo
) -> Result<Response<SeiMsg>, CourtContractError> {
	
	let user_payment = enforce_single_payment(&msg_info, &votes_denom(&env_info.env))?;
	let user_stats_map = get_user_stats_store_mut(env_info.storage.clone())?;

	let mut user_stats = user_stats_map.get_or_default_autosaving(&msg_info.sender)?;
	user_stats.staked_votes = user_stats.staked_votes.checked_add(user_payment.amount)?;

	Ok(
		Response::new()
			.add_event(
				Event::new("stake")
					.add_attribute("user", &msg_info.sender)
					.add_attribute("new_amount", user_payment.amount)
					.add_attribute("user_total", user_stats.staked_votes)
			)
	)
}

pub fn process_unstake(
	env_info: ClonableEnvInfoMut<SeiQueryWrapper>,
	msg_info: MessageInfo
) -> Result<Response<SeiMsg>, CourtContractError> {
	enforce_unfunded(&msg_info)?;

	let has_active_votes = get_all_user_vote_info_iter(
		*env_info.storage.borrow(),
		msg_info.sender.clone()
	)?.next().is_some();
	
	if has_active_votes {
		return Err(CourtContractError::VotesActive);
	}

	let user_stats_map = get_user_stats_store_mut(env_info.storage.clone())?;

	let mut user_stats = user_stats_map.get_or_default_autosaving(&msg_info.sender)?;
	let unstake_amount = user_stats.staked_votes;
	user_stats.staked_votes = Uint128::zero();

	Ok(
		Response::new()
			.add_event(
				Event::new("unstake")
					.add_attribute("user", &msg_info.sender)
					.add_attribute("unstake_amount", unstake_amount)
			)
			.add_message(
				BankMsg::Send {
					to_address: msg_info.sender.to_string(),
					amount: vec![votes_coin(&env_info.env, unstake_amount.u128())]
				}
			)
	)
}

pub fn process_vote(
	env_info: ClonableEnvInfoMut<SeiQueryWrapper>,
	msg_info: MessageInfo,
	proposal_id: u32,
	approve: bool
) -> Result<Response<SeiMsg>, CourtContractError> {
	enforce_unfunded(&msg_info)?;
	let app_config = CourtAppConfig::load_non_empty(*env_info.storage.borrow())?;
	let token_supply = total_supply_workaround(*env_info.storage.borrow(), &votes_denom(&env_info.env));
	let user_stats = get_user_stats_store_mut(env_info.storage.clone())?
		.get(&msg_info.sender)?
		.unwrap_or_default();

	let proposals = get_transaction_proposal_stored_vec_mut(env_info.storage.clone())?;
	let mut proposal = proposals.get(proposal_id)?.ok_or(
		StdError::not_found(format!("Proposal {} does not exist", proposal_id))
	)?;
	proposal.status(env_info.env.block.time.millis(), &token_supply, &app_config).enforce_status(TransactionProposalStatus::Pending)?;

	let mut user_vote_info = get_user_vote_info_store_mut(env_info.storage.clone())?
		.get_or_default_autosaving(&(msg_info.sender.clone(), proposal_id))?;
	
	if user_vote_info.active_votes.is_zero() {
		if user_stats.staked_votes.is_zero() {
			return Err(CourtContractError::NoStakedVotes);
		}
		user_vote_info.active_votes = user_stats.staked_votes;
		user_vote_info.voted_for = approve;
		if approve {
			proposal.votes_for = proposal.votes_for.checked_add(user_stats.staked_votes)?;
		} else {
			proposal.votes_against = proposal.votes_against.checked_add(user_stats.staked_votes)?;
		}
	} else if user_vote_info.active_votes == user_stats.staked_votes {
		return Err(CourtContractError::AlreadyVoted);
	} else if user_vote_info.voted_for != approve {
		return Err(CourtContractError::VotingForBothSides);
	} else if user_vote_info.active_votes < user_stats.staked_votes {
		user_vote_info.active_votes = user_stats.staked_votes;
		// Panics on underflow, we also just checked "<" above.
		let diff = user_stats.staked_votes - user_vote_info.active_votes;
		if approve {
			proposal.votes_for = proposal.votes_for.checked_add(diff)?;
		} else {
			proposal.votes_against = proposal.votes_against.checked_add(diff)?;
		}
	} else {
		unreachable!("User votes associated with proposal should never be below the total votes they have");
	}

	proposals.set(proposal_id, &proposal)?;
	Ok(
		Response::new()
			.add_event(
				Event::new("vote")
					.add_attribute("proposal_id", proposal_id.to_string())
					.add_attribute("voter", msg_info.sender)
					.add_attribute("votes", user_stats.staked_votes)
					.add_attribute("approve", approve.to_string())
			)
	)
}


pub fn process_propose_transaction(
	env_info: ClonableEnvInfoMut<SeiQueryWrapper>,
	msg_info: MessageInfo,
	msgs: Vec<CosmosMsg<SeiMsg>>,
	expiry_time_seconds: u32
) -> Result<Response<SeiMsg>, CourtContractError> {
	enforce_unfunded(&msg_info)?;
	let proposer = msg_info.sender;

	let app_config = CourtAppConfig::load_non_empty(*env_info.storage.borrow())?;
	let token_supply = total_supply_workaround(*env_info.storage.borrow(), &votes_denom(&env_info.env));
	let user_stats = get_user_stats_store_mut(env_info.storage.clone())?
		.get(&proposer)?
		.unwrap_or_default();

	if msgs.len() == 0 {
		return Err(CourtContractError::EmptyProposal);
	}
	if expiry_time_seconds > app_config.max_expiry_time_seconds {
		return Err(CourtContractError::ProposalLivesTooLong);
	}
	if !app_config.allow_new_proposals {
		return Err(CourtContractError::NewProposalsNotAllowed);
	}
	if
		u8::try_from(
			user_stats.staked_votes
				.checked_mul(100u128.into())?
				.checked_div(token_supply)?
				.u128()
		).unwrap() < app_config.minimum_vote_proposal_percent {
	} else {
		return Err(CourtContractError::InsufficientVotesForProposal);
	}

	let mut proposals = get_transaction_proposal_stored_vec_mut(env_info.storage.clone())?;
	let new_proposal = TransactionProposalInfo::new(
		proposer.clone(),
		user_stats.staked_votes,
		msgs,
		env_info.env.block.time.plus_seconds(expiry_time_seconds as u64).millis()
	);
	let new_proposal_id = proposals.len();
	proposals.push(&new_proposal)?;

	get_user_vote_info_store_mut(env_info.storage.clone())?.set(
		&(proposer.clone(), new_proposal_id),
		&CourtUserVoteInfo {
			active_votes: user_stats.staked_votes,
			voted_for: true
		}
	)?;
	
	Ok(
		Response::new()
			.add_event(
				Event::new("proposal")
					.add_attribute("proposal_id", new_proposal_id.to_string())
					.add_attribute("proposer", proposer.clone())
			)
			.add_event(
				Event::new("vote")
					.add_attribute("proposal_id", new_proposal_id.to_string())
					.add_attribute("voter", proposer)
					.add_attribute("votes", user_stats.staked_votes)
					.add_attribute("approve", true.to_string())
			)
			
	)
}
