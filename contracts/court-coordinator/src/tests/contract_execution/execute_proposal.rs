use cosmwasm_std::Binary;
use sei_cosmwasm::SeiMsg;

use super::*;
#[test]
pub fn unfunded_check() {
	let mut env_deps = new_env_and_instantiate(None);
	let proposal_id = helpers::execute_create_unanimous_proposal(
		&mut env_deps,
		vec![ProposedCourtMsgJsonable::SendCoin {
			to: RANDOM_ACCOUNT_1.into(),
			denom: "usei".into(),
			amount: 1337u128.into(),
		}],
		420,
		SHARES_HOLDER_ACCOUNT_1,
	);
	helpers::advance_time_to_vote_end(&mut env_deps, proposal_id);
	// Sanity check
	assert_eq!(
		helpers::query_get_proposal(&env_deps, proposal_id)
			.unwrap()
			.unwrap()
			.status,
		TransactionProposalStatus::Passed
	);

	helpers::assert_unfunded_instruction(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(RANDOM_ACCOUNT_5),
			funds: vec![],
		}),
		CourtExecuteMsg::ExecuteProposal { id: proposal_id },
	);
}

#[test]
pub fn permissionless_execution_check() {
	let mut env_deps = new_env_and_instantiate(None);
	let proposal_id = helpers::execute_create_unanimous_proposal(
		&mut env_deps,
		vec![
			ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_ACCOUNT_1.into(),
				denom: "usei".into(),
				amount: 1337u128.into(),
			},
			ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_ACCOUNT_2.into(),
				denom: "usei".into(),
				amount: 42069u128.into(),
			},
		],
		420,
		SHARES_HOLDER_ACCOUNT_1,
	);
	helpers::advance_time_to_vote_end(&mut env_deps, proposal_id);
	// Sanity check
	assert_eq!(
		helpers::query_get_proposal(&env_deps, proposal_id)
			.unwrap()
			.unwrap()
			.status,
		TransactionProposalStatus::Passed
	);

	// Random account which has nothing to do with the proposal
	let execute_result = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(RANDOM_ACCOUNT_5),
			funds: vec![],
		}),
		CourtExecuteMsg::ExecuteProposal { id: proposal_id },
	)
	.unwrap();
	assert_eq!(
		execute_result.events,
		vec![cosmwasm_std::Event::new("proposal_executed").add_attribute("proposal_id", proposal_id.to_string())]
	);
	assert_eq!(
		execute_result.messages,
		vec![
			cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::<SeiMsg>::Bank(cosmwasm_std::BankMsg::Send {
				to_address: RANDOM_ACCOUNT_1.into(),
				amount: vec![coin(1337, "usei")]
			})),
			cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::<SeiMsg>::Bank(cosmwasm_std::BankMsg::Send {
				to_address: RANDOM_ACCOUNT_2.into(),
				amount: vec![coin(42069, "usei")]
			}))
		]
	);
}

