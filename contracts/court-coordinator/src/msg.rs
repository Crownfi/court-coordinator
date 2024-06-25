use cosmwasm_schema::{cw_serde, schemars::{self, JsonSchema}, QueryResponses};
use cosmwasm_std::{Uint128, Addr};
use serde::{Deserialize, Serialize};

use crate::{proposed_msg::ProposedCourtMsgJsonable, state::{app::{CourtAppConfigJsonable, TransactionProposalInfoJsonable, TransactionProposalStatus}, user::{CourtUserStatsJsonable, CourtUserVoteInfoJsonable, CourtUserVoteStatus}}};


#[cw_serde]
pub struct CourtMigrateMsg {}

#[cw_serde]
pub struct CourtInstantiateMsg {
	pub admin: Addr,
	pub shares_mint_amount: Uint128,
	pub shares_mint_receiver: Addr,
	pub minimum_vote_proposal_percent: u8,
	pub minimum_vote_turnout_percent: u8,
	pub minimum_vote_pass_percent: u8,
	pub max_proposal_expiry_time_seconds: u32,
	pub execution_expiry_time_seconds: u32,
	pub vote_share_name: String,
	pub vote_share_symbol: String,
	pub vote_share_description: String
}

#[cw_serde]
pub enum CourtAdminExecuteMsg {
	/// Change config options
	ChangeConfig {
		minimum_vote_proposal_percent: Option<u8>,
		minimum_vote_turnout_percent: Option<u8>,
		minimum_vote_pass_percent: Option<u8>,
		max_proposal_expiry_time_seconds: Option<u32>,
		execution_expiry_time_seconds: Option<u32>
	},
	ChangeAdmin {
		admin: Addr
	},
	AllowNewProposals {
		allowed: bool
	},
	/// Mints new shares, effectively diluting the existing ones
	MintShares {
		receiver: Addr,
		amount: Uint128
	}
}

#[cw_serde]
pub enum CourtExecuteMsg {
	/// Instruction can only be activated by the configured admin
	Admin(CourtAdminExecuteMsg),
	Stake,
	Unstake,
	Vote {
		id: u32,
		vote: CourtUserVoteStatus
	},
	DeactivateVotes {
		user: Option<Addr>,
		limit: Option<u32>
	},
	ProposeTransaction {
		msgs: Vec<ProposedCourtMsgJsonable>,
		expiry_time_seconds: u32
	},
	ExecuteProposal {
		id: u32
	}
}

//#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema)]
//pub enum Cw20ReceiveMsgData {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum CourtQueryMsg {
	#[returns(CourtAppConfigJsonable)]
	Config,
	#[returns(CourtQueryResponseDenom)]
	Denom,
	#[returns(u32)]
	ProposalAmount,
	#[returns(Option<CourtQueryResponseTransactionProposal>)]
	GetProposal {
		id: u32
	},
	#[returns(Vec<CourtQueryResponseTransactionProposal>)]
	GetProposals {
		skip: Option<u32>,
		limit: Option<u32>,
		descending: bool
	},
	#[returns(CourtUserStatsJsonable)]
	UserStats {
		user: Addr
	},
	#[returns(CourtUserVoteInfoJsonable)]
	UserVoteInfo {
		user: Addr,
		proposal_id: u32
	},
	#[returns(Vec<CourtQueryUserWithActiveProposal>)]
	GetUsersWithActiveProposals {
		after: Option<CourtQueryUserWithActiveProposal>,
		limit: Option<u32>,
		descending: bool
	},
	#[returns(Vec<u32>)]
	GetUserActiveProposals {
		user: Addr,
		skip: Option<u32>,
		limit: Option<u32>,
		descending: bool
	},
	#[returns(Vec<CourtQueryResponseUserVote>)]
	GetProposalUserVotes {
		proposal_id: u32,
		after: Option<Addr>,
		limit: Option<u32>,
		descending: bool
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CourtQueryResponseDenom {
	pub votes: String
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CourtQueryResponseTransactionProposal {
	pub proposal_id: u32,
	pub status: TransactionProposalStatus,
	pub info: TransactionProposalInfoJsonable,
	pub messages: Vec<ProposedCourtMsgJsonable>
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CourtQueryResponseUserVote {
	pub user: Addr,
	pub info: CourtUserVoteInfoJsonable
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CourtQueryUserWithActiveProposal {
	pub user: Addr,
	pub proposal_id: u32,
}
