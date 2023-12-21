use cosmwasm_std::StdError;
use thiserror::Error;

use crate::state::app::TransactionProposalStatus;

#[derive(Error, Debug, PartialEq)]
pub enum CourtContractError {
	#[error("{0}")]
	Std(StdError),
	#[error("Permission denied: {0}")]
	Unauthorized(String),
	#[error("This instruction shouldn't have been funded with {0}")]
	UnexpectedFunds(String),
	#[error("Expected token {0} but got {1}")]
	TokenMismatch(String, String),
	#[error("Expected token {0} missing")]
	TokenMissing(String),
	#[error("Proposal {0} must have failed or have been passed and executed")]
	ProposalNotFinalized(u32),
	#[error("You don't have any votes staked")]
	NoStakedVotes,
	#[error("Your staked votes must not be tied to any proposals")]
	VotesActive,
	#[error("You've already voted on this proposal")]
	AlreadyVoted,
	#[error("Your new votes cannot contradict your previous votes")]
	VotingForBothSides,
	#[error("Proposal status should be \"{expected}\" for this operation but it is currently \"{actual}\"")]
	UnexpectedProposalStatus {
		expected: TransactionProposalStatus,
		actual: TransactionProposalStatus
	},
	#[error("Proposal must have at least one message")]
	EmptyProposal,
	#[error("Proposal takes too long to expire")]
	ProposalLivesTooLong,
	#[error("You don't have enough staked votes to submit a proposal")]
	InsufficientVotesForProposal,
	#[error("New proposals currently aren't being accepted")]
	NewProposalsNotAllowed
}

impl<E> From<E> for CourtContractError where E: Into<StdError> {
	fn from(value: E) -> Self {
		Self::Std(value.into())
	}
}
