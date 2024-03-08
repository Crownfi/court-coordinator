use std::{cell::RefCell, rc::Rc};

use bytemuck::{Pod, Zeroable};
use cosmwasm_schema::schemars::{self, JsonSchema};
use cosmwasm_std::{Api, StdError, Storage, Uint128};
use crownfi_cw_common::{data_types::canonical_addr::SeiCanonicalAddr, impl_serializable_as_ref, storage::{item::StoredItem, map::{StoredMap, StoredMapIter}, MaybeMutableStorage, SerializableItem}};
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

impl CourtUserStats {
	pub fn into_jsonable(&self, _api: &dyn Api) -> Result<CourtUserStatsJsonable, StdError> {
		// This function is infallable and doesn't need to use the api, but this function may be moved to a trait
		// in the future, so it may be important to implement in a way that can be easily changed
		Ok(
			CourtUserStatsJsonable {
				staked_votes: self.staked_votes.into()
			}
		)
	}
}
impl_serializable_as_ref!(CourtUserStats);
impl StoredItem for CourtUserStats {
	fn namespace() -> &'static [u8] {
		USER_VOTES_NAMESPACE.as_bytes()
	}
}
impl CourtUserStatsJsonable {
	pub fn into_storable(&self, _api: &dyn Api) -> Result<CourtUserStats, StdError> {
		Ok(
			CourtUserStats {
				staked_votes: self.staked_votes.into()
			}
		)
	}
}

pub fn get_user_stats_store<'a>(
	storage: &'a dyn Storage
) -> Result<StoredMap<'a, SeiCanonicalAddr, CourtUserStats>, StdError> {
	Ok(StoredMap::new(USER_VOTES_NAMESPACE.as_ref(), MaybeMutableStorage::Immutable(storage)))
}
pub fn get_user_stats_store_mut<'a>(
	storage: Rc<RefCell<&'a mut dyn Storage>>
) -> Result<StoredMap<'a, SeiCanonicalAddr, CourtUserStats>, StdError> {
	Ok(StoredMap::new(USER_VOTES_NAMESPACE.as_ref(), MaybeMutableStorage::MutableShared(storage)))
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
	pub fn into_jsonable(&self, _api: &dyn Api) -> Result<CourtUserVoteInfoJsonable, StdError> {
		Ok(
			CourtUserVoteInfoJsonable {
				active_votes: self.active_votes.into(),
				voted_for: self.voted_for()
			}
		)
	}
}
impl_serializable_as_ref!(CourtUserVoteInfo);
impl CourtUserVoteInfoJsonable {
	pub fn into_storable(&self, _api: &dyn Api) -> Result<CourtUserVoteInfo, StdError> {
		Ok(
			CourtUserVoteInfo {
				active_votes: self.active_votes.into(),
				voted_for: self.voted_for as u8,
				.. Zeroable::zeroed()
			}
		)
	}
}

pub fn get_user_vote_info_store<'a>(
	storage: &'a dyn Storage
) -> Result<StoredMap<'a, (SeiCanonicalAddr, u32), CourtUserVoteInfo>, StdError> {
	Ok(StoredMap::new(USER_PROPOSALS_NAMESPACE.as_ref(), MaybeMutableStorage::Immutable(storage)))
}
pub fn get_user_vote_info_store_mut<'a>(
	storage: Rc<RefCell<&'a mut dyn Storage>>
) -> Result<StoredMap<'a, (SeiCanonicalAddr, u32), CourtUserVoteInfo>, StdError> {
	Ok(StoredMap::new(USER_PROPOSALS_NAMESPACE.as_ref(), MaybeMutableStorage::MutableShared(storage)))
}

pub fn get_all_user_votes<'a>(
	storage: &'a dyn Storage,
	user: SeiCanonicalAddr
) -> Result<StoredMapIter<'a, u32, CourtUserVoteInfo>, StdError> {
	StoredMapIter::new(
		MaybeMutableStorage::Immutable(storage),
		USER_PROPOSALS_NAMESPACE.as_ref(),
		user,
		None,
		None
	)
}