#[test]
pub fn different_proposal_types() {
	let mut env_deps = new_env_and_instantiate(None);
	let proposal_id = helpers::execute_create_unanimous_proposal(
		&mut env_deps,
		vec![
			ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_ACCOUNT_1.into(),
				denom: "usei".into(),
				amount: 1337u128.into(),
			},
			ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_ACCOUNT_2.into(),
				denom: format!("cw20/{RANDOM_CONTRACT}").into(),
				amount: 1337u128.into(),
			},
			ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_EVM_ACCOUNT_1.into(),
				denom: format!("erc20/{RANDOM_EVM_ACCOUNT_1}").into(),
				amount: 1337u128.into(),
			},
			ProposedCourtMsgJsonable::ExecuteEvmContract {
				contract: RANDOM_EVM_ACCOUNT_1.into(),
				msg: Binary(vec![0x3e, 0xfd, 0x97, 0x4f]),
				value: 0u128.into(),
			},
			ProposedCourtMsgJsonable::ExecuteWasmContract {
				contract: Addr::unchecked(RANDOM_CONTRACT),
				msg: Binary("\"ayy_lmao\"".as_bytes().into()),
				funds: vec![coin(69, "usei")],
			},
			ProposedCourtMsgJsonable::ChangeWasmContractAdmin {
				contract: Addr::unchecked(RANDOM_CONTRACT),
				new_admin: Addr::unchecked(RANDOM_ACCOUNT_4),
			},
			ProposedCourtMsgJsonable::ClearWasmContractAdmin {
				contract: Addr::unchecked(RANDOM_CONTRACT),
			},
			ProposedCourtMsgJsonable::TokenfactoryMint {
				tokens: coin(42069, format!("factory/{RANDOM_CONTRACT}/ayylmao")),
			},
		],
		420,
		SHARES_HOLDER_ACCOUNT_1,
	);

	helpers::advance_time_to_vote_end(&mut env_deps, proposal_id);
	// Sanity check
	assert_eq!(
		helpers::query_get_proposal(&env_deps, proposal_id)
			.unwrap()
			.unwrap()
			.status,
		TransactionProposalStatus::Passed
	);

	let execute_result = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(RANDOM_ACCOUNT_5),
			funds: vec![],
		}),
		CourtExecuteMsg::ExecuteProposal { id: proposal_id },
	)
	.unwrap();
	assert_eq!(
		execute_result.events,
		vec![cosmwasm_std::Event::new("proposal_executed").add_attribute("proposal_id", proposal_id.to_string())]
	);

	assert_eq!(execute_result.messages.len(), 8);
	assert_eq!(
		execute_result.messages[0],
		cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::<SeiMsg>::Bank(cosmwasm_std::BankMsg::Send {
			to_address: RANDOM_ACCOUNT_1.into(),
			amount: vec![coin(1337, "usei")]
		}))
	);
	assert_eq!(
		execute_result.messages[1],
		cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::<SeiMsg>::Wasm(
			cosmwasm_std::WasmMsg::Execute {
				contract_addr: RANDOM_CONTRACT.into(),
				msg: Binary(
					format!("{{\"transfer\":{{\"recipient\":\"{RANDOM_ACCOUNT_2}\",\"amount\":\"1337\"}}}}")
						.as_bytes()
						.into()
				),
				funds: vec![]
			}
		))
	);
	assert_eq!(
		execute_result.messages[2],
		cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::<SeiMsg>::Custom(SeiMsg::CallEvm {
			value: 0u128.into(),
			to: RANDOM_EVM_ACCOUNT_1.to_ascii_lowercase().into(),
			// ERC20 transfer of 1337 tokens to RANDOM_EVM_ACCOUNT_1
			data: "qQWcuwAAAAAAAAAAAAAAAGkgc3BpbGwgbXkgZHJpbmsgVF9UAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABTk=".into()
		}))
	);
	assert_eq!(
		execute_result.messages[3],
		cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::<SeiMsg>::Custom(SeiMsg::CallEvm {
			value: 0u128.into(),
			to: RANDOM_EVM_ACCOUNT_1.to_ascii_lowercase().into(),
			// Buffer.from([0x3e, 0xfd, 0x97, 0x4f]).toString("base64")
			data: "Pv2XTw==".into()
		}))
	);
	assert_eq!(
		execute_result.messages[4],
		cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::<SeiMsg>::Wasm(
			cosmwasm_std::WasmMsg::Execute {
				contract_addr: RANDOM_CONTRACT.into(),
				msg: Binary("\"ayy_lmao\"".as_bytes().into()),
				funds: vec![coin(69, "usei")],
			}
		))
	);
	assert_eq!(
		execute_result.messages[5],
		cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::<SeiMsg>::Wasm(
			cosmwasm_std::WasmMsg::UpdateAdmin {
				contract_addr: RANDOM_CONTRACT.into(),
				admin: RANDOM_ACCOUNT_4.into()
			}
		))
	);
	assert_eq!(
		execute_result.messages[6],
		cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::<SeiMsg>::Wasm(
			cosmwasm_std::WasmMsg::ClearAdmin {
				contract_addr: RANDOM_CONTRACT.into()
			}
		))
	);
	assert_eq!(
		execute_result.messages[7],
		cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::<SeiMsg>::Custom(SeiMsg::MintTokens {
			amount: coin(42069, format!("factory/{RANDOM_CONTRACT}/ayylmao"))
		}))
	);
}

