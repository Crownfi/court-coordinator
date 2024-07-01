use core::fmt;

use bytemuck::{Pod, Zeroable};
use cosmwasm_schema::schemars::{self, JsonSchema};
use cosmwasm_std::{StdError, Uint128};
use crownfi_cw_common::{data_types::canonical_addr::SeiCanonicalAddr, impl_serializable_as_ref, storage::{item::StoredItem, map::{StoredMap, StoredMapIter, StoredMapKeyIter}, set::StoredSet, SerializableItem}};
use serde::{Deserialize, Serialize};


const USER_VOTES_NAMESPACE: &str = "user_stats";
#[derive(Debug, Clone, Copy, Default, Zeroable, Pod, PartialEq, Eq)]
#[repr(C)]
pub struct CourtUserStats {
	pub staked_votes: u128
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default, PartialEq, Eq)]
pub struct CourtUserStatsJsonable {
	pub staked_votes: Uint128
}
impl_serializable_as_ref!(CourtUserStats);
impl StoredItem for CourtUserStats {
	fn namespace() -> &'static [u8] {
		USER_VOTES_NAMESPACE.as_bytes()
	}
}
impl TryFrom<&CourtUserStatsJsonable> for CourtUserStats {
	type Error = StdError;
	fn try_from(value: &CourtUserStatsJsonable) -> Result<Self, Self::Error> {
		Ok(
			CourtUserStats {
				staked_votes: value.staked_votes.into()
			}
		)
	}
}
impl TryFrom<&CourtUserStats> for CourtUserStatsJsonable {
	type Error = StdError;
	fn try_from(value: &CourtUserStats) -> Result<Self, Self::Error> {
		Ok(
			CourtUserStatsJsonable {
				staked_votes: value.staked_votes.into()
			}
		)
	}
}
pub fn get_user_stats_store() -> StoredMap<SeiCanonicalAddr, CourtUserStats> {
	StoredMap::new(USER_VOTES_NAMESPACE.as_ref())
}

#[derive(Debug, Clone, Copy, Default, Zeroable, Pod, PartialEq, Eq)]
#[repr(C)]
pub struct CourtUserVoteInfo {
	pub active_votes: u128,
	vote: u8,
	_unused: [u8; 15]
}
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema, PartialEq, Eq)]
pub struct CourtUserVoteInfoJsonable {
	pub active_votes: Uint128,
	pub vote: CourtUserVoteStatus
}

#[derive(Debug, Clone, Copy, Serialize, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[repr(u8)]
pub enum CourtUserVoteStatus {
	#[default]
	Abstain = 0,
	Approve = 1,
	Oppose = 2,
}
// SAFTY: is a u8 and has a varient set to "0"
unsafe impl Zeroable for CourtUserVoteStatus {}
impl From<u8> for CourtUserVoteStatus {
	fn from(value: u8) -> Self{
		match value {
			0 => CourtUserVoteStatus::Abstain,
			1 => CourtUserVoteStatus::Approve,
			2 => CourtUserVoteStatus::Oppose,
			_ => CourtUserVoteStatus::Abstain
		}
	}
}
impl From<CourtUserVoteStatus> for u8 {
	fn from(value: CourtUserVoteStatus) -> Self {
		value as u8
	}
}
impl fmt::Display for CourtUserVoteStatus {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			CourtUserVoteStatus::Oppose => f.write_str("oppose"),
			CourtUserVoteStatus::Approve => f.write_str("approve"),
			CourtUserVoteStatus::Abstain => f.write_str("abstain"),
		}
	}
}

impl CourtUserVoteInfo {
	pub fn vote(&self) -> CourtUserVoteStatus {
		self.vote.into()
	}
	pub fn set_vote(&mut self, value: CourtUserVoteStatus) {
		self.vote = value.into()
	}
}
impl_serializable_as_ref!(CourtUserVoteInfo);
impl TryFrom<&CourtUserVoteInfoJsonable> for CourtUserVoteInfo {
	type Error = StdError;

	fn try_from(value: &CourtUserVoteInfoJsonable) -> Result<Self, Self::Error> {
		Ok(
			CourtUserVoteInfo {
				active_votes: value.active_votes.into(),
				vote: value.vote.into(),
				.. Zeroable::zeroed()
			}
		)
	}
}
impl TryFrom<&CourtUserVoteInfo> for CourtUserVoteInfoJsonable {
	type Error = StdError;

	fn try_from(value: &CourtUserVoteInfo) -> Result<Self, Self::Error> {
		Ok(
			CourtUserVoteInfoJsonable {
				active_votes: value.active_votes.into(),
				vote: value.vote.into()
			}
		)
	}
}

const USER_ACTIVE_PROPOSAL_NAMESPACE: &str = "user_prop_a";
pub fn get_user_active_proposal_id_set() -> StoredSet<(SeiCanonicalAddr, u32)> {
	StoredSet::new(USER_ACTIVE_PROPOSAL_NAMESPACE.as_ref())
}
pub fn get_all_user_active_proposal_ids(
	user: SeiCanonicalAddr
) -> Result<StoredMapKeyIter<u32>, StdError> {
	StoredMapKeyIter::new(
		USER_ACTIVE_PROPOSAL_NAMESPACE.as_ref(),
		user,
		None,
		None
	)
}

const USER_PROPOSAL_VOTES_NAMESPACE: &str = "user_prop_v";
pub fn get_proposal_user_vote_store() -> StoredMap<(u32, SeiCanonicalAddr), CourtUserVoteInfo> {
	StoredMap::new(USER_PROPOSAL_VOTES_NAMESPACE.as_ref())
}

/// Gets all the users who voted for a proposal
///
/// `start` is inclusive while `end` is exclusive
pub fn get_all_proposal_user_votes(
	proposal_id: u32,
	start: Option<SeiCanonicalAddr>,
	end: Option<SeiCanonicalAddr>
) -> Result<StoredMapIter<SeiCanonicalAddr, CourtUserVoteInfo>, StdError> {
	StoredMapIter::new(
		USER_PROPOSAL_VOTES_NAMESPACE.as_ref(),
		proposal_id,
		start,
		end
	)
}
