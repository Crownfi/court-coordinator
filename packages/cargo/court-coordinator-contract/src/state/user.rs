use std::{cell::RefCell, rc::Rc};

use cosmwasm_schema::schemars::{self, JsonSchema};
use cosmwasm_std::{Uint128, Storage, StdError, Addr, StdResult};
use crownfi_cw_common::storage::{map::{StoredMap, StoredMapIter}, MaybeMutableStorage};
use serde::{Deserialize, Serialize};


const USER_VOTES_NAMESPACE: &str = "user_stats";
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct CourtUserStats {
	pub staked_votes: Uint128
}
pub fn get_user_stats_store<'a>(
	storage: &'a dyn Storage
) -> Result<StoredMap<'a, Addr, CourtUserStats>, StdError> {
	return Ok(StoredMap::new(USER_VOTES_NAMESPACE.as_ref(), MaybeMutableStorage::Immutable(storage)));
}
pub fn get_user_stats_store_mut<'a>(
	storage: Rc<RefCell<&'a mut dyn Storage>>
) -> Result<StoredMap<'a, Addr, CourtUserStats>, StdError> {
	return Ok(StoredMap::new(USER_VOTES_NAMESPACE.as_ref(), MaybeMutableStorage::MutableShared(storage)));
}

const USER_PROPOSALS_NAMESPACE: &str = "user_votes";
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default, PartialEq, Eq)]
pub struct CourtUserVoteInfo {
	pub active_votes: Uint128,
	pub voted_for: bool
}
pub fn get_user_vote_info_store<'a>(
	storage: &'a dyn Storage
) -> Result<StoredMap<'a, (Addr, u32), CourtUserVoteInfo>, StdError> {
	return Ok(StoredMap::new(USER_PROPOSALS_NAMESPACE.as_ref(), MaybeMutableStorage::Immutable(storage)));
}
pub fn get_user_vote_info_store_mut<'a>(
	storage: Rc<RefCell<&'a mut dyn Storage>>
) -> Result<StoredMap<'a, (Addr, u32), CourtUserVoteInfo>, StdError> {
	return Ok(StoredMap::new(USER_PROPOSALS_NAMESPACE.as_ref(), MaybeMutableStorage::MutableShared(storage)));
}
pub fn get_all_user_vote_info_iter<'a>(
	storage: &'a dyn Storage,
	user: Addr
) -> StdResult<StoredMapIter::<'a, u32, CourtUserVoteInfo>> {
	StoredMapIter::<'a, u32, CourtUserVoteInfo>::new(
		MaybeMutableStorage::Immutable(storage),
		USER_PROPOSALS_NAMESPACE.as_bytes(),
		user,
		None,
		None
	)
}
pub fn get_all_user_vote_info_iter_mut<'a>(
	storage: Rc<RefCell<&'a mut dyn Storage>>,
	user: Addr
) -> StdResult<StoredMapIter::<'a, u32, CourtUserVoteInfo>> {
	StoredMapIter::<'a, u32, CourtUserVoteInfo>::new(
		MaybeMutableStorage::MutableShared(storage),
		USER_PROPOSALS_NAMESPACE.as_bytes(),
		user,
		None,
		None
	)
}

pub fn get_all_user_vote_info_keys_iter<'a>(
	storage: &'a dyn Storage,
	user: Addr
) -> StdResult<impl Iterator<Item = u32> + 'a> {
	// Using the unit type prevents the value from being deserialized
	Ok(
		StoredMapIter::<'a, u32, ()>::new(
			MaybeMutableStorage::Immutable(storage),
			USER_PROPOSALS_NAMESPACE.as_bytes(),
			user,
			None,
			None
		)?.map(|(index, _)| {index})
	)
}

pub fn get_all_user_vote_info_keys_iter_mut<'a>(
	storage: Rc<RefCell<&'a mut dyn Storage>>,
	user: Addr
) -> StdResult<impl Iterator<Item = u32> + 'a> {
	// Using the unit type prevents the value from being deserialized
	Ok(
		StoredMapIter::<'a, u32, ()>::new(
			MaybeMutableStorage::MutableShared(storage),
			USER_PROPOSALS_NAMESPACE.as_bytes(),
			user,
			None,
			None
		)?.map(|(index, _)| {index})
	)
}


/*
pub fn get_user_proposals_queue<'a>(
	addr: &Addr,
	storage: &'a dyn Storage
) -> Result<StoredVecDeque<'a, u32>, StdError> {
	// Kinda shot myself in my foot here by not allowing pathed namespaces, but hey, this saves on gas and it's not
	// like our program exists for more than a couple of seconds anyway.
	let full_namespace: &'static str = {
		let mut string = String::with_capacity(USER_PROPOSALS_NAMESPACE.len() + addr.as_str().len());
		string.push_str(USER_PROPOSALS_NAMESPACE);
		string.push_str(addr.as_str());
		// LEAKING 👏 MEMORY 👏 IS 👏 SAFE!!
		string.leak()
	};
	return Ok(StoredVecDeque::new(full_namespace.as_bytes(), MaybeMutableStorage::Immutable(storage)))
}
*/
