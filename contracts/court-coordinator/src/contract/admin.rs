use cosmwasm_std::{MessageInfo, Addr, Response, Uint128, StdError};
use crownfi_cw_common::{data_types::canonical_addr::SeiCanonicalAddr, env::MinimalEnvInfo, extentions::timestamp::TimestampExtentions, storage::{item::StoredItem, OZeroCopy}};
use sei_cosmwasm::SeiMsg;

use crate::{error::CourtContractError, state::app::{get_transaction_proposal_info_vec, CourtAppConfig, CourtAppStats}, workarounds::{mint_to_workaround, total_supply_workaround}};

use super::{shares::votes_denom, enforce_unfunded};



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
		admin: Option<Addr>
	) -> Result<Response<SeiMsg>, CourtContractError> {
		enforce_unfunded(msg_info)?;
		let app_stats = CourtAppStats::load()?.unwrap_or_default();
		let latest_proposal = get_transaction_proposal_info_vec()
			.get(app_stats.latest_proposal_expiry_id)?.ok_or(StdError::not_found("latest proposal doesn't exist?!"))?;
		
		let current_timestamp_ms = self.env_info.env.block.time.millis();
		let token_supply = total_supply_workaround(&votes_denom(&self.env_info.env));
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
			self.app_config.admin = admin.try_into()?;
		}
		self.app_config.save()?;
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
		self.app_config.save()?;
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
				&votes_denom(&self.env_info.env),
				&receiver,
				amount.u128()
			)?
		)
	}
}
