use std::{cell::RefCell, rc::Rc};

use bytemuck::{Pod, Zeroable};
use cosmwasm_schema::schemars::{self, JsonSchema};
use cosmwasm_std::{Addr, Api, StdError, Storage, Uint128};
use crownfi_cw_common::{data_types::canonical_addr::SeiCanonicalAddr, impl_serializable_as_ref, storage::{item::StoredItem, SerializableItem, vec::StoredVec, MaybeMutableStorage}};
use serde::{Serialize, Deserialize};

use crate::{error::CourtContractError, proposed_msg::ProposedCourtMsg};

pub const CONFIG_NAMESPACE: &str = "app_config";

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct CourtAppConfig {
	allow_new_proposals: u8, // bool
	pub minimum_vote_proposal_percent: u8,
	pub minimum_vote_turnout_percent: u8,
	pub minimum_vote_pass_percent: u8,
	pub max_proposal_expiry_time_seconds: u32,
	pub execution_expiry_time_seconds: u32,
	_unused: [u8; 4],
	pub last_config_change_timestamp_ms: u64,
	pub admin: SeiCanonicalAddr
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CourtAppConfigJsonable {
	pub allow_new_proposals: bool,
	pub minimum_vote_proposal_percent: u8,
	pub minimum_vote_turnout_percent: u8,
	pub minimum_vote_pass_percent: u8,
	pub max_proposal_expiry_time_seconds: u32,
	pub execution_expiry_time_seconds: u32,
	pub last_config_change_timestamp_ms: u64,
	pub admin: Addr
}
impl_serializable_as_ref!(CourtAppConfig);
impl StoredItem for CourtAppConfig {
    fn namespace() -> &'static [u8] {
        CONFIG_NAMESPACE.as_bytes()
    }
}
impl CourtAppConfig {
	pub fn allow_new_proposals(&self) -> bool {
		self.allow_new_proposals != 0
	}
	pub fn set_allow_new_proposals(&mut self, value: bool) {
		self.allow_new_proposals = value as u8;
	}
	pub fn load_non_empty(storage: & dyn Storage) -> Result<Self, StdError> where Self: Sized {
		match Self::load(storage)? {
			Some(result) => {
				Ok(result)
			},
			None => {
				Err(StdError::NotFound { kind: "CourtAppConfig".into() })
			}
		}
	}
	pub fn into_jsonable(&self, api: &dyn Api) -> Result<CourtAppConfigJsonable, StdError> {
		Ok(
			CourtAppConfigJsonable {
				allow_new_proposals: self.allow_new_proposals(),
				minimum_vote_proposal_percent: self.minimum_vote_proposal_percent,
				minimum_vote_turnout_percent: self.minimum_vote_turnout_percent,
				minimum_vote_pass_percent: self.minimum_vote_pass_percent,
				max_proposal_expiry_time_seconds: self.max_proposal_expiry_time_seconds,
				execution_expiry_time_seconds: self.execution_expiry_time_seconds,
				last_config_change_timestamp_ms: self.last_config_change_timestamp_ms,
				admin: self.admin.into_addr_using_api(api)?
			}
		)
	}
}
impl CourtAppConfigJsonable {
	pub fn into_storable(&self, api: &dyn Api) -> Result<CourtAppConfig, StdError> {
		Ok(
			CourtAppConfig {
				allow_new_proposals: self.allow_new_proposals as u8,
				minimum_vote_proposal_percent: self.minimum_vote_proposal_percent,
				minimum_vote_turnout_percent: self.minimum_vote_turnout_percent,
				minimum_vote_pass_percent: self.minimum_vote_pass_percent,
				max_proposal_expiry_time_seconds: self.max_proposal_expiry_time_seconds,
				execution_expiry_time_seconds: self.execution_expiry_time_seconds,
				last_config_change_timestamp_ms: self.last_config_change_timestamp_ms,
				admin: SeiCanonicalAddr::from_addr_using_api(&self.admin, api)?,
				.. Zeroable::zeroed()
			}
		)
	}
}

pub const STATS_NAMESPACE: &str = "app_stats";
#[derive(Debug, Clone, Copy, Default, Zeroable, Pod)]
#[repr(C)]
pub struct CourtAppStats {
	pub latest_proposal_expiry_id: u32,
	_unused: [u8; 4],
	pub latest_proposal_expiry_timestamp_ms: u64,

}
impl_serializable_as_ref!(CourtAppStats);
impl StoredItem for CourtAppStats {
    fn namespace() -> &'static [u8] {
        STATS_NAMESPACE.as_bytes()
    }
}


const PROPOSAL_INFO_NAMESPACE: &str = "app_prop_i";
const PROPOSAL_MSG_NAMESPACE: &str = "app_prop_m";