#[test]
pub fn only_execute_on_passed_state() {
	let mut env_deps = new_env_and_instantiate(None);

	// Case 1: Passing proposal expires and config options changed since then
	let proposal_id = helpers::execute_create_passing_proposal(
		&mut env_deps,
		vec![ProposedCourtMsgJsonable::SendCoin {
			to: RANDOM_ACCOUNT_1.into(),
			denom: "usei".into(),
			amount: 1337u128.into(),
		}],
		420,
		SHARES_HOLDER_ACCOUNT_1,
		SHARES_HOLDER_ACCOUNT_2,
	);
	assert_eq!(
		helpers::query_get_proposal(&env_deps, proposal_id)
			.unwrap()
			.unwrap()
			.status,
		TransactionProposalStatus::Pending
	);

	let execute_response = helpers::execute(
		&mut env_deps,
		None,
		CourtExecuteMsg::ExecuteProposal { id: proposal_id },
	);
	assert!(execute_response.is_err_and(|err| {
		err.to_string()
			.contains("status should be \"passed\" for this operation")
	}));

	helpers::advance_time_to_vote_end(&mut env_deps, proposal_id);
	assert_eq!(
		helpers::query_get_proposal(&env_deps, proposal_id)
			.unwrap()
			.unwrap()
			.status,
		TransactionProposalStatus::Passed
	);
	helpers::advance_time_to_execution_expiry(&mut env_deps, proposal_id);
	assert_eq!(
		helpers::query_get_proposal(&env_deps, proposal_id)
			.unwrap()
			.unwrap()
			.status,
		TransactionProposalStatus::ExecutionExpired
	);

	let execute_response = helpers::execute(
		&mut env_deps,
		None,
		CourtExecuteMsg::ExecuteProposal { id: proposal_id },
	);
	assert!(execute_response.is_err_and(|err| {
		err.to_string()
			.contains("status should be \"passed\" for this operation")
	}));

	helpers::execute_deactivate_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1);
	helpers::execute_deactivate_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_2);
	helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(ADMIN_ACCOUNT),
			funds: vec![],
		}),
		CourtExecuteMsg::Admin(CourtAdminExecuteMsg::ChangeConfig {
			minimum_vote_proposal_percent: None,
			minimum_vote_turnout_percent: None,
			minimum_vote_pass_percent: None,
			max_proposal_expiry_time_seconds: None,
			execution_expiry_time_seconds: None,
		}),
	)
	.unwrap();
	assert_eq!(
		helpers::query_get_proposal(&env_deps, proposal_id)
			.unwrap()
			.unwrap()
			.status,
		TransactionProposalStatus::RejectedOrExpired
	);
	let execute_response = helpers::execute(
		&mut env_deps,
		None,
		CourtExecuteMsg::ExecuteProposal { id: proposal_id },
	);
	assert!(execute_response.is_err_and(|err| {
		err.to_string()
			.contains("status should be \"passed\" for this operation")
	}));
	helpers::execute_deactivate_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1);
	helpers::execute_deactivate_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_2);

	// Case 2: Passing proposal passes at the end of voting period
	let proposal_id = helpers::execute_create_passing_proposal(
		&mut env_deps,
		vec![ProposedCourtMsgJsonable::SendCoin {
			to: RANDOM_ACCOUNT_1.into(),
			denom: "usei".into(),
			amount: 1337u128.into(),
		}],
		420,
		SHARES_HOLDER_ACCOUNT_1,
		SHARES_HOLDER_ACCOUNT_2,
	);
	assert_eq!(
		helpers::query_get_proposal(&env_deps, proposal_id)
			.unwrap()
			.unwrap()
			.status,
		TransactionProposalStatus::Pending
	);

	let execute_response = helpers::execute(
		&mut env_deps,
		None,
		CourtExecuteMsg::ExecuteProposal { id: proposal_id },
	);
	assert!(execute_response.is_err_and(|err| {
		err.to_string()
			.contains("status should be \"passed\" for this operation")
	}));

	helpers::advance_time_to_vote_end(&mut env_deps, proposal_id);
	assert_eq!(
		helpers::query_get_proposal(&env_deps, proposal_id)
			.unwrap()
			.unwrap()
			.status,
		TransactionProposalStatus::Passed
	);

	helpers::execute(
		&mut env_deps,
		None,
		CourtExecuteMsg::ExecuteProposal { id: proposal_id },
	)
	.unwrap();
	assert_eq!(
		helpers::query_get_proposal(&env_deps, proposal_id)
			.unwrap()
			.unwrap()
			.status,
		TransactionProposalStatus::Executed
	);

	let execute_response = helpers::execute(
		&mut env_deps,
		None,
		CourtExecuteMsg::ExecuteProposal { id: proposal_id },
	);
	assert!(execute_response.is_err_and(|err| {
		err.to_string()
			.contains("status should be \"passed\" for this operation")
	}));
	helpers::execute_deactivate_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1);
	helpers::execute_deactivate_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_2);

	// Case 3: Passing proposal passes immediately due to passing the threshold
	let proposal_id = helpers::execute_create_guaranteed_passing_proposal(
		&mut env_deps,
		vec![ProposedCourtMsgJsonable::SendCoin {
			to: RANDOM_ACCOUNT_1.into(),
			denom: "usei".into(),
			amount: 1337u128.into(),
		}],
		420,
		SHARES_HOLDER_ACCOUNT_1,
	);
	assert_eq!(
		helpers::query_get_proposal(&env_deps, proposal_id)
			.unwrap()
			.unwrap()
			.status,
		TransactionProposalStatus::Passed
	);

	helpers::execute(
		&mut env_deps,
		None,
		CourtExecuteMsg::ExecuteProposal { id: proposal_id },
	)
	.unwrap();
	assert_eq!(
		helpers::query_get_proposal(&env_deps, proposal_id)
			.unwrap()
			.unwrap()
			.status,
		TransactionProposalStatus::Executed
	);

	let execute_response = helpers::execute(
		&mut env_deps,
		None,
		CourtExecuteMsg::ExecuteProposal { id: proposal_id },
	);
	assert!(execute_response.is_err_and(|err| {
		err.to_string()
			.contains("status should be \"passed\" for this operation")
	}));
	helpers::execute_deactivate_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1);
	helpers::execute_deactivate_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_2);

	// Case 4: Proposal failed due to low voter turnout
	let proposal_id = helpers::execute_create_unanimous_proposal_with_low_turnout(
		&mut env_deps,
		vec![ProposedCourtMsgJsonable::SendCoin {
			to: RANDOM_ACCOUNT_1.into(),
			denom: "usei".into(),
			amount: 1337u128.into(),
		}],
		420,
		SHARES_HOLDER_ACCOUNT_1,
	);
	assert_eq!(
		helpers::query_get_proposal(&env_deps, proposal_id)
			.unwrap()
			.unwrap()
			.status,
		TransactionProposalStatus::Pending
	);

	let execute_response = helpers::execute(
		&mut env_deps,
		None,
		CourtExecuteMsg::ExecuteProposal { id: proposal_id },
	);
	assert!(execute_response.is_err_and(|err| {
		err.to_string()
			.contains("status should be \"passed\" for this operation")
	}));

	helpers::advance_time_to_vote_end(&mut env_deps, proposal_id);
	assert_eq!(
		helpers::query_get_proposal(&env_deps, proposal_id)
			.unwrap()
			.unwrap()
			.status,
		TransactionProposalStatus::Rejected
	);

	let execute_response = helpers::execute(
		&mut env_deps,
		None,
		CourtExecuteMsg::ExecuteProposal { id: proposal_id },
	);
	assert!(execute_response.is_err_and(|err| {
		err.to_string()
			.contains("status should be \"passed\" for this operation")
	}));
	helpers::execute_deactivate_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1);
	helpers::execute_deactivate_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_2);

	// Case 5: Proposal failed due to negative votes
	let proposal_id = helpers::execute_create_failing_proposal(
		&mut env_deps,
		vec![ProposedCourtMsgJsonable::SendCoin {
			to: RANDOM_ACCOUNT_1.into(),
			denom: "usei".into(),
			amount: 1337u128.into(),
		}],
		420,
		SHARES_HOLDER_ACCOUNT_1,
		SHARES_HOLDER_ACCOUNT_2,
	);
	assert_eq!(
		helpers::query_get_proposal(&env_deps, proposal_id)
			.unwrap()
			.unwrap()
			.status,
		TransactionProposalStatus::Pending
	);

	let execute_response = helpers::execute(
		&mut env_deps,
		None,
		CourtExecuteMsg::ExecuteProposal { id: proposal_id },
	);
	assert!(execute_response.is_err_and(|err| {
		err.to_string()
			.contains("status should be \"passed\" for this operation")
	}));

	helpers::advance_time_to_vote_end(&mut env_deps, proposal_id);
	assert_eq!(
		helpers::query_get_proposal(&env_deps, proposal_id)
			.unwrap()
			.unwrap()
			.status,
		TransactionProposalStatus::Rejected
	);
}

