use cosmwasm_schema::{
	cw_serde,
	schemars::{self, JsonSchema},
	QueryResponses,
};
use cosmwasm_std::{Addr, Uint128};
use serde::{Deserialize, Serialize};

use crate::{
	proposed_msg::ProposedCourtMsgJsonable,
	state::{
		app::{CourtAppConfigJsonable, TransactionProposalInfoJsonable, TransactionProposalStatus},
		user::{CourtUserStatsJsonable, CourtUserVoteInfoJsonable, CourtUserVoteStatus},
	},
};

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
	pub vote_share_description: String,
}

#[cw_serde]
pub enum CourtAdminExecuteMsg {
	/// Change config options
	/// 
	/// This cannot be called while proposals can be added
	ChangeConfig {
		minimum_vote_proposal_percent: Option<u8>,
		minimum_vote_turnout_percent: Option<u8>,
		minimum_vote_pass_percent: Option<u8>,
		max_proposal_expiry_time_seconds: Option<u32>,
		execution_expiry_time_seconds: Option<u32>,
	},
	/// Change the admin to the address specified
	/// 
	/// This is an immediate change
	ChangeAdmin {
		admin: Addr,
	},
	/// Set whether or not to allow new proposal
	AllowNewProposals {
		allowed: bool,
	},
	/// Mints new shares, effectively diluting the existing ones
	MintShares {
		receiver: Addr,
		amount: Uint128,
	},
}

#[cw_serde]
pub enum CourtExecuteMsg {
	/// Instruction can only be activated by the configured admin
	Admin(CourtAdminExecuteMsg),
	/// "Stake" your voting shares, doing this is what allows you to vote on proposals
	Stake,
	/// Have your voting shares which you previously staked returned to you.
	/// 
	/// You may not do this while you are voting on active proposals
	Unstake,
	/// Vote on a proposal, you can change your opinion if you'd like, or increase your vote if you stake more.
	Vote {
		id: u32,
		vote: CourtUserVoteStatus,
	},
	/// This must be done before unstaking
	DeactivateVotes {
		user: Option<Addr>,
		limit: Option<u32>,
	},
	/// Propose a new transaction
	ProposeTransaction {
		msgs: Vec<ProposedCourtMsgJsonable>,
		expiry_time_seconds: u32,
	},
	/// If a proposal has passed, this is how you execute it.
	ExecuteProposal {
		id: u32,
	},
}

//#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema)]
//pub enum Cw20ReceiveMsgData {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum CourtQueryMsg {
	/// Gets config
	#[returns(CourtAppConfigJsonable)]
	Config,
	/// Gets denom of the tokens which hold special functionality
	#[returns(CourtQueryResponseDenom)]
	Denom,
	/// Gets the total supply of the tokens which hold special functionality
	#[returns(CourtQueryResponseTotalSupply)]
	TotalSupply,
	/// The number of proposals which exist
	#[returns(u32)]
	ProposalAmount,
	#[returns(Option<CourtQueryResponseTransactionProposal>)]
	/// Gets a specific proposal, may be null
	GetProposal {
		/// The proposal ID
		id: u32
	},
	/// Returns a list of proposals based on the range provided
	#[returns(Vec<CourtQueryResponseTransactionProposal>)]
	GetProposals {
		/// Where to start the array from
		skip: Option<u32>,
		/// The maximum length of the array
		limit: Option<u32>,
		/// if `false`, array will be in ascending order. if `true`, descending order.
		descending: bool,
	},
	#[returns(CourtUserStatsJsonable)]
	UserStats { user: Addr },
	#[returns(CourtUserVoteInfoJsonable)]
	UserVoteInfo { user: Addr, proposal_id: u32 },
	#[returns(Vec<CourtQueryUserWithActiveProposal>)]
	GetUsersWithActiveProposals {
		after: Option<CourtQueryUserWithActiveProposal>,
		limit: Option<u32>,
		descending: bool,
	},
	#[returns(Vec<u32>)]
	GetUserActiveProposals {
		user: Addr,
		skip: Option<u32>,
		limit: Option<u32>,
		descending: bool,
	},
	#[returns(Vec<CourtQueryResponseUserVote>)]
	GetProposalUserVotes {
		proposal_id: u32,
		after: Option<Addr>,
		limit: Option<u32>,
		descending: bool,
	},
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CourtQueryResponseDenom {
	/// The voting shares denom
	pub votes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CourtQueryResponseTotalSupply {
	/// Total supply of voting shares
	pub votes: Uint128,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CourtQueryResponseTransactionProposal {
	pub proposal_id: u32,
	pub status: TransactionProposalStatus,
	pub info: TransactionProposalInfoJsonable,
	pub messages: Vec<ProposedCourtMsgJsonable>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CourtQueryResponseUserVote {
	pub user: Addr,
	pub info: CourtUserVoteInfoJsonable,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CourtQueryUserWithActiveProposal {
	pub user: Addr,
	pub proposal_id: u32,
}
