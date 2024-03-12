use cosmwasm_std::{CosmosMsg, Event, MessageInfo, Response, StdError};
use crownfi_cw_common::{data_types::canonical_addr::SeiCanonicalAddr, env::ClonableEnvInfoMut, extentions::timestamp::TimestampExtentions};

use sei_cosmwasm::{SeiQueryWrapper, SeiMsg};

use crate::{error::CourtContractError, proposed_msg::ProposedCourtMsg, state::{app::{get_transaction_proposal_info_vec_mut, get_transaction_proposal_messages_vec, CourtAppConfig, TransactionProposalExecutionStatus, TransactionProposalStatus}, user::{get_all_user_votes, get_user_vote_info_store_mut}}, workarounds::{mint_workaround, total_supply_workaround}};

use super::{enforce_unfunded, shares::votes_denom};

pub fn process_deactivate_votes(
	env_info: ClonableEnvInfoMut<SeiQueryWrapper>,
	msg_info: MessageInfo,
	user: Option<SeiCanonicalAddr>,
	limit: Option<u32>
) -> Result<Response<SeiMsg>, CourtContractError> {
	enforce_unfunded(&msg_info)?;
	let app_config = CourtAppConfig::load_non_empty(*env_info.storage.borrow())?;
	let token_supply = total_supply_workaround(*env_info.storage.borrow(), &votes_denom(&env_info.env));
	let user = user.unwrap_or(SeiCanonicalAddr::from_addr_using_api(&msg_info.sender, *env_info.api)?);


	let proposals = get_transaction_proposal_info_vec_mut(env_info.storage.clone())?;
	let user_vote_info_store = get_user_vote_info_store_mut(env_info.storage.clone())?;

	// this .take() is a little fugly, though the resolution of the following issue would help clean up the code:
	// https://github.com/rust-lang/rust/issues/63065
	let binding = env_info.storage.borrow();
 	let user_vote_info_keys_iter = get_all_user_votes(
		*binding,
		user.clone()
	)?.take(limit.map(|limit| {limit as usize}).unwrap_or(usize::MAX));

	for (proposal_id, _) in user_vote_info_keys_iter {
		let proposal = proposals.get(proposal_id)?.ok_or(
			StdError::not_found(format!("Proposal {} which the user voted for doesn't exist?!", proposal_id))
		)?;
		if !proposal.status(env_info.env.block.time.millis(), token_supply.u128(), &app_config).is_finalized() {
			return Err(CourtContractError::ProposalNotFinalized(proposal_id));
		}
		user_vote_info_store.remove(&(user.clone(), proposal_id));
	}
	Ok(
		Response::new()
	)
}



pub fn process_execute_proposal(
	env_info: ClonableEnvInfoMut<SeiQueryWrapper>,
	msg_info: MessageInfo,
	proposal_id: u32
) -> Result<Response<SeiMsg>, CourtContractError> {
	enforce_unfunded(&msg_info)?;
	let app_config = CourtAppConfig::load_non_empty(*env_info.storage.borrow())?;
	let token_supply = total_supply_workaround(*env_info.storage.borrow(), &votes_denom(&env_info.env));

	let proposals = get_transaction_proposal_info_vec_mut(env_info.storage.clone())?;
	let mut proposal = proposals.get(proposal_id)?.ok_or(
		StdError::not_found(format!("Proposal {} does not exist", proposal_id))
	)?;
	let proposal_status = proposal.status(env_info.env.block.time.millis(), token_supply.u128(), &app_config);
	if proposal_status == TransactionProposalStatus::ExecutionExpired {
		proposal.set_execution_status(TransactionProposalExecutionStatus::Cancelled);
		return Ok(
			Response::new()
				.add_event(
					Event::new("proposal_cancelled")
						.add_attribute("proposal_id", proposal_id.to_string())
				)
		)
	}
	proposal_status.enforce_status(TransactionProposalStatus::Passed)?;
	proposal.set_execution_status(TransactionProposalExecutionStatus::Executed);
	proposals.set(proposal_id, &proposal)?;

	let votes_denom = votes_denom(&env_info.env);
	Ok(
		Response::new()
			.add_event(
				Event::new("proposal_executed")
					.add_attribute("proposal_id", proposal_id.to_string())
			)
			.add_messages(
				get_transaction_proposal_messages_vec(*env_info.storage.borrow())?
					.get(proposal_id)?
					.unwrap_or_default()
					.into_iter()
					.map(|p_msg| {
						match p_msg {
								ProposedCourtMsg::TokenfactoryMint {
									tokens
								} if tokens.denom == votes_denom => {
									if
										total_supply_workaround(
											*env_info.storage.borrow(), &votes_denom
										).u128().saturating_mul(10000) == u128::MAX
									{
										// Allow us to "unsafely" do permyriad calculations without fear of overflow
										return Err(CourtContractError::TooManyVotesToMint);
									}
									// HACK: https://github.com/sei-protocol/sei-wasmd/issues/38
									Ok(
										mint_workaround(*env_info.storage.borrow_mut(), &tokens.denom, tokens.amount)
											.map(|msg| { CosmosMsg::from(msg) })?
									)
								},
								_ => {
									Ok(
										p_msg.into_cosm_msg(*env_info.api)?
									)
								}
							}
						
					})
					.collect::<Result<Vec<_>, CourtContractError>>()?
			)
	)
}
