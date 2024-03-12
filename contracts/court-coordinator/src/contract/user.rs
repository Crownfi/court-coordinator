use cosmwasm_std::{MessageInfo, Response, Event, Uint128, BankMsg, StdError};
use crownfi_cw_common::{data_types::canonical_addr::SeiCanonicalAddr, env::ClonableEnvInfoMut, extentions::timestamp::TimestampExtentions};
use sei_cosmwasm::{SeiMsg, SeiQueryWrapper};

use crate::{error::CourtContractError, proposed_msg::ProposedCourtMsg, state::{app::{get_transaction_proposal_info_vec_mut, get_transaction_proposal_messages_vec_mut, CourtAppConfig, TransactionProposalInfo, TransactionProposalStatus}, user::{get_all_user_votes, get_user_stats_store_mut, get_user_vote_info_store_mut, CourtUserVoteInfoJsonable}}, workarounds::total_supply_workaround};

use super::{enforce_single_payment, shares::{votes_denom, votes_coin}, enforce_unfunded};


pub fn process_stake(
	env_info: ClonableEnvInfoMut<SeiQueryWrapper>,
	msg_info: MessageInfo
) -> Result<Response<SeiMsg>, CourtContractError> {
	let msg_sender = SeiCanonicalAddr::from_addr_using_api(&msg_info.sender, *env_info.api)?;
	let user_payment = enforce_single_payment(&msg_info, &votes_denom(&env_info.env))?;
	let user_stats_map = get_user_stats_store_mut(env_info.storage.clone())?;

	let mut user_stats = user_stats_map.get_or_default_autosaving(&msg_sender)?;
	user_stats.staked_votes = user_stats.staked_votes.checked_add(user_payment.amount.into()).unwrap();

	Ok(
		Response::new()
			.add_event(
				Event::new("stake")
					.add_attribute("user", &msg_info.sender)
					.add_attribute("new_amount", user_payment.amount)
					.add_attribute("user_total", Uint128::from(user_stats.staked_votes))
			)
	)
}

