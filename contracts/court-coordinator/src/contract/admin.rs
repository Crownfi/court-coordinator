use cosmwasm_std::{MessageInfo, Addr, Response, Uint128};
use crownfi_cw_common::{data_types::canonical_addr::SeiCanonicalAddr, env::MinimalEnvInfo, extentions::timestamp::TimestampExtentions, storage::{item::StoredItem, OZeroCopy}};
use cw_utils::nonpayable;
use sei_cosmwasm::SeiMsg;

use crate::{error::CourtContractError, state::{app::CourtAppConfig, user::get_user_active_proposal_id_set}, workarounds::mint_to_workaround};

use super::shares::votes_denom;



pub struct AdminMsgExecutor<'exec, Q: cosmwasm_std::CustomQuery> {
	env_info: MinimalEnvInfo<'exec, Q>,
	app_config: OZeroCopy<CourtAppConfig>
}
impl<'exec, Q: cosmwasm_std::CustomQuery> AdminMsgExecutor<'exec, Q> {
	pub fn new(env_info: MinimalEnvInfo<'exec, Q>, msg_info: &MessageInfo) -> Result<Self, CourtContractError> {
		let msg_sender = SeiCanonicalAddr::try_from(&msg_info.sender)?;
		let app_config = CourtAppConfig::load_non_empty()?;
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
	) -> Result<Response<SeiMsg>, CourtContractError> {
		nonpayable(msg_info)?;
		if get_user_active_proposal_id_set().iter()?.next().is_some() {
			// Easiest way to check if there are any active proposals
			return Err(CourtContractError::VotesActive);
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
		self.app_config.last_config_change_timestamp_ms = self.env_info.env.block.time.millis();
		self.app_config.save()?;
		Ok(Response::new())
	}
	pub fn process_change_admin(
		&mut self,
		msg_info: &MessageInfo,
		admin: Addr
	) -> Result<Response<SeiMsg>, CourtContractError> {
		nonpayable(msg_info)?;
		// A better check would be "are there any approved proposals which will restore proposals?"
		// ...but this works for now
		if !self.app_config.allow_new_proposals() && self.env_info.env.contract.address == admin {
			return Err(CourtContractError::WouldLockupContract);
		}
		self.app_config.admin = admin.try_into()?;
		self.app_config.save()?;
		Ok(Response::new())
	}
	pub fn process_allow_new_proposals(
		&mut self,
		msg_info: &MessageInfo,
		allow: bool
	) -> Result<Response<SeiMsg>, CourtContractError> {
		nonpayable(msg_info)?;
		// A better check would be "are there any approved proposals which will restore proposals?"
		// ...but this works for now
		if msg_info.sender == self.env_info.env.contract.address && !allow {
			return Err(CourtContractError::WouldLockupContract);
		}
		self.app_config.set_allow_new_proposals(allow);
		self.app_config.save()?;
		Ok(Response::new())
	}
	pub fn process_mint_shares(
		&self,
		msg_info: &MessageInfo,
		receiver: Addr,
		amount: Uint128
	) -> Result<Response<SeiMsg>, CourtContractError> {
		nonpayable(msg_info)?;
		Ok(
			mint_to_workaround(
				Response::new(),
				&votes_denom(&self.env_info.env),
				&receiver,
				amount.u128()
			)?
		)
	}
}