#[test]
pub fn mint_proposals_work() {
	let mut env_deps = new_env_and_instantiate(None);
	let vote_denom = helpers::query_denom(&env_deps).unwrap().votes;
	let new_token_supply = helpers::get_known_vote_supply(&env_deps) + 1337;
	let proposal_id = helpers::execute_create_guaranteed_passing_proposal(
		&mut env_deps,
		vec![ProposedCourtMsgJsonable::TokenfactoryMint {
			tokens: coin(1337, vote_denom),
		}],
		420,
		SHARES_HOLDER_ACCOUNT_1,
	);
	assert_eq!(
		helpers::query_get_proposal(&env_deps, proposal_id)
			.unwrap()
			.unwrap()
			.status,
		TransactionProposalStatus::Passed
	);
	helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(RANDOM_ACCOUNT_5),
			funds: vec![],
		}),
		CourtExecuteMsg::ExecuteProposal { id: proposal_id },
	)
	.unwrap();
	assert_eq!(helpers::get_known_vote_supply(&env_deps), new_token_supply);
}

#[test]
pub fn prevent_mint_proposal_overflow() {
	let mut env_deps = new_env_and_instantiate(None);
	let vote_denom = helpers::query_denom(&env_deps).unwrap().votes;
	let tokens_to_mint = u128::MAX
		.div_ceil(10000)
		.saturating_sub(helpers::get_known_vote_supply(&env_deps))
		+ 1;
	let proposal_id = helpers::execute_create_guaranteed_passing_proposal(
		&mut env_deps,
		vec![ProposedCourtMsgJsonable::TokenfactoryMint {
			tokens: coin(tokens_to_mint, vote_denom),
		}],
		420,
		SHARES_HOLDER_ACCOUNT_1,
	);
	assert_eq!(
		helpers::query_get_proposal(&env_deps, proposal_id)
			.unwrap()
			.unwrap()
			.status,
		TransactionProposalStatus::Passed
	);
	let execute_response = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(RANDOM_ACCOUNT_5),
			funds: vec![],
		}),
		CourtExecuteMsg::ExecuteProposal { id: proposal_id },
	);
	assert!(execute_response.is_err_and(|err| {
		err.to_string()
			.contains("cannot safely operate with the amount of new shares minted")
	}));
	helpers::advance_time_to_execution_expiry(&mut env_deps, proposal_id);
	// Might as well double check that users can deactivate their votes on a passed yet expired thing.
	helpers::execute_deactivate_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1);
}

