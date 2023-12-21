use cosmwasm_std::{MessageInfo, Addr, Response, Uint128, BankMsg, StdError, BankQuery};
use crownfi_cw_common::{env::ClonableEnvInfoMut, storage::item::StoredItem, extentions::timestamp::TimestampExtentions};
use sei_cosmwasm::SeiMsg;

use crate::{state::app::{CourtAppConfig, CourtAppStats, get_transaction_proposal_stored_vec}, error::CourtContractError, workarounds::mint_to_workaround};

use super::{shares::{votes_coin, votes_denom}, enforce_unfunded};



pub struct AdminMsgExecutor<'exec, Q: cosmwasm_std::CustomQuery> {
	env_info: ClonableEnvInfoMut<'exec, Q>,
	app_config: CourtAppConfig
}
impl<'exec, Q: cosmwasm_std::CustomQuery> AdminMsgExecutor<'exec, Q> {
	pub fn new(env_info: ClonableEnvInfoMut<'exec, Q>, msg_info: &MessageInfo) -> Result<Self, CourtContractError> {
		let app_config = CourtAppConfig::load_non_empty(*env_info.storage.borrow())?;
		if msg_info.sender != app_config.admin {
			return Err(CourtContractError::Unauthorized("Transaction sender is not an admin".into()));
		}
		Ok(
			Self {
				env_info: env_info.clone(),
				app_config
			}
		)
	}
	pub fn process_change_config(
		&mut self,
		msg_info: &MessageInfo,
		minimum_vote_proposal_percent: Option<u8>,
		minimum_vote_turnout_percent: Option<u8>,
		minimum_vote_pass_percent: Option<u8>,
		max_expiry_time_seconds: Option<u32>,
		admin: Option<Addr>
	) -> Result<Response<SeiMsg>, CourtContractError> {
		enforce_unfunded(msg_info)?;
		let app_stats = CourtAppStats::load(*self.env_info.storage.borrow())?.unwrap_or_default();
		let latest_proposal = get_transaction_proposal_stored_vec(
			*self.env_info.storage.borrow()
		)?.get(app_stats.latest_proposal_expiry_id)?.ok_or(StdError::not_found("latest proposal doesn't exist?!"))?;
		
		let current_timestamp_ms = self.env_info.env.block.time.millis();
		let token_supply = self.env_info.querier.query(
			&BankQuery::Supply { denom: votes_denom(&self.env_info.env) }.into()
		)?;
		if !latest_proposal.status(current_timestamp_ms, &token_supply, &self.app_config).is_finalized() {
			return Err(CourtContractError::ProposalNotFinalized(app_stats.latest_proposal_expiry_id));
		}
		if let Some(minimum_vote_proposal_percent) = minimum_vote_proposal_percent {
			self.app_config.minimum_vote_proposal_percent = minimum_vote_proposal_percent;
		}
		if let Some(minimum_vote_turnout_percent) = minimum_vote_turnout_percent {
			self.app_config.minimum_vote_turnout_percent = minimum_vote_turnout_percent;
		}
		if let Some(minimum_vote_pass_percent) = minimum_vote_pass_percent {
			self.app_config.minimum_vote_pass_percent = minimum_vote_pass_percent;
		}
		if let Some(max_expiry_time_seconds) = max_expiry_time_seconds {
			self.app_config.max_expiry_time_seconds = max_expiry_time_seconds;
		}
		if let Some(admin) = admin  {
			self.app_config.admin = admin;
		}
		self.app_config.save(*self.env_info.storage.borrow_mut())?;
		Ok(Response::new())
	}
	pub fn process_allow_new_proposals(
		&mut self,
		msg_info: &MessageInfo,
		allow: bool
	) -> Result<Response<SeiMsg>, CourtContractError> {
		enforce_unfunded(msg_info)?;
		self.app_config.allow_new_proposals = allow;
		self.app_config.save(*self.env_info.storage.borrow_mut())?;
		Ok(Response::new())
	}
	pub fn process_mint_shares(
		&self,
		msg_info: &MessageInfo,
		receiver: Addr,
		amount: Uint128
	) -> Result<Response<SeiMsg>, CourtContractError> {
		enforce_unfunded(msg_info)?;
		Ok(
			mint_to_workaround(
				Response::new(),
				*self.env_info.storage.borrow_mut(),
				&votes_denom(&self.env_info.env),
				&receiver,
				amount.u128()
			)?
		)
	}
}