pub fn process_unstake(
	env_info: ClonableEnvInfoMut<SeiQueryWrapper>,
	msg_info: MessageInfo
) -> Result<Response<SeiMsg>, CourtContractError> {
	enforce_unfunded(&msg_info)?;
	let msg_sender = SeiCanonicalAddr::from_addr_using_api(&msg_info.sender, *env_info.api)?;
	let has_active_votes = get_all_user_votes(
		*env_info.storage.borrow(),
		msg_sender
	)?.next().is_some();
	
	if has_active_votes {
		return Err(CourtContractError::VotesActive);
	}

	let user_stats_map = get_user_stats_store_mut(env_info.storage.clone())?;

	let mut user_stats = user_stats_map.get_or_default_autosaving(&msg_sender)?;
	let unstake_amount = user_stats.staked_votes;
	user_stats.staked_votes = 0;

	Ok(
		Response::new()
			.add_event(
				Event::new("unstake")
					.add_attribute("user", &msg_info.sender)
					.add_attribute("unstake_amount", Uint128::from(unstake_amount))
			)
			.add_message(
				BankMsg::Send {
					to_address: msg_info.sender.to_string(),
					amount: vec![votes_coin(&env_info.env, unstake_amount)]
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
	let msg_sender = SeiCanonicalAddr::from_addr_using_api(&msg_info.sender, *env_info.api)?;
	let app_config = CourtAppConfig::load_non_empty(*env_info.storage.borrow())?;
	let token_supply = total_supply_workaround(*env_info.storage.borrow(), &votes_denom(&env_info.env));
	let user_stats = get_user_stats_store_mut(env_info.storage.clone())?
		.get(&msg_sender)?
		.unwrap_or_default();

	let proposals = get_transaction_proposal_info_vec_mut(env_info.storage.clone())?;
	let mut proposal = proposals.get(proposal_id)?.ok_or(
		StdError::not_found(format!("Proposal {} does not exist", proposal_id))
	)?;
	proposal.status(env_info.env.block.time.millis(), token_supply.u128(), &app_config)
		.enforce_status(TransactionProposalStatus::Pending)?;

	let mut user_vote_info = get_user_vote_info_store_mut(env_info.storage.clone())?
		.get_or_default_autosaving(&(msg_sender, proposal_id))?;
	
	if user_vote_info.active_votes == 0 {
		if user_stats.staked_votes == 0 {
			return Err(CourtContractError::NoStakedVotes);
		}
		user_vote_info.active_votes = user_stats.staked_votes;
		user_vote_info.set_voted_for(approve);
		if approve {
			proposal.votes_for = proposal.votes_for.checked_add(user_stats.staked_votes).unwrap();
		} else {
			proposal.votes_against = proposal.votes_against.checked_add(user_stats.staked_votes).unwrap();
		}
	} else {
		if user_vote_info.active_votes == user_stats.staked_votes {
			return Err(CourtContractError::AlreadyVoted);
		}
		if user_vote_info.voted_for() != approve {
			return Err(CourtContractError::VotingForBothSides);
		}
		if user_vote_info.active_votes > user_stats.staked_votes {
			unreachable!("User votes associated with proposal should never be more than the total votes they have");
		}
		user_vote_info.active_votes = user_stats.staked_votes;

		// We just checked ">" above.
		let diff = user_stats.staked_votes - user_vote_info.active_votes;
		if approve {
			proposal.votes_for = proposal.votes_for.checked_add(diff).unwrap();
		} else {
			proposal.votes_against = proposal.votes_against.checked_add(diff).unwrap();
		}
	}

	proposals.set(proposal_id, &proposal)?;
	Ok(
		Response::new()
			.add_event(
				Event::new("vote")
					.add_attribute("proposal_id", proposal_id.to_string())
					.add_attribute("voter", msg_info.sender)
					.add_attribute("votes", Uint128::from(user_stats.staked_votes))
					.add_attribute("approve", approve.to_string())
			)
	)
}


pub fn process_propose_transaction(
	env_info: ClonableEnvInfoMut<SeiQueryWrapper>,
	msg_info: MessageInfo,
	msgs: Vec<ProposedCourtMsg>,
	expiry_time_seconds: u32
) -> Result<Response<SeiMsg>, CourtContractError> {
	enforce_unfunded(&msg_info)?;
	let proposer = SeiCanonicalAddr::from_addr_using_api(&msg_info.sender, *env_info.api)?;
	let proposer_addr = msg_info.sender;

	let app_config = CourtAppConfig::load_non_empty(*env_info.storage.borrow())?;
	let token_supply = total_supply_workaround(*env_info.storage.borrow(), &votes_denom(&env_info.env));
	let user_stats = get_user_stats_store_mut(env_info.storage.clone())?
		.get(&proposer)?
		.unwrap_or_default();

	if msgs.len() == 0 {
		return Err(CourtContractError::EmptyProposal);
	}
	if expiry_time_seconds > app_config.max_proposal_expiry_time_seconds {
		return Err(CourtContractError::ProposalLivesTooLong);
	}
	if !app_config.allow_new_proposals() {
		return Err(CourtContractError::NewProposalsNotAllowed);
	}
	if
		u8::try_from(
			user_stats.staked_votes
				.checked_mul(100u128.into()).unwrap()
				.checked_div(token_supply.into()).unwrap()
		).unwrap() < app_config.minimum_vote_proposal_percent {
	} else {
		return Err(CourtContractError::InsufficientVotesForProposal);
	}

	let mut proposal_infos = get_transaction_proposal_info_vec_mut(env_info.storage.clone())?;
	let mut proposal_msgs = get_transaction_proposal_messages_vec_mut(env_info.storage.clone())?;
	let new_proposal = TransactionProposalInfo::new(
		proposer.clone(),
		user_stats.staked_votes,
		env_info.env.block.time.plus_seconds(expiry_time_seconds as u64).millis()
	);
	let new_proposal_id = proposal_infos.len();
	proposal_infos.push(&new_proposal)?;
	proposal_msgs.push(&msgs)?;
	assert_eq!(proposal_infos.len(), proposal_msgs.len());

	get_user_vote_info_store_mut(env_info.storage.clone())?.set(
		&(proposer.clone(), new_proposal_id),
		&CourtUserVoteInfoJsonable {
			active_votes: user_stats.staked_votes.into(),
			voted_for: true
		}.into_storable(*env_info.api)?
	)?;
	
	Ok(
		Response::new()
			.add_event(
				Event::new("proposal")
					.add_attribute("proposal_id", new_proposal_id.to_string())
					.add_attribute("proposer", proposer_addr.clone())
			)
			.add_event(
				Event::new("vote")
					.add_attribute("proposal_id", new_proposal_id.to_string())
					.add_attribute("voter", proposer_addr)
					.add_attribute("votes", Uint128::from(user_stats.staked_votes))
					.add_attribute("approve", true.to_string())
			)
			
	)
}
