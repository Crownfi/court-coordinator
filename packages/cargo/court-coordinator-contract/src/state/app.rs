use std::{cell::RefCell, rc::Rc};

use cosmwasm_schema::schemars::{self, JsonSchema};
use cosmwasm_std::{CosmosMsg, Storage, StdError, Uint128, Timestamp, Addr};
use crownfi_cw_common::storage::{item::StoredItem, vec::StoredVec, MaybeMutableStorage};
use sei_cosmwasm::SeiMsg;
use serde::{Serialize, Deserialize};

use crate::error::CourtContractError;

pub const CONFIG_NAMESPACE: &str = "app_config";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CourtAppConfig {
	pub allow_new_proposals: bool,
	pub minimum_vote_proposal_percent: u8,
	pub minimum_vote_turnout_percent: u8,
	pub minimum_vote_pass_percent: u8,
	pub max_expiry_time_seconds: u32,
	pub last_config_change_timestamp_ms: u64,
	pub admin: Addr

}
impl StoredItem for CourtAppConfig {
    fn namespace() -> &'static [u8] {
        CONFIG_NAMESPACE.as_bytes()
    }
}
impl CourtAppConfig {
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
}

pub const STATS_NAMESPACE: &str = "app_stats";
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct CourtAppStats {
	pub latest_proposal_expiry_id: u32,
	pub latest_proposal_expiry_timestamp_ms: u64,

}
impl StoredItem for CourtAppStats {
    fn namespace() -> &'static [u8] {
        STATS_NAMESPACE.as_bytes()
    }
}


const PROPOSAL_NAMESPACE: &str = "app_proposals";

/// Transaction proposal status, this is derived from the actual proposal struct rather than as a property.
/// 
/// The way this is derived is documented below. This is true because no votes can be changed after expiry.
/// ```
/// let proposal_status = if executed {
/// 	TransactionProposalStatus::Executed
/// } else if expiry < last_config_change_time {
/// 	TransactionProposalStatus::Rejected
/// } else if current_time < expiry {
/// 	TransactionProposalStatus::Pending
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
pub enum TransactionProposalStatus {
	/// Votes are still being collected
	#[default]
	Pending,
	/// The proposed transaction will not be executed
	Rejected,
	/// The proposed transaction will be executed, but hasn't yet
	Passed,
	/// The propsoal passed and the transaction has executed
	Executed
}
impl TransactionProposalStatus {
	/// Checks if the proposal is `Executed` or `Rejected`. 
	pub fn is_finalized(&self) -> bool {
		match self {
			TransactionProposalStatus::Rejected |
			TransactionProposalStatus::Executed => true,
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
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct TransactionProposalInfo {
	pub proposer: Addr,
	pub messages: Vec<CosmosMsg<SeiMsg>>,
	pub votes_for: Uint128,
	pub votes_against: Uint128,
	pub executed: bool,
	// Serializing numbers as strings is cringe. It's a unix timestamp, it'll fit in 2**53.
	pub expiry_timestamp_ms: u64
}
impl TransactionProposalInfo {
	pub fn new(
		proposer: Addr,
		proposer_votes: Uint128,
		messages: Vec<CosmosMsg<SeiMsg>>,
		expiry_timestamp_ms: u64
	) -> Self {
		Self {
			proposer,
			messages,
			votes_for: proposer_votes,
			votes_against: Uint128::zero(),
			executed: false,
			expiry_timestamp_ms
		}
	}
	pub fn status(&self, current_timestamp_ms: u64, token_supply: &Uint128, app_config: &CourtAppConfig) -> TransactionProposalStatus {
		if self.executed {
			TransactionProposalStatus::Executed
		} else if self.expiry_timestamp_ms < app_config.last_config_change_timestamp_ms {
			// The pass threshold may have changed, but that's not relevant to when the transaction was created.
			TransactionProposalStatus::Rejected
		} else if current_timestamp_ms < self.expiry_timestamp_ms {
			TransactionProposalStatus::Pending
		} else if current_timestamp_ms >= self.expiry_timestamp_ms && (
			u8::try_from(
				((self.votes_for + self.votes_against) * Uint128::from(100u8) / token_supply).u128()
			).unwrap() < app_config.minimum_vote_turnout_percent ||
			u8::try_from(
				(self.votes_for * Uint128::from(100u8) / (self.votes_for + self.votes_against)).u128()
			).unwrap() < app_config.minimum_vote_pass_percent
		) {
			TransactionProposalStatus::Rejected
		} else {
			TransactionProposalStatus::Passed
		}
	}
}

pub fn get_transaction_proposal_stored_vec<'a>(storage: &'a dyn Storage) -> Result<StoredVec<'a, TransactionProposalInfo>, StdError> {
	return Ok(StoredVec::new(PROPOSAL_NAMESPACE.as_ref(), MaybeMutableStorage::Immutable(storage)));
}
pub fn get_transaction_proposal_stored_vec_mut<'a>(storage: Rc<RefCell<&'a mut dyn Storage>>) -> Result<StoredVec<'a, TransactionProposalInfo>, StdError> {
	return Ok(StoredVec::new(PROPOSAL_NAMESPACE.as_ref(), MaybeMutableStorage::MutableShared(storage)));
}