#[test]
pub fn abstain_votes_count_towards_turnout() {
	let mut env_deps = new_env_and_instantiate(None);
	let proposal_id = helpers::execute_create_unanimous_proposal(
		&mut env_deps,
		vec![ProposedCourtMsgJsonable::SendCoin {
			to: RANDOM_ACCOUNT_1.into(),
			denom: "usei".into(),
			amount: 1337u128.into(),
		}],
		420,
		SHARES_HOLDER_ACCOUNT_1,
	);
	helpers::execute_stake_exact_amount(&mut env_deps, SHARES_HOLDER_ACCOUNT_2, 1);
	helpers::execute_vote(
		&mut env_deps,
		SHARES_HOLDER_ACCOUNT_1,
		proposal_id,
		CourtUserVoteStatus::Abstain,
	);
	helpers::execute_vote(
		&mut env_deps,
		SHARES_HOLDER_ACCOUNT_2,
		proposal_id,
		CourtUserVoteStatus::Approve,
	);

	helpers::advance_time_to_vote_end(&mut env_deps, proposal_id);
	// Sanity check
	assert_eq!(
		helpers::query_get_proposal(&env_deps, proposal_id)
			.unwrap()
			.unwrap()
			.status,
		TransactionProposalStatus::Passed
	);

	// Random account which has nothing to do with the proposal
	let execute_result = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(RANDOM_ACCOUNT_5),
			funds: vec![],
		}),
		CourtExecuteMsg::ExecuteProposal { id: proposal_id },
	)
	.unwrap();
	assert_eq!(
		execute_result.events,
		vec![cosmwasm_std::Event::new("proposal_executed").add_attribute("proposal_id", proposal_id.to_string())]
	);
	assert_eq!(
		execute_result.messages,
		vec![cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::<SeiMsg>::Bank(
			cosmwasm_std::BankMsg::Send {
				to_address: RANDOM_ACCOUNT_1.into(),
				amount: vec![coin(1337, "usei")]
			}
		))]
	);
}

