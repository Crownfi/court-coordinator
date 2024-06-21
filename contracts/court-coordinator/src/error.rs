use crownfi_cw_common::impl_from_cosmwasm_std_error_common;
use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

use crate::state::app::TransactionProposalStatus;

#[derive(Error, Debug, PartialEq)]
pub enum CourtContractError {
	#[error("{0}")]
	Std(#[from] StdError),
	#[error("Payment Error: {0}")]
	PaymentError(#[from] PaymentError),
	#[error("Permission denied: {0}")]
	Unauthorized(String),
	#[error("Proposal {0} must have failed or have been passed and executed")]
	ProposalNotFinalized(u32),
	#[error("You don't have any votes staked")]
	NoStakedVotes,
	#[error("Your staked votes must not be tied to any proposals")]
	VotesActive,
	#[error("You've already voted on this proposal")]
	AlreadyVoted,
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
	NewProposalsNotAllowed,
	#[error("This contract cannot safely operate with the amount of new shares minted")]
	TooManyVotesToMint,
	#[error("Doing this may result in this contract becoming unusable")]
	WouldLockupContract,
	#[error("Invalid address \"{wrong_addr}\", an address beginning with \"0x\" is required for {proprety_name}")]
	EvmAddressRequired {
		wrong_addr: String,
		proprety_name: String
	},
	#[error("Invalid address \"{wrong_addr}\", an address beginning with \"sei1\" is required for {proprety_name}")]
	SeiAddressRequired {
		wrong_addr: String,
		proprety_name: String
	}
}
impl_from_cosmwasm_std_error_common!(CourtContractError);