/// Transaction proposal status, this is derived from the actual proposal struct rather than as a property.
/// 
/// The way this is derived is documented below.
/// ```
/// let proposal_status = if transaction_executed_status == TransactionExecutionStatus::Executed {
/// 	TransactionProposalStatus::Executed
/// } else if transaction_executed_status == TransactionExecutionStatus::Expired {
/// 	TransactionProposalStatus::ExecutionExpired
/// } else if expiry < last_config_change_time {
/// 	TransactionProposalStatus::Rejected
/// } else if current_time < expiry {
/// 	if
/// 		((votes_for + votes_against) * 100 / token_supply) >= minimum_vote_turnout_percent ||
/// 		(votes_for * 100 / token_supply) >= minimum_vote_pass_percent
/// 	{
/// 		TransactionProposalStatus::Passed
/// 	} else {
/// 		TransactionProposalStatus::Pending
/// 	}
/// } else if current_time >= expiry && (
/// 	((votes_for + votes_against) * 100 / token_supply) < minimum_vote_turnout_percent ||
/// 	(votes_for * 100 / (votes_for + votes_against)) < minimum_vote_pass_percent
/// ) {
/// 	TransactionProposalStatus::Rejected
/// } else {
/// 	TransactionProposalStatus::Passed
/// }
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, Default, PartialEq, Eq)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum TransactionProposalStatus {
	/// Votes are still being collected
	#[default]
	Pending = 0,
	/// The proposed transaction will not be executed
	Rejected = 1,
	/// The proposed transaction will be executed, but hasn't yet
	Passed = 2,
	/// The proposal passed and the transaction has executed
	Executed = 3,
	/// The proposal passed but couldn't be executed before the expiry time
	ExecutionExpired = 4,
	/// The proposal was cancelled
	Cancelled = 5
}
// SAFTY: TransactionProposalStatus::Pending is explicitly defined as 0
unsafe impl Zeroable for TransactionProposalStatus {}
impl TransactionProposalStatus {
	/// Checks if the proposal is `Executed` or `Rejected`. 
	pub fn is_finalized(&self) -> bool {
		match self {
			TransactionProposalStatus::Rejected |
			TransactionProposalStatus::Executed | 
			TransactionProposalStatus::Cancelled => true,
			_ => false
		}
	}
	pub fn enforce_status(&self, other: Self) -> Result<(), CourtContractError> {
		if *self == other {
			Ok(())
		} else {
			Err(CourtContractError::UnexpectedProposalStatus { expected: other, actual: *self })
		}
	}
}
impl std::fmt::Display for TransactionProposalStatus {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			TransactionProposalStatus::Pending => {
				f.write_str("pending")
			},
			TransactionProposalStatus::Rejected => {
				f.write_str("rejected")
			},
			TransactionProposalStatus::Passed => {
				f.write_str("passed")
			},
			TransactionProposalStatus::Executed => {
				f.write_str("executed")
			},
			TransactionProposalStatus::ExecutionExpired => {
				f.write_str("execution_expired")
			},
			TransactionProposalStatus::Cancelled => {
				f.write_str("cancelled")
			}
		}
	}
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, Default, PartialEq, Eq)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum TransactionProposalExecutionStatus {
	#[default]
	NotExecuted = 0,
	Executed = 1,
	Cancelled = 2
}
// SAFTY: TransactionProposalStatus::Pending is explicitly defined as 0
unsafe impl Zeroable for TransactionProposalExecutionStatus {}
impl TransactionProposalExecutionStatus {
	pub fn as_proposal_status(&self) -> Option<TransactionProposalStatus> {
		match self {
			TransactionProposalExecutionStatus::NotExecuted => None,
			TransactionProposalExecutionStatus::Executed => Some(TransactionProposalStatus::Executed),
			TransactionProposalExecutionStatus::Cancelled => Some(TransactionProposalStatus::Cancelled),
		}
	}
}
impl TryFrom<u8> for TransactionProposalExecutionStatus {
	type Error = u8;
	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
			0 => Ok(Self::NotExecuted),
			1 => Ok(Self::Executed),
			2 => Ok(Self::Cancelled),
			_ => Err(value)
		}
	}
}
impl From<TransactionProposalExecutionStatus> for u8 {
	fn from(value: TransactionProposalExecutionStatus) -> Self {
		value as u8
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Zeroable, Pod)]
#[repr(C)]
pub struct TransactionProposalInfo {
	pub proposer: SeiCanonicalAddr,
	pub votes_for: u128,
	pub votes_against: u128,
	execution_status: u8, // bool
	_unused: [u8; 7],
	pub expiry_timestamp_ms: u64
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TransactionProposalInfoJsonable {
	pub proposer: Addr,
	pub votes_for: Uint128,
	pub votes_against: Uint128,
	pub execution_status: TransactionProposalExecutionStatus,
	// Serializing numbers as strings is cringe. It's a unix timestamp, it'll fit in 2**53.
	pub expiry_timestamp_ms: u64
}

impl TransactionProposalInfo {
	pub fn new(
		proposer: SeiCanonicalAddr,
		proposer_votes: u128,
		expiry_timestamp_ms: u64
	) -> Self {
		Self {
			proposer,
			votes_for: proposer_votes,
			expiry_timestamp_ms,
			.. Zeroable::zeroed()
		}
	}
	pub fn execution_status(&self) -> TransactionProposalExecutionStatus {
		todo!()
	}
	pub fn set_execution_status(&mut self, value: TransactionProposalExecutionStatus) {
		self.execution_status = value as u8
	}
	pub fn status(&self, current_timestamp_ms: u64, token_supply: u128, app_config: &CourtAppConfig) -> TransactionProposalStatus {
		if let Some(status) = self.execution_status().as_proposal_status() {
			status
		} else if self.expiry_timestamp_ms < app_config.last_config_change_timestamp_ms {
			// The pass threshold may have changed, but that's not relevant to when the transaction was created.
			// Note: last_config_change_timestamp_ms cannot be incremented before proposals are fully executed.
			TransactionProposalStatus::Rejected
		} else if current_timestamp_ms < self.expiry_timestamp_ms {
			let total_vote_for_percent_of_supply = u8::try_from(self.votes_for * 100 / token_supply).unwrap();
			let total_turnout_percent = u8::try_from(
				(self.votes_for + self.votes_against) * 100 / token_supply
			).unwrap();
			if
				total_vote_for_percent_of_supply >= app_config.minimum_vote_pass_percent &&
				total_turnout_percent >= app_config.minimum_vote_turnout_percent
			{
				// At this point, this proposal can't be rejected, (unless new votes are minted) so we might as well
				// allow the transaction to be executed early to save everyone time.
				TransactionProposalStatus::Passed
			} else {
				TransactionProposalStatus::Pending
			}
		} else if 
			u8::try_from(
				// OVERFLOW SAFETY:
				// The Mint function doesn't allow a total supply greater than 34028236692093846346337460743176821.
				// Therefore multiplying by up to 10000 will not overflow.
				// Proposals cannot be created by the contract unless the token supply is non-zero.
				// By definition, the total votes for and against cannot exceed the total number existing votes.
				(self.votes_for + self.votes_against) * 100 / token_supply
			).unwrap() < app_config.minimum_vote_turnout_percent ||
			u8::try_from(
				self.votes_for * 100 / (self.votes_for + self.votes_against)
			).unwrap() < app_config.minimum_vote_pass_percent
		{
			TransactionProposalStatus::Rejected
		} else if current_timestamp_ms > self.expiry_timestamp_ms.saturating_add(
			app_config.execution_expiry_time_seconds.saturating_mul(1000).into()
		) {
			TransactionProposalStatus::ExecutionExpired
		} else {
			TransactionProposalStatus::Passed
		}
	}
	pub fn into_jsonable(&self, api: &dyn Api) -> Result<TransactionProposalInfoJsonable, StdError> {
		Ok(
			TransactionProposalInfoJsonable {
				proposer: self.proposer.into_addr_using_api(api)?,
				votes_for: self.votes_for.into(),
				votes_against: self.votes_against.into(),
				execution_status: self.execution_status(),
				expiry_timestamp_ms: self.expiry_timestamp_ms
			}
		)
	}
}
impl_serializable_as_ref!(TransactionProposalInfo);
impl TransactionProposalInfoJsonable {
	pub fn into_storable(&self, api: &dyn Api) -> Result<TransactionProposalInfo, StdError> {
		Ok(
			TransactionProposalInfo {
				proposer: SeiCanonicalAddr::from_addr_using_api(&self.proposer, api)?,
				votes_for: self.votes_for.u128(),
				votes_against: self.votes_against.u128(),
				execution_status: self.execution_status as u8,
				_unused: Zeroable::zeroed(),
				expiry_timestamp_ms: self.expiry_timestamp_ms
			}
		)
	}
}

pub fn get_transaction_proposal_info_vec<'a>(
	storage: &'a dyn Storage
) -> Result<StoredVec<'a, TransactionProposalInfo>, StdError> {
	Ok(StoredVec::new(PROPOSAL_INFO_NAMESPACE.as_ref(), MaybeMutableStorage::Immutable(storage)))
}
pub fn get_transaction_proposal_info_vec_mut<'a>(
	storage: Rc<RefCell<&'a mut dyn Storage>>
) -> Result<StoredVec<'a, TransactionProposalInfo>, StdError> {
	Ok(StoredVec::new(PROPOSAL_INFO_NAMESPACE.as_ref(), MaybeMutableStorage::MutableShared(storage)))
}
pub fn get_transaction_proposal_messages_vec<'a>(
	storage: &'a dyn Storage
) -> Result<StoredVec<'a, Vec<ProposedCourtMsg>>, StdError> {
	Ok(StoredVec::new(PROPOSAL_MSG_NAMESPACE.as_ref(), MaybeMutableStorage::Immutable(storage)))
}
pub fn get_transaction_proposal_messages_vec_mut<'a>(
	storage: Rc<RefCell<&'a mut dyn Storage>>
) -> Result<StoredVec<'a, Vec<ProposedCourtMsg>>, StdError> {
	Ok(StoredVec::new(PROPOSAL_MSG_NAMESPACE.as_ref(), MaybeMutableStorage::MutableShared(storage)))
}
