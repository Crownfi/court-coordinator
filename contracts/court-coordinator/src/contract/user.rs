use cosmwasm_std::{MessageInfo, Response, Event, Uint128, BankMsg, StdError};
use crownfi_cw_common::{data_types::canonical_addr::SeiCanonicalAddr, env::MinimalEnvInfo, extentions::timestamp::TimestampExtentions};
use cw_utils::{must_pay, nonpayable};
use sei_cosmwasm::{SeiMsg, SeiQueryWrapper};

use crate::{error::CourtContractError, proposed_msg::ProposedCourtMsg, state::{app::{get_transaction_proposal_info_vec, get_transaction_proposal_messages_vec, CourtAppConfig, TransactionProposalInfo, TransactionProposalStatus}, user::{get_all_user_active_proposal_ids, get_proposal_user_vote_store, get_user_active_proposal_id_set, get_user_stats_store, CourtUserVoteInfoJsonable, CourtUserVoteStatus}}, workarounds::total_supply_workaround};

use super::shares::{votes_denom, votes_coin};


pub fn process_stake(
	env_info: MinimalEnvInfo<SeiQueryWrapper>,
	msg_info: MessageInfo
) -> Result<Response<SeiMsg>, CourtContractError> {
	let msg_sender = SeiCanonicalAddr::try_from(&msg_info.sender)?;
	let user_payment_amount = must_pay(&msg_info, &votes_denom(&env_info.env))?;
	let user_stats_map = get_user_stats_store();

	let mut user_stats = user_stats_map.get_or_default_autosaving(&msg_sender)?;
	user_stats.staked_votes = user_stats.staked_votes.checked_add(user_payment_amount.into()).unwrap();

	Ok(
		Response::new()
			.add_event(
				Event::new("stake")
					.add_attribute("user", &msg_info.sender)
					.add_attribute("new_amount", user_payment_amount)
					.add_attribute("user_total", Uint128::from(user_stats.staked_votes))
			)
	)
}

pub fn process_unstake(
	env_info: MinimalEnvInfo<SeiQueryWrapper>,
	msg_info: MessageInfo
) -> Result<Response<SeiMsg>, CourtContractError> {
	nonpayable(&msg_info)?;
	let msg_sender = SeiCanonicalAddr::try_from(&msg_info.sender)?;
	let has_active_votes = get_all_user_active_proposal_ids(
		msg_sender
	)?.next().is_some();
	
	if has_active_votes {
		return Err(CourtContractError::VotesActive);
	}

	let user_stats_map = get_user_stats_store();

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
	env_info: MinimalEnvInfo<SeiQueryWrapper>,
	msg_info: MessageInfo,
	proposal_id: u32,
	approve: CourtUserVoteStatus
) -> Result<Response<SeiMsg>, CourtContractError> {
	nonpayable(&msg_info)?;
	let msg_sender = SeiCanonicalAddr::try_from(&msg_info.sender)?;
	let app_config = CourtAppConfig::load_non_empty()?;
	let token_supply = total_supply_workaround(&votes_denom(&env_info.env));
	let user_stats = get_user_stats_store()
		.get(&msg_sender)?
		.unwrap_or_default();

	let proposals = get_transaction_proposal_info_vec();
	let mut proposal = proposals.get(proposal_id)?.ok_or(
		StdError::not_found(format!("Proposal {} does not exist", proposal_id))
	)?;
	proposal.status(env_info.env.block.time.millis(), token_supply.u128(), &app_config)
		.enforce_status(TransactionProposalStatus::Pending)?;

	let user_active_proposals = get_user_active_proposal_id_set();
	let mut user_vote_info = get_proposal_user_vote_store()
		.get_or_default_autosaving(&(proposal_id, msg_sender))?;
	
	if user_stats.staked_votes == 0 {
		return Err(CourtContractError::NoStakedVotes);
	}
	if user_vote_info.active_votes != 0 {
		if
			user_vote_info.active_votes == user_stats.staked_votes &&
			user_vote_info.vote() == approve
		{
			return Err(CourtContractError::AlreadyVoted);
		}
		// User is either adding votes or chainging their vote, so we gotta take away the old votes first
		match user_vote_info.vote() {
			CourtUserVoteStatus::Oppose => {
				proposal.votes_against = proposal.votes_against.checked_sub(user_vote_info.active_votes).unwrap();
			},
			CourtUserVoteStatus::Approve => {
				proposal.votes_for = proposal.votes_for.checked_sub(user_vote_info.active_votes).unwrap();	
			},
			CourtUserVoteStatus::Abstain => {
				proposal.votes_abstain = proposal.votes_against.checked_sub(user_vote_info.active_votes).unwrap();	
			},
		}
	}
	user_vote_info.active_votes = user_stats.staked_votes;
	user_vote_info.set_vote(approve);
	match approve {
		CourtUserVoteStatus::Oppose => {
			proposal.votes_against = proposal.votes_against.checked_add(user_stats.staked_votes).unwrap();
		},
		CourtUserVoteStatus::Approve => {
			proposal.votes_for = proposal.votes_for.checked_add(user_stats.staked_votes).unwrap();	
		},
		CourtUserVoteStatus::Abstain => {
			proposal.votes_abstain = proposal.votes_against.checked_add(user_stats.staked_votes).unwrap();	
		},
	}
	user_active_proposals.add(&(msg_sender, proposal_id))?;
	proposals.set(proposal_id, &proposal)?;
	Ok(
		Response::new()
			.add_event(
				Event::new("vote")
					.add_attribute("proposal_id", proposal_id.to_string())
					.add_attribute("voter", msg_info.sender)
					.add_attribute("votes", Uint128::from(user_stats.staked_votes))
					.add_attribute("vote", approve.to_string())
			)
	)
}


pub fn process_propose_transaction(
	env_info: MinimalEnvInfo<SeiQueryWrapper>,
	msg_info: MessageInfo,
	msgs: Vec<ProposedCourtMsg>,
	expiry_time_seconds: u32
) -> Result<Response<SeiMsg>, CourtContractError> {
	nonpayable(&msg_info)?;
	let proposer = SeiCanonicalAddr::try_from(&msg_info.sender)?;
	let proposer_addr = msg_info.sender;

	let app_config = CourtAppConfig::load_non_empty()?;
	let token_supply = total_supply_workaround(&votes_denom(&env_info.env));
	let user_stats = get_user_stats_store()
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

	let mut proposal_infos = get_transaction_proposal_info_vec();
	let mut proposal_msgs = get_transaction_proposal_messages_vec();
	let new_proposal = TransactionProposalInfo::new(
		proposer.clone(),
		user_stats.staked_votes,
		env_info.env.block.time.plus_seconds(expiry_time_seconds as u64).millis()
	);
	let new_proposal_id = proposal_infos.len();
	proposal_infos.push(&new_proposal)?;
	proposal_msgs.push(&msgs)?;
	assert_eq!(proposal_infos.len(), proposal_msgs.len());

	get_proposal_user_vote_store().set(
		&(new_proposal_id, proposer),
		&(&CourtUserVoteInfoJsonable {
			active_votes: user_stats.staked_votes.into(),
			vote: CourtUserVoteStatus::Approve
		}).try_into()?
	)?;
	get_user_active_proposal_id_set().add(&(proposer, new_proposal_id))?;
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
					.add_attribute("vote", "approve")
			)
	)
}
