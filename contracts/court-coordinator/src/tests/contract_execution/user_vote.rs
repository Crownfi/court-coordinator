use super::*;

#[test]
pub fn unfunded_check() {
	let mut env_deps = new_env_and_instantiate(None);
	// Sanity tests for current config we're testing against
	assert_eq!(
		helpers::query_config(&env_deps).unwrap().minimum_vote_proposal_percent,
		10
	);
	assert_eq!(
		helpers::query_config(&env_deps)
			.unwrap()
			.max_proposal_expiry_time_seconds,
		7200
	);
	assert_eq!(helpers::get_known_vote_supply(&env_deps), 1000000);

	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 140000);
	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_2, 110000);
	helpers::execute_propose_transaction(
		&mut env_deps,
		SHARES_HOLDER_ACCOUNT_1,
		vec![ProposedCourtMsgJsonable::SendCoin {
			to: RANDOM_ACCOUNT_1.into(),
			denom: "usei".into(),
			amount: 1337u128.into(),
		}],
		1200,
	);

	helpers::assert_unfunded_instruction(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
			funds: vec![],
		}),
		CourtExecuteMsg::Vote {
			id: 0,
			vote: CourtUserVoteStatus::Oppose,
		},
	);

	helpers::execute_propose_transaction(
		&mut env_deps,
		SHARES_HOLDER_ACCOUNT_1,
		vec![ProposedCourtMsgJsonable::SendCoin {
			to: RANDOM_ACCOUNT_1.into(),
			denom: "usei".into(),
			amount: 1337u128.into(),
		}],
		1200,
	);

	helpers::assert_unfunded_instruction(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
			funds: vec![],
		}),
		CourtExecuteMsg::Vote {
			id: 1,
			vote: CourtUserVoteStatus::Oppose,
		},
	);
}

#[test]
pub fn vote_on_proposals_which_exist() {
	let mut env_deps = new_env_and_instantiate(None);
	// Sanity tests for current config we're testing against
	assert_eq!(
		helpers::query_config(&env_deps).unwrap().minimum_vote_proposal_percent,
		10
	);
	assert_eq!(
		helpers::query_config(&env_deps)
			.unwrap()
			.max_proposal_expiry_time_seconds,
		7200
	);
	assert_eq!(helpers::get_known_vote_supply(&env_deps), 1000000);
	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 140000);
	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_2, 110000);
	let execute_response = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
			funds: vec![],
		}),
		CourtExecuteMsg::Vote {
			id: 0,
			vote: CourtUserVoteStatus::Oppose,
		},
	);
	assert!(execute_response.is_err_and(|err| { err.to_string().contains(" 0 does not exist") }));
	let execute_response = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
			funds: vec![],
		}),
		CourtExecuteMsg::Vote {
			id: 1,
			vote: CourtUserVoteStatus::Oppose,
		},
	);
	assert!(execute_response.is_err_and(|err| { err.to_string().contains(" 1 does not exist") }));
	let execute_response = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
			funds: vec![],
		}),
		CourtExecuteMsg::Vote {
			id: 69420,
			vote: CourtUserVoteStatus::Oppose,
		},
	);
	assert!(execute_response.is_err_and(|err| { err.to_string().contains(" 69420 does not exist") }));

	helpers::execute_propose_transaction(
		&mut env_deps,
		SHARES_HOLDER_ACCOUNT_1,
		vec![ProposedCourtMsgJsonable::SendCoin {
			to: RANDOM_ACCOUNT_1.into(),
			denom: "usei".into(),
			amount: 1337u128.into(),
		}],
		1200,
	);
	helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
			funds: vec![],
		}),
		CourtExecuteMsg::Vote {
			id: 0,
			vote: CourtUserVoteStatus::Oppose,
		},
	)
	.unwrap();
	let execute_response = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
			funds: vec![],
		}),
		CourtExecuteMsg::Vote {
			id: 1,
			vote: CourtUserVoteStatus::Oppose,
		},
	);
	assert!(execute_response.is_err_and(|err| { err.to_string().contains(" 1 does not exist") }));
	helpers::execute_propose_transaction(
		&mut env_deps,
		SHARES_HOLDER_ACCOUNT_1,
		vec![ProposedCourtMsgJsonable::SendCoin {
			to: RANDOM_ACCOUNT_1.into(),
			denom: "usei".into(),
			amount: 1337u128.into(),
		}],
		1200,
	);
	helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
			funds: vec![],
		}),
		CourtExecuteMsg::Vote {
			id: 1,
			vote: CourtUserVoteStatus::Oppose,
		},
	)
	.unwrap();
	let execute_response = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
			funds: vec![],
		}),
		CourtExecuteMsg::Vote {
			id: 2,
			vote: CourtUserVoteStatus::Oppose,
		},
	);
	assert!(execute_response.is_err_and(|err| { err.to_string().contains(" 2 does not exist") }));
}

