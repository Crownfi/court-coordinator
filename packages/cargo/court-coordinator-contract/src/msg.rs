use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Uint128, Addr, CosmosMsg};
use sei_cosmwasm::SeiMsg;

use crate::state::{app::{CourtAppConfig, TransactionProposalStatus, TransactionProposalInfo}, user::{CourtUserStats, CourtUserVoteInfo}};


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
	pub max_expiry_time_seconds: u32,
}

#[cw_serde]
pub enum CourtAdminExecuteMsg {
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
pub enum CourtExecuteMsg {
	/// Instruction can only be activated by the configured admin
	Admin(CourtAdminExecuteMsg),
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
pub enum CourtQueryMsg {
	#[returns(CourtAppConfig)]
	Config,
	#[returns(CourtQueryResponseDenom)]
	Denom,
	#[returns(Option<TransactionProposalInfo>)]
	ProposalInfo {
		id: u32
	},
	#[returns(Vec<CourtQueryResponseTransactionProposal>)]
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
	#[returns(Vec<CourtQueryResponseUserVote>)]
	GetUserVotes {
		test_prop1: Vec<Addr>,
		test_prop2: [u32; 5],
		user: Addr,
		skip: u32,
		limit: u32,
		descending: bool
	}
}

#[cw_serde]
pub struct CourtQueryResponseDenom {
	pub votes: String
}

#[cw_serde]
pub struct CourtQueryResponseTransactionProposal {
	pub proposal_id: u32,
	pub status: TransactionProposalStatus,
	pub info: TransactionProposalInfo
}

#[cw_serde]
pub struct CourtQueryResponseUserVote {
	pub proposal_id: u32,
	pub info: CourtUserVoteInfo
}
