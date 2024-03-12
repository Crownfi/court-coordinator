use cosmwasm_std::{MessageInfo, Addr, Response, Uint128, StdError};
use crownfi_cw_common::{data_types::canonical_addr::SeiCanonicalAddr, env::ClonableEnvInfoMut, extentions::timestamp::TimestampExtentions, storage::item::StoredItem};
use sei_cosmwasm::SeiMsg;

use crate::{error::CourtContractError, state::app::{get_transaction_proposal_info_vec, CourtAppConfig, CourtAppStats}, workarounds::{mint_to_workaround, total_supply_workaround}};

use super::{shares::votes_denom, enforce_unfunded};



pub struct AdminMsgExecutor<'exec, Q: cosmwasm_std::CustomQuery> {
	env_info: ClonableEnvInfoMut<'exec, Q>,
	app_config: CourtAppConfig
}
impl<'exec, Q: cosmwasm_std::CustomQuery> AdminMsgExecutor<'exec, Q> {
	pub fn new(env_info: ClonableEnvInfoMut<'exec, Q>, msg_info: &MessageInfo) -> Result<Self, CourtContractError> {
		let msg_sender = SeiCanonicalAddr::from_addr_using_api(&msg_info.sender, *env_info.api)?;
		let app_config = CourtAppConfig::load_non_empty(*env_info.storage.borrow())?;
		if msg_sender != app_config.admin {
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
		max_proposal_expiry_time_seconds: Option<u32>,
		execution_expiry_time_seconds: Option<u32>,
		admin: Option<Addr>
	) -> Result<Response<SeiMsg>, CourtContractError> {
		enforce_unfunded(msg_info)?;
		let app_stats = CourtAppStats::load(*self.env_info.storage.borrow())?.unwrap_or_default();
		let latest_proposal = get_transaction_proposal_info_vec(
			*self.env_info.storage.borrow()
		)?.get(app_stats.latest_proposal_expiry_id)?.ok_or(StdError::not_found("latest proposal doesn't exist?!"))?;
		
		let current_timestamp_ms = self.env_info.env.block.time.millis();
		let token_supply = total_supply_workaround(*self.env_info.storage.borrow(), &votes_denom(&self.env_info.env));
		if !latest_proposal.status(current_timestamp_ms, token_supply.u128(), &self.app_config).is_finalized() {
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
		if let Some(max_proposal_expiry_time_seconds) = max_proposal_expiry_time_seconds {
			self.app_config.max_proposal_expiry_time_seconds = max_proposal_expiry_time_seconds;
		}
		if let Some(execution_expiry_time_seconds) = execution_expiry_time_seconds {
			self.app_config.execution_expiry_time_seconds = execution_expiry_time_seconds;
		}
		if let Some(admin) = admin  {
			// A better check would be "are there any approved proposals which will restore proposals?"
			// ...but this works for now
			if !self.app_config.allow_new_proposals() && self.env_info.env.contract.address == admin {
				return Err(CourtContractError::WouldLockupContract);
			}
			self.app_config.admin = SeiCanonicalAddr::from_addr_using_api(&admin, *self.env_info.api)?;
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
		// A better check would be "are there any approved proposals which will restore proposals?"
		// ...but this works for now
		if msg_info.sender == self.env_info.env.contract.address && !allow {
			return Err(CourtContractError::WouldLockupContract);
		}
		self.app_config.set_allow_new_proposals(allow);
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