#[test]
pub fn cannot_vote_past_expiry() {
	let mut env_deps = new_env_and_instantiate(None);
	// Sanity tests for current config we're testing against
	assert_eq!(
		helpers::query_config(&env_deps).unwrap().minimum_vote_proposal_percent,
		10
	);
	assert_eq!(
		helpers::query_config(&env_deps)
			.unwrap()
			.max_proposal_expiry_time_seconds,
		7200
	);
	assert_eq!(helpers::get_known_vote_supply(&env_deps), 1000000);

	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 140000);
	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_2, 110000);
	helpers::execute_propose_transaction(
		&mut env_deps,
		SHARES_HOLDER_ACCOUNT_1,
		vec![ProposedCourtMsgJsonable::SendCoin {
			to: RANDOM_ACCOUNT_1.into(),
			denom: "usei".into(),
			amount: 1337u128.into(),
		}],
		1200,
	);

	env_deps.0.block.time = env_deps.0.block.time.plus_seconds(1200);
	let execute_response = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
			funds: vec![],
		}),
		CourtExecuteMsg::Vote {
			id: 0,
			vote: CourtUserVoteStatus::Oppose,
		},
	);
	assert!(execute_response.is_err_and(|err| {
		err.to_string()
			.contains("status should be \"pending\" for this operation")
	}));
	env_deps.0.block.time = env_deps.0.block.time.minus_seconds(1);
	helpers::execute_vote(&mut env_deps, SHARES_HOLDER_ACCOUNT_2, 0, CourtUserVoteStatus::Oppose);

	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 100000);
	helpers::execute_propose_transaction(
		&mut env_deps,
		SHARES_HOLDER_ACCOUNT_1,
		vec![ProposedCourtMsgJsonable::SendCoin {
			to: RANDOM_ACCOUNT_1.into(),
			denom: "usei".into(),
			amount: 1337u128.into(),
		}],
		6200,
	);
	env_deps.0.block.time = env_deps.0.block.time.plus_seconds(6200);
	let execute_response = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
			funds: vec![],
		}),
		CourtExecuteMsg::Vote {
			id: 1,
			vote: CourtUserVoteStatus::Oppose,
		},
	);
	assert!(execute_response.is_err_and(|err| {
		err.to_string()
			.contains("status should be \"pending\" for this operation")
	}));
}

