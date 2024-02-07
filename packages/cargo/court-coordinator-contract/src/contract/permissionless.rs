use cosmwasm_std::{Response, Addr, MessageInfo, StdError, Event};
use crownfi_cw_common::{env::ClonableEnvInfoMut, extentions::timestamp::TimestampExtentions};

use sei_cosmwasm::{SeiQueryWrapper, SeiMsg};

use crate::{error::CourtContractError, state::{app::{get_transaction_proposal_stored_vec_mut, CourtAppConfig, TransactionProposalStatus}, user::{get_user_vote_info_store_mut, get_all_user_vote_info_keys_iter_mut}}, workarounds::total_supply_workaround};

use super::{enforce_unfunded, shares::votes_denom};

pub fn process_deactivate_votes(
	env_info: ClonableEnvInfoMut<SeiQueryWrapper>,
	msg_info: MessageInfo,
	user: Option<Addr>,
	limit: Option<u32>
) -> Result<Response<SeiMsg>, CourtContractError> {
	enforce_unfunded(&msg_info)?;
	let app_config = CourtAppConfig::load_non_empty(*env_info.storage.borrow())?;
	let token_supply = total_supply_workaround(*env_info.storage.borrow(), &votes_denom(&env_info.env));
	let user = user.unwrap_or(msg_info.sender.clone());

	let proposals = get_transaction_proposal_stored_vec_mut(env_info.storage.clone())?;
	let user_vote_info_store = get_user_vote_info_store_mut(env_info.storage.clone())?;
	// this .take() is a little fugly, though the resolution of the following issue would help clean up the code:
	// https://github.com/rust-lang/rust/issues/63065
	let user_vote_info_keys_iter = get_all_user_vote_info_keys_iter_mut(
		env_info.storage.clone(),
		user.clone()
	)?.take(limit.map(|limit| {limit as usize}).unwrap_or(usize::MAX));

	for proposal_id in user_vote_info_keys_iter {
		let proposal = proposals.get(proposal_id)?.ok_or(
			StdError::not_found(format!("Proposal {} which the user voted for doesn't exist?!", proposal_id))
		)?;
		if !proposal.status(env_info.env.block.time.millis(), &token_supply, &app_config).is_finalized() {
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

	let proposals = get_transaction_proposal_stored_vec_mut(env_info.storage.clone())?;
	let mut proposal = proposals.get(proposal_id)?.ok_or(
		StdError::not_found(format!("Proposal {} does not exist", proposal_id))
	)?;
	proposal.status(env_info.env.block.time.millis(), &token_supply, &app_config).enforce_status(TransactionProposalStatus::Passed)?;
	proposal.executed = true;
	proposals.set(proposal_id, &proposal)?;
	Ok(
		Response::new()
			.add_event(
				Event::new("proposal_executed")
					.add_attribute("proposal_id", proposal_id.to_string())
			)
			.add_messages(proposal.messages)
	)
}
