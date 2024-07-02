use super::*;
use crate::{
	msg::*,
	proposed_msg::ProposedCourtMsgJsonable,
	state::{
		app::{
			CourtAppConfigJsonable, TransactionProposalExecutionStatus, TransactionProposalInfoJsonable,
			TransactionProposalStatus,
		},
		user::{CourtUserStatsJsonable, CourtUserVoteInfoJsonable, CourtUserVoteStatus},
	},
};
use cosmwasm_std::{coin, MessageInfo};
use cw2::{get_contract_version, ContractVersion};
use helpers::{get_known_vote_supply, new_env_and_instantiate};
mod admin_change_admin;
mod admin_change_config;
mod admin_disallow_new_proposals;
mod admin_mint_shares;
mod helpers;
mod user_propose_transaction;
mod user_stake_votes;
mod user_unstake_votes;

#[test]
pub fn instantiate() {
	let mut env_deps = new_global_env();
	let contract_addr = &env_deps.0.contract.address.clone();
	let vote_shares_denom = format!("factory/{}/votes", contract_addr);

	let instantiate_response = helpers::instantiate(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(ADMIN_ACCOUNT),
			funds: vec![coin(1337, "usei")],
		}),
		None,
	);

	// instantiate is unfunded
	assert!(instantiate_response.is_err_and(|err| {
		// Typo won't be fixed until we upgrade to cosmwasm-std 2.x
		err.to_string().contains("does no accept funds") || err.to_string().contains("does not accept funds")
	}));
	let instantiate_response = helpers::instantiate(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(RANDOM_ACCOUNT_1),
			funds: vec![],
		}),
		Some(CourtInstantiateMsg {
			admin: Addr::unchecked(RANDOM_ACCOUNT_2),
			shares_mint_amount: 1404438u128.into(),
			shares_mint_receiver: Addr::unchecked(RANDOM_ACCOUNT_3),
			minimum_vote_proposal_percent: 27,
			minimum_vote_turnout_percent: 20,
			minimum_vote_pass_percent: 28,
			max_proposal_expiry_time_seconds: 3428,
			execution_expiry_time_seconds: 2506,
			vote_share_name: "Test vote tokens".into(),
			vote_share_symbol: "TVT".into(),
			vote_share_description: "Will everyone named Mike please stand".into(),
		}),
	)
	.unwrap();

	// Adheres to cw2
	assert!(get_contract_version(&env_deps.1.storage).is_ok_and(|info| {
		info.eq(&ContractVersion {
			contract: "court-coordinator-contract".into(),
			version: env!("CARGO_PKG_VERSION").into(),
		})
	}));

	// Config is what's applied
	assert!(helpers::query_config(&env_deps).is_ok_and(|config| {
		config.eq(&CourtAppConfigJsonable {
			allow_new_proposals: true,
			minimum_vote_proposal_percent: 27,
			minimum_vote_turnout_percent: 20,
			minimum_vote_pass_percent: 28,
			max_proposal_expiry_time_seconds: 3428,
			execution_expiry_time_seconds: 2506,
			last_config_change_timestamp_ms: env_deps.0.block.time.millis(),
			admin: Addr::unchecked(RANDOM_ACCOUNT_2),
		})
	}));

	// Queried votes denom is correct
	assert!(helpers::query_denom(&env_deps).is_ok_and(|denoms| { denoms.votes == vote_shares_denom }));

	// Creates the votes denom
	assert!(instantiate_response.messages.iter().any(|sub_msg| {
		match &sub_msg.msg {
			cosmwasm_std::CosmosMsg::Custom(sub_msg) => match sub_msg {
				sei_cosmwasm::SeiMsg::CreateDenom { subdenom } => subdenom == "votes",
				_ => false,
			},
			_ => false,
		}
	}));

	// Mints the amount specified
	assert!(instantiate_response.messages.iter().any(|sub_msg| {
		match &sub_msg.msg {
			cosmwasm_std::CosmosMsg::Custom(sub_msg) => match sub_msg {
				sei_cosmwasm::SeiMsg::MintTokens { amount } => *amount == coin(1404438u128, &vote_shares_denom),
				_ => false,
			},
			_ => false,
		}
	}));

	// Sends the amount of tokens minted to the user specified
	assert!(instantiate_response.messages.iter().any(|sub_msg| {
		match &sub_msg.msg {
			cosmwasm_std::CosmosMsg::Bank(sub_msg) => {
				*sub_msg
					== cosmwasm_std::BankMsg::Send {
						to_address: RANDOM_ACCOUNT_3.into(),
						amount: vec![coin(1404438u128, &vote_shares_denom)],
					}
			}
			_ => false,
		}
	}));

	assert_eq!(get_known_vote_supply(&env_deps), 1404438u128);

	// Registers the metadata
	assert!(instantiate_response.messages.iter().any(|sub_msg| {
		match &sub_msg.msg {
			cosmwasm_std::CosmosMsg::Custom(sub_msg) => match sub_msg {
				sei_cosmwasm::SeiMsg::SetMetadata { metadata } => {
					*metadata
						== sei_cosmwasm::Metadata {
							description: "Will everyone named Mike please stand".into(),
							denom_units: vec![
								sei_cosmwasm::DenomUnit {
									denom: vote_shares_denom.clone(),
									exponent: 0,
									aliases: vec!["utvt".into(), "microtvt".into()],
								},
								sei_cosmwasm::DenomUnit {
									denom: "mtvt".into(),
									exponent: 3,
									aliases: vec!["millitvt".into()],
								},
								sei_cosmwasm::DenomUnit {
									denom: "tvt".into(),
									exponent: 6,
									aliases: vec![],
								},
							],
							base: vote_shares_denom.clone(),
							display: "tvt".into(),
							name: "Test vote tokens".into(),
							symbol: "TVT".into(),
						}
				}
				_ => false,
			},
			_ => false,
		}
	}));
}