#[test]
pub fn allow_vote_change() {
	let mut env_deps = new_env_and_instantiate(None);
	// Sanity tests for current config we're testing against
	assert_eq!(
		helpers::query_config(&env_deps).unwrap().minimum_vote_proposal_percent,
		10
	);
	assert_eq!(
		helpers::query_config(&env_deps)
			.unwrap()
			.max_proposal_expiry_time_seconds,
		7200
	);
	assert_eq!(helpers::get_known_vote_supply(&env_deps), 1000000);

	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 140000);
	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_2, 110000);
	helpers::execute_propose_transaction(
		&mut env_deps,
		SHARES_HOLDER_ACCOUNT_1,
		vec![ProposedCourtMsgJsonable::SendCoin {
			to: RANDOM_ACCOUNT_1.into(),
			denom: "usei".into(),
			amount: 1337u128.into(),
		}],
		1200,
	);
	// Users who haven't voted with "abstain" with 0 votes.
	assert_eq!(
		helpers::query_user_vote_info(&env_deps, SHARES_HOLDER_ACCOUNT_2, 0),
		Ok(CourtUserVoteInfoJsonable {
			active_votes: 0u128.into(),
			vote: CourtUserVoteStatus::Abstain
		})
	);
	assert_eq!(
		helpers::query_get_proposal(&env_deps, 0),
		Ok(Some(CourtQueryResponseTransactionProposal {
			proposal_id: 0,
			status: TransactionProposalStatus::Pending,
			info: TransactionProposalInfoJsonable {
				proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
				votes_for: 140000u128.into(),
				votes_against: 0u128.into(),
				votes_abstain: 0u128.into(),
				execution_status: TransactionProposalExecutionStatus::NotExecuted,
				expiry_timestamp_ms: env_deps.0.block.time.plus_seconds(1200).millis()
			},
			messages: vec![ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_ACCOUNT_1.into(),
				denom: "usei".into(),
				amount: 1337u128.into()
			}]
		}))
	);

	// The helper function asserts the change to the user stats
	helpers::execute_vote(&mut env_deps, SHARES_HOLDER_ACCOUNT_2, 0, CourtUserVoteStatus::Approve);
	assert_eq!(
		helpers::query_get_proposal(&env_deps, 0),
		Ok(Some(CourtQueryResponseTransactionProposal {
			proposal_id: 0,
			status: TransactionProposalStatus::Pending,
			info: TransactionProposalInfoJsonable {
				proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
				votes_for: 250000u128.into(),
				votes_against: 0u128.into(),
				votes_abstain: 0u128.into(),
				execution_status: TransactionProposalExecutionStatus::NotExecuted,
				expiry_timestamp_ms: env_deps.0.block.time.plus_seconds(1200).millis()
			},
			messages: vec![ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_ACCOUNT_1.into(),
				denom: "usei".into(),
				amount: 1337u128.into()
			}]
		}))
	);
	helpers::execute_vote(&mut env_deps, SHARES_HOLDER_ACCOUNT_2, 0, CourtUserVoteStatus::Abstain);
	assert_eq!(
		helpers::query_get_proposal(&env_deps, 0),
		Ok(Some(CourtQueryResponseTransactionProposal {
			proposal_id: 0,
			status: TransactionProposalStatus::Pending,
			info: TransactionProposalInfoJsonable {
				proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
				votes_for: 140000u128.into(),
				votes_against: 0u128.into(),
				votes_abstain: 110000u128.into(),
				execution_status: TransactionProposalExecutionStatus::NotExecuted,
				expiry_timestamp_ms: env_deps.0.block.time.plus_seconds(1200).millis()
			},
			messages: vec![ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_ACCOUNT_1.into(),
				denom: "usei".into(),
				amount: 1337u128.into()
			}]
		}))
	);
	helpers::execute_vote(&mut env_deps, SHARES_HOLDER_ACCOUNT_2, 0, CourtUserVoteStatus::Oppose);
	assert_eq!(
		helpers::query_get_proposal(&env_deps, 0),
		Ok(Some(CourtQueryResponseTransactionProposal {
			proposal_id: 0,
			status: TransactionProposalStatus::Pending,
			info: TransactionProposalInfoJsonable {
				proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
				votes_for: 140000u128.into(),
				votes_against: 110000u128.into(),
				votes_abstain: 0u128.into(),
				execution_status: TransactionProposalExecutionStatus::NotExecuted,
				expiry_timestamp_ms: env_deps.0.block.time.plus_seconds(1200).millis()
			},
			messages: vec![ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_ACCOUNT_1.into(),
				denom: "usei".into(),
				amount: 1337u128.into()
			}]
		}))
	);
	// Cannot vote for the same position twice
	let execute_response = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
			funds: vec![],
		}),
		CourtExecuteMsg::Vote {
			id: 0,
			vote: CourtUserVoteStatus::Oppose,
		},
	);
	assert!(execute_response.is_err_and(|err| { err.to_string().contains("already voted on this proposal") }));

	// Even the proposer can change their position (if it is changed)
	let execute_response = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
			funds: vec![],
		}),
		CourtExecuteMsg::Vote {
			id: 0,
			vote: CourtUserVoteStatus::Approve,
		},
	);
	assert!(execute_response.is_err_and(|err| { err.to_string().contains("already voted on this proposal") }));

	helpers::execute_vote(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 0, CourtUserVoteStatus::Abstain);
	assert_eq!(
		helpers::query_get_proposal(&env_deps, 0),
		Ok(Some(CourtQueryResponseTransactionProposal {
			proposal_id: 0,
			status: TransactionProposalStatus::Pending,
			info: TransactionProposalInfoJsonable {
				proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
				votes_for: 0u128.into(),
				votes_against: 110000u128.into(),
				votes_abstain: 140000u128.into(),
				execution_status: TransactionProposalExecutionStatus::NotExecuted,
				expiry_timestamp_ms: env_deps.0.block.time.plus_seconds(1200).millis()
			},
			messages: vec![ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_ACCOUNT_1.into(),
				denom: "usei".into(),
				amount: 1337u128.into()
			}]
		}))
	);
	helpers::execute_vote(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 0, CourtUserVoteStatus::Oppose);
	assert_eq!(
		helpers::query_get_proposal(&env_deps, 0),
		Ok(Some(CourtQueryResponseTransactionProposal {
			proposal_id: 0,
			status: TransactionProposalStatus::Pending,
			info: TransactionProposalInfoJsonable {
				proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
				votes_for: 0u128.into(),
				votes_against: 250000u128.into(),
				votes_abstain: 0u128.into(),
				execution_status: TransactionProposalExecutionStatus::NotExecuted,
				expiry_timestamp_ms: env_deps.0.block.time.plus_seconds(1200).millis()
			},
			messages: vec![ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_ACCOUNT_1.into(),
				denom: "usei".into(),
				amount: 1337u128.into()
			}]
		}))
	);
}