#[test]
pub fn only_absatain_votes() {
	let mut env_deps = new_env_and_instantiate(None);
	let proposal_id = helpers::execute_create_unanimous_proposal(
		&mut env_deps,
		vec![ProposedCourtMsgJsonable::SendCoin {
			to: RANDOM_ACCOUNT_1.into(),
			denom: "usei".into(),
			amount: 1337u128.into(),
		}],
		420,
		SHARES_HOLDER_ACCOUNT_1,
	);
	helpers::execute_vote(
		&mut env_deps,
		SHARES_HOLDER_ACCOUNT_1,
		proposal_id,
		CourtUserVoteStatus::Abstain,
	);

	helpers::advance_time_to_vote_end(&mut env_deps, proposal_id);
	// Sanity check
	assert_eq!(
		helpers::query_get_proposal(&env_deps, proposal_id)
			.unwrap()
			.unwrap()
			.status,
		TransactionProposalStatus::Rejected
	);

	// Random account which has nothing to do with the proposal
	let execute_result = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(RANDOM_ACCOUNT_5),
			funds: vec![],
		}),
		CourtExecuteMsg::ExecuteProposal { id: proposal_id },
	);
	assert!(execute_result.is_err_and(|err| {
		err.to_string()
			.contains("status should be \"passed\" for this operation")
	}));
}
