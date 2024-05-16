use bytemuck::{Pod, Zeroable};
use cosmwasm_schema::schemars::{self, JsonSchema};
use cosmwasm_std::{StdError, Uint128};
use crownfi_cw_common::{data_types::canonical_addr::SeiCanonicalAddr, impl_serializable_as_ref, storage::{item::StoredItem, map::{StoredMap, StoredMapIter}, SerializableItem}};
use serde::{Deserialize, Serialize};


const USER_VOTES_NAMESPACE: &str = "user_stats";
#[derive(Debug, Clone, Copy, Default, Zeroable, Pod)]
#[repr(C)]
pub struct CourtUserStats {
	pub staked_votes: u128
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
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
const USER_PROPOSALS_NAMESPACE: &str = "user_votes";
#[derive(Debug, Clone, Copy, Default, Zeroable, Pod)]
#[repr(C)]
pub struct CourtUserVoteInfo {
	pub active_votes: u128,
	voted_for: u8,
	_unused: [u8; 7]
}
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct CourtUserVoteInfoJsonable {
	pub active_votes: Uint128,
	pub voted_for: bool
}

impl CourtUserVoteInfo {
	pub fn voted_for(&self) -> bool {
		self.voted_for != 0
	}
	pub fn set_voted_for(&mut self, value: bool) {
		self.voted_for = value as u8;
	}
}
impl_serializable_as_ref!(CourtUserVoteInfo);
impl TryFrom<&CourtUserVoteInfoJsonable> for CourtUserVoteInfo {
	type Error = StdError;

	fn try_from(value: &CourtUserVoteInfoJsonable) -> Result<Self, Self::Error> {
		Ok(
			CourtUserVoteInfo {
				active_votes: value.active_votes.into(),
				voted_for: value.voted_for as u8,
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
				voted_for: value.voted_for()
			}
		)
	}
}

pub fn get_user_vote_info_store() -> StoredMap<(SeiCanonicalAddr, u32), CourtUserVoteInfo> {
	StoredMap::new(USER_PROPOSALS_NAMESPACE.as_ref())
}
pub fn get_all_user_votes(
	user: SeiCanonicalAddr
) -> Result<StoredMapIter<u32, CourtUserVoteInfo>, StdError> {
	StoredMapIter::new(
		USER_PROPOSALS_NAMESPACE.as_ref(),
		user,
		None,
		None
	)
}