#[test]
pub fn allow_vote_increase() {
	let mut env_deps = new_env_and_instantiate(None);
	// Sanity tests for current config we're testing against
	assert_eq!(
		helpers::query_config(&env_deps).unwrap().minimum_vote_proposal_percent,
		10
	);
	assert_eq!(
		helpers::query_config(&env_deps)
			.unwrap()
			.max_proposal_expiry_time_seconds,
		7200
	);
	assert_eq!(helpers::get_known_vote_supply(&env_deps), 1000000);

	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 140000);
	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_2, 110000);
	helpers::execute_propose_transaction(
		&mut env_deps,
		SHARES_HOLDER_ACCOUNT_1,
		vec![ProposedCourtMsgJsonable::SendCoin {
			to: RANDOM_ACCOUNT_1.into(),
			denom: "usei".into(),
			amount: 1337u128.into(),
		}],
		1200,
	);
	helpers::execute_vote(&mut env_deps, SHARES_HOLDER_ACCOUNT_2, 0, CourtUserVoteStatus::Oppose);

	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 140000);
	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_2, 110000);

	// execute_vote already asserts that the new increase in stake got acknowledged
	helpers::execute_vote(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 0, CourtUserVoteStatus::Approve);
	helpers::execute_vote(&mut env_deps, SHARES_HOLDER_ACCOUNT_2, 0, CourtUserVoteStatus::Oppose);
	// UNFINISHED, needs to actually vote
}

#[test]
pub fn emits_vote_event() {
	let mut env_deps = new_env_and_instantiate(None);
	// Sanity tests for current config we're testing against
	assert_eq!(
		helpers::query_config(&env_deps).unwrap().minimum_vote_proposal_percent,
		10
	);
	assert_eq!(
		helpers::query_config(&env_deps)
			.unwrap()
			.max_proposal_expiry_time_seconds,
		7200
	);
	assert_eq!(helpers::get_known_vote_supply(&env_deps), 1000000);

	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 140000);
	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_2, 110000);
	helpers::execute_propose_transaction(
		&mut env_deps,
		SHARES_HOLDER_ACCOUNT_1,
		vec![ProposedCourtMsgJsonable::SendCoin {
			to: RANDOM_ACCOUNT_1.into(),
			denom: "usei".into(),
			amount: 1337u128.into(),
		}],
		1200,
	);

	let execute_response = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
			funds: vec![],
		}),
		CourtExecuteMsg::Vote {
			id: 0,
			vote: CourtUserVoteStatus::Oppose,
		},
	)
	.unwrap();
	assert_eq!(
		execute_response.events,
		vec![cosmwasm_std::Event::new("vote")
			.add_attribute("proposal_id", "0")
			.add_attribute("voter", SHARES_HOLDER_ACCOUNT_2)
			.add_attribute("votes", "110000")
			.add_attribute("vote", "oppose")]
	);
	let execute_response = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
			funds: vec![],
		}),
		CourtExecuteMsg::Vote {
			id: 0,
			vote: CourtUserVoteStatus::Abstain,
		},
	)
	.unwrap();
	assert_eq!(
		execute_response.events,
		vec![cosmwasm_std::Event::new("vote")
			.add_attribute("proposal_id", "0")
			.add_attribute("voter", SHARES_HOLDER_ACCOUNT_2)
			.add_attribute("votes", "110000")
			.add_attribute("vote", "abstain")]
	);
	let execute_response = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
			funds: vec![],
		}),
		CourtExecuteMsg::Vote {
			id: 0,
			vote: CourtUserVoteStatus::Approve,
		},
	)
	.unwrap();
	assert_eq!(
		execute_response.events,
		vec![cosmwasm_std::Event::new("vote")
			.add_attribute("proposal_id", "0")
			.add_attribute("voter", SHARES_HOLDER_ACCOUNT_2)
			.add_attribute("votes", "110000")
			.add_attribute("vote", "approve")]
	);
}
