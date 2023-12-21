use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Uint128, Addr, CosmosMsg};
use sei_cosmwasm::SeiMsg;

use crate::state::{app::{CourtAppConfig, TransactionProposalStatus, TransactionProposalInfo}, user::{CourtUserStats, CourtUserVoteInfo}};


#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct InstantiateMsg {
	pub admin: Addr,
	pub shares_mint_amount: Uint128,
	pub shares_mint_receiver: Addr,
	pub minimum_vote_proposal_percent: u8,
	pub minimum_vote_turnout_percent: u8,
	pub minimum_vote_pass_percent: u8,
	pub max_expiry_time_seconds: u32,
}

#[cw_serde]
pub enum AdminExecuteMsg {
	/// Change config options
	ChangeConfig {
		minimum_vote_proposal_percent: Option<u8>,
		minimum_vote_turnout_percent: Option<u8>,
		minimum_vote_pass_percent: Option<u8>,
		max_expiry_time_seconds: Option<u32>,
		admin: Option<Addr>
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
pub enum ExecuteMsg {
	/// Instruction can only be activated by the configured admin
	Admin(AdminExecuteMsg),
	Stake,
	Unstake,
	Vote {
		id: u32,
		approval: bool
	},
	DeactivateVotes {
		user: Option<Addr>,
		limit: Option<u32>
	},
	ProposeTransaction {
		msgs: Vec<CosmosMsg<SeiMsg>>,
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
pub enum QueryMsg {
	#[returns(CourtAppConfig)]
	Config,
	#[returns(QueryResponseDenom)]
	Denom,
	#[returns(Option<TransactionProposalInfo>)]
	ProposalInfo {
		id: u32
	},
	#[returns(Vec<QueryResponseTransactionProposal>)]
	GetProposals {
		skip: u32,
		limit: u32,
		descending: bool
	},
	#[returns(CourtUserStats)]
	UserStats {
		user: Addr
	},
	#[returns(Option<CourtUserVoteInfo>)]
	UserVoteInfo {
		user: Addr,
		proposal_id: u32
	},
	#[returns(Vec<QueryResponseUserVote>)]
	GetUserVotes {
		user: Addr,
		skip: u32,
		limit: u32,
		descending: bool
	}
}

#[cw_serde]
pub struct QueryResponseDenom {
	pub votes: String
}

#[cw_serde]
pub struct QueryResponseTransactionProposal {
	pub proposal_id: u32,
	pub status: TransactionProposalStatus,
	pub info: TransactionProposalInfo
}

#[cw_serde]
pub struct QueryResponseUserVote {
	pub proposal_id: u32,
	pub info: CourtUserVoteInfo
}
