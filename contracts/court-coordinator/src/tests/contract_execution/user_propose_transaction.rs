use super::*;

#[test]
pub fn unfunded_check() {
	let mut env_deps = new_env_and_instantiate(None);
	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 500000);
	helpers::assert_unfunded_instruction(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
			funds: vec![],
		}),
		CourtExecuteMsg::ProposeTransaction {
			msgs: vec![ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_ACCOUNT_1.into(),
				denom: "usei".into(),
				amount: 1337u128.into(),
			}],
			expiry_time_seconds: 86400,
		},
	);
}

#[test]
pub fn non_empty_check() {
	let mut env_deps = new_env_and_instantiate(None);
	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 500000);
	let execute_result = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
			funds: vec![],
		}),
		CourtExecuteMsg::ProposeTransaction {
			msgs: vec![],
			expiry_time_seconds: 86400,
		},
	);
	assert!(execute_result.is_err_and(|err| { err.to_string().contains("Proposal must have at least one message") }));
}

#[test]
pub fn expire_time_check() {
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

	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 500000);
	let execute_result = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
			funds: vec![],
		}),
		CourtExecuteMsg::ProposeTransaction {
			msgs: vec![ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_ACCOUNT_1.into(),
				denom: "usei".into(),
				amount: 1337u128.into(),
			}],
			expiry_time_seconds: 7201,
		},
	);
	assert!(execute_result.is_err_and(|err| { err.to_string().contains("Proposal takes too long to expire") }));
	helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
			funds: vec![],
		}),
		CourtExecuteMsg::ProposeTransaction {
			msgs: vec![ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_ACCOUNT_1.into(),
				denom: "usei".into(),
				amount: 1337u128.into(),
			}],
			expiry_time_seconds: 7200,
		},
	)
	.unwrap();
}

#[test]
pub fn minimum_vote_check() {
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

	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 99999);
	let execute_result = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
			funds: vec![],
		}),
		CourtExecuteMsg::ProposeTransaction {
			msgs: vec![ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_ACCOUNT_1.into(),
				denom: "usei".into(),
				amount: 1337u128.into(),
			}],
			expiry_time_seconds: 1200,
		},
	);
	assert!(execute_result.is_err_and(|err| {
		err.to_string()
			.contains("User doesn't have enough staked votes to submit a proposal")
	}));
	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 1);

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

	assert_eq!(
		helpers::query_get_proposal(&env_deps, 0),
		Ok(Some(CourtQueryResponseTransactionProposal {
			proposal_id: 0,
			status: TransactionProposalStatus::Pending,
			info: TransactionProposalInfoJsonable {
				proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
				votes_for: 100000u128.into(),
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
	assert_eq!(
		helpers::query_get_proposal_user_votes(&env_deps, 0, None, None, false),
		Ok(vec![CourtQueryResponseUserVote {
			user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
			info: CourtUserVoteInfoJsonable {
				active_votes: 100000u128.into(),
				vote: CourtUserVoteStatus::Approve
			}
		}])
	);
	assert_eq!(
		helpers::query_get_proposals(&env_deps, None, None, false),
		Ok(vec![CourtQueryResponseTransactionProposal {
			proposal_id: 0,
			status: TransactionProposalStatus::Pending,
			info: TransactionProposalInfoJsonable {
				proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
				votes_for: 100000u128.into(),
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
		}])
	);
	assert_eq!(helpers::query_proposal_amount(&env_deps), Ok(1));
	assert_eq!(
		helpers::query_get_user_active_proposals(&env_deps, SHARES_HOLDER_ACCOUNT_1, None, None, false),
		Ok(vec![0])
	);
	assert_eq!(
		helpers::query_get_user_active_proposals(&env_deps, SHARES_HOLDER_ACCOUNT_2, None, None, false),
		Ok(vec![])
	);
	assert_eq!(
		helpers::query_get_users_with_active_proposals(&env_deps, None, None, false),
		Ok(vec![CourtQueryUserWithActiveProposal {
			user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
			proposal_id: 0
		}])
	);
	assert_eq!(
		helpers::query_user_vote_info(&env_deps, SHARES_HOLDER_ACCOUNT_1, 0),
		Ok(CourtUserVoteInfoJsonable {
			active_votes: 100000u128.into(),
			vote: CourtUserVoteStatus::Approve
		})
	);
	assert_eq!(
		helpers::query_user_vote_info(&env_deps, SHARES_HOLDER_ACCOUNT_2, 0),
		Ok(CourtUserVoteInfoJsonable {
			active_votes: 0u128.into(),
			vote: CourtUserVoteStatus::Abstain
		})
	);
	assert_eq!(
		helpers::query_user_vote_info(&env_deps, SHARES_HOLDER_ACCOUNT_1, 1),
		Ok(CourtUserVoteInfoJsonable {
			active_votes: 0u128.into(),
			vote: CourtUserVoteStatus::Abstain
		})
	);
}

#[test]
pub fn multiple() {
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

	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 100000);
	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_2, 150000);

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
	helpers::execute_propose_transaction(
		&mut env_deps,
		SHARES_HOLDER_ACCOUNT_1,
		vec![ProposedCourtMsgJsonable::SendCoin {
			to: RANDOM_ACCOUNT_3.into(),
			denom: "usei".into(),
			amount: 1338u128.into(),
		}],
		1300,
	);
	helpers::execute_propose_transaction(
		&mut env_deps,
		SHARES_HOLDER_ACCOUNT_2,
		vec![ProposedCourtMsgJsonable::SendCoin {
			to: RANDOM_ACCOUNT_2.into(),
			denom: "usei".into(),
			amount: 1339u128.into(),
		}],
		1400,
	);

	assert_eq!(
		helpers::query_get_proposal(&env_deps, 0),
		Ok(Some(CourtQueryResponseTransactionProposal {
			proposal_id: 0,
			status: TransactionProposalStatus::Pending,
			info: TransactionProposalInfoJsonable {
				proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
				votes_for: 100000u128.into(),
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
	assert_eq!(
		helpers::query_get_proposal(&env_deps, 1),
		Ok(Some(CourtQueryResponseTransactionProposal {
			proposal_id: 1,
			status: TransactionProposalStatus::Pending,
			info: TransactionProposalInfoJsonable {
				proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
				votes_for: 100000u128.into(),
				votes_against: 0u128.into(),
				votes_abstain: 0u128.into(),
				execution_status: TransactionProposalExecutionStatus::NotExecuted,
				expiry_timestamp_ms: env_deps.0.block.time.plus_seconds(1300).millis()
			},
			messages: vec![ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_ACCOUNT_3.into(),
				denom: "usei".into(),
				amount: 1338u128.into()
			}]
		}))
	);
	assert_eq!(
		helpers::query_get_proposal(&env_deps, 2),
		Ok(Some(CourtQueryResponseTransactionProposal {
			proposal_id: 2,
			status: TransactionProposalStatus::Pending,
			info: TransactionProposalInfoJsonable {
				proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
				votes_for: 150000u128.into(),
				votes_against: 0u128.into(),
				votes_abstain: 0u128.into(),
				execution_status: TransactionProposalExecutionStatus::NotExecuted,
				expiry_timestamp_ms: env_deps.0.block.time.plus_seconds(1400).millis()
			},
			messages: vec![ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_ACCOUNT_2.into(),
				denom: "usei".into(),
				amount: 1339u128.into()
			}]
		}))
	);
	assert_eq!(
		helpers::query_get_proposal_user_votes(&env_deps, 0, None, None, false),
		Ok(vec![CourtQueryResponseUserVote {
			user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
			info: CourtUserVoteInfoJsonable {
				active_votes: 100000u128.into(),
				vote: CourtUserVoteStatus::Approve
			}
		}])
	);
	assert_eq!(
		helpers::query_get_proposal_user_votes(&env_deps, 1, None, None, false),
		Ok(vec![CourtQueryResponseUserVote {
			user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
			info: CourtUserVoteInfoJsonable {
				active_votes: 100000u128.into(),
				vote: CourtUserVoteStatus::Approve
			}
		}])
	);
	assert_eq!(
		helpers::query_get_proposal_user_votes(&env_deps, 2, None, None, false),
		Ok(vec![CourtQueryResponseUserVote {
			user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
			info: CourtUserVoteInfoJsonable {
				active_votes: 150000u128.into(),
				vote: CourtUserVoteStatus::Approve
			}
		}])
	);

	assert_eq!(
		helpers::query_get_proposals(&env_deps, None, None, false),
		Ok(vec![
			CourtQueryResponseTransactionProposal {
				proposal_id: 0,
				status: TransactionProposalStatus::Pending,
				info: TransactionProposalInfoJsonable {
					proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
					votes_for: 100000u128.into(),
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
			},
			CourtQueryResponseTransactionProposal {
				proposal_id: 1,
				status: TransactionProposalStatus::Pending,
				info: TransactionProposalInfoJsonable {
					proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
					votes_for: 100000u128.into(),
					votes_against: 0u128.into(),
					votes_abstain: 0u128.into(),
					execution_status: TransactionProposalExecutionStatus::NotExecuted,
					expiry_timestamp_ms: env_deps.0.block.time.plus_seconds(1300).millis()
				},
				messages: vec![ProposedCourtMsgJsonable::SendCoin {
					to: RANDOM_ACCOUNT_3.into(),
					denom: "usei".into(),
					amount: 1338u128.into()
				}]
			},
			CourtQueryResponseTransactionProposal {
				proposal_id: 2,
				status: TransactionProposalStatus::Pending,
				info: TransactionProposalInfoJsonable {
					proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
					votes_for: 150000u128.into(),
					votes_against: 0u128.into(),
					votes_abstain: 0u128.into(),
					execution_status: TransactionProposalExecutionStatus::NotExecuted,
					expiry_timestamp_ms: env_deps.0.block.time.plus_seconds(1400).millis()
				},
				messages: vec![ProposedCourtMsgJsonable::SendCoin {
					to: RANDOM_ACCOUNT_2.into(),
					denom: "usei".into(),
					amount: 1339u128.into()
				}]
			}
		])
	);
	assert_eq!(
		helpers::query_get_proposals(&env_deps, Some(1), Some(1), false),
		Ok(vec![CourtQueryResponseTransactionProposal {
			proposal_id: 1,
			status: TransactionProposalStatus::Pending,
			info: TransactionProposalInfoJsonable {
				proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
				votes_for: 100000u128.into(),
				votes_against: 0u128.into(),
				votes_abstain: 0u128.into(),
				execution_status: TransactionProposalExecutionStatus::NotExecuted,
				expiry_timestamp_ms: env_deps.0.block.time.plus_seconds(1300).millis()
			},
			messages: vec![ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_ACCOUNT_3.into(),
				denom: "usei".into(),
				amount: 1338u128.into()
			}]
		}])
	);
	assert_eq!(
		helpers::query_get_proposals(&env_deps, None, None, true),
		Ok(vec![
			CourtQueryResponseTransactionProposal {
				proposal_id: 2,
				status: TransactionProposalStatus::Pending,
				info: TransactionProposalInfoJsonable {
					proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
					votes_for: 150000u128.into(),
					votes_against: 0u128.into(),
					votes_abstain: 0u128.into(),
					execution_status: TransactionProposalExecutionStatus::NotExecuted,
					expiry_timestamp_ms: env_deps.0.block.time.plus_seconds(1400).millis()
				},
				messages: vec![ProposedCourtMsgJsonable::SendCoin {
					to: RANDOM_ACCOUNT_2.into(),
					denom: "usei".into(),
					amount: 1339u128.into()
				}]
			},
			CourtQueryResponseTransactionProposal {
				proposal_id: 1,
				status: TransactionProposalStatus::Pending,
				info: TransactionProposalInfoJsonable {
					proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
					votes_for: 100000u128.into(),
					votes_against: 0u128.into(),
					votes_abstain: 0u128.into(),
					execution_status: TransactionProposalExecutionStatus::NotExecuted,
					expiry_timestamp_ms: env_deps.0.block.time.plus_seconds(1300).millis()
				},
				messages: vec![ProposedCourtMsgJsonable::SendCoin {
					to: RANDOM_ACCOUNT_3.into(),
					denom: "usei".into(),
					amount: 1338u128.into()
				}]
			},
			CourtQueryResponseTransactionProposal {
				proposal_id: 0,
				status: TransactionProposalStatus::Pending,
				info: TransactionProposalInfoJsonable {
					proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
					votes_for: 100000u128.into(),
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
			},
		])
	);
	assert_eq!(
		helpers::query_get_proposals(&env_deps, Some(2), None, true),
		Ok(vec![CourtQueryResponseTransactionProposal {
			proposal_id: 0,
			status: TransactionProposalStatus::Pending,
			info: TransactionProposalInfoJsonable {
				proposer: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
				votes_for: 100000u128.into(),
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
		},])
	);
	assert_eq!(helpers::query_proposal_amount(&env_deps), Ok(3));
	assert_eq!(
		helpers::query_get_user_active_proposals(&env_deps, SHARES_HOLDER_ACCOUNT_1, None, None, false),
		Ok(vec![0, 1])
	);
	assert_eq!(
		helpers::query_get_user_active_proposals(&env_deps, SHARES_HOLDER_ACCOUNT_2, None, None, false),
		Ok(vec![2])
	);
	assert_eq!(
		helpers::query_get_user_active_proposals(&env_deps, SHARES_HOLDER_ACCOUNT_1, None, Some(1), false),
		Ok(vec![0])
	);
	assert_eq!(
		helpers::query_get_user_active_proposals(&env_deps, SHARES_HOLDER_ACCOUNT_1, None, Some(1), true),
		Ok(vec![1])
	);
	assert_eq!(
		helpers::query_get_user_active_proposals(&env_deps, SHARES_HOLDER_ACCOUNT_1, Some(1), None, false),
		Ok(vec![1])
	);
	assert_eq!(
		helpers::query_get_user_active_proposals(&env_deps, SHARES_HOLDER_ACCOUNT_1, Some(1), None, true),
		Ok(vec![0])
	);
	assert_eq!(
		helpers::query_get_users_with_active_proposals(&env_deps, None, None, false),
		Ok(vec![
			CourtQueryUserWithActiveProposal {
				user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
				proposal_id: 0
			},
			CourtQueryUserWithActiveProposal {
				user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
				proposal_id: 1
			},
			CourtQueryUserWithActiveProposal {
				user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
				proposal_id: 2
			}
		])
	);
	assert_eq!(
		helpers::query_get_users_with_active_proposals(&env_deps, None, None, true),
		Ok(vec![
			CourtQueryUserWithActiveProposal {
				user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
				proposal_id: 2
			},
			CourtQueryUserWithActiveProposal {
				user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
				proposal_id: 1
			},
			CourtQueryUserWithActiveProposal {
				user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
				proposal_id: 0
			}
		])
	);
	assert_eq!(
		helpers::query_get_users_with_active_proposals(&env_deps, None, Some(1), false),
		Ok(vec![CourtQueryUserWithActiveProposal {
			user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
			proposal_id: 0
		}])
	);
	assert_eq!(
		helpers::query_get_users_with_active_proposals(
			&env_deps,
			Some(CourtQueryUserWithActiveProposal {
				user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
				proposal_id: 0
			}),
			None,
			false
		),
		Ok(vec![
			CourtQueryUserWithActiveProposal {
				user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
				proposal_id: 1
			},
			CourtQueryUserWithActiveProposal {
				user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
				proposal_id: 2
			}
		])
	);
	assert_eq!(
		helpers::query_get_users_with_active_proposals(
			&env_deps,
			Some(CourtQueryUserWithActiveProposal {
				user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
				proposal_id: 2
			}),
			Some(1),
			true
		),
		Ok(vec![CourtQueryUserWithActiveProposal {
			user: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
			proposal_id: 1
		},])
	);

	assert_eq!(
		helpers::query_user_vote_info(&env_deps, SHARES_HOLDER_ACCOUNT_1, 0),
		Ok(CourtUserVoteInfoJsonable {
			active_votes: 100000u128.into(),
			vote: CourtUserVoteStatus::Approve
		})
	);
	assert_eq!(
		helpers::query_user_vote_info(&env_deps, SHARES_HOLDER_ACCOUNT_2, 0),
		Ok(CourtUserVoteInfoJsonable {
			active_votes: 0u128.into(),
			vote: CourtUserVoteStatus::Abstain
		})
	);
	assert_eq!(
		helpers::query_user_vote_info(&env_deps, SHARES_HOLDER_ACCOUNT_1, 1),
		Ok(CourtUserVoteInfoJsonable {
			active_votes: 100000u128.into(),
			vote: CourtUserVoteStatus::Approve
		})
	);
	assert_eq!(
		helpers::query_user_vote_info(&env_deps, SHARES_HOLDER_ACCOUNT_2, 1),
		Ok(CourtUserVoteInfoJsonable {
			active_votes: 0u128.into(),
			vote: CourtUserVoteStatus::Abstain
		})
	);
	assert_eq!(
		helpers::query_user_vote_info(&env_deps, SHARES_HOLDER_ACCOUNT_1, 2),
		Ok(CourtUserVoteInfoJsonable {
			active_votes: 0u128.into(),
			vote: CourtUserVoteStatus::Abstain
		})
	);
	assert_eq!(
		helpers::query_user_vote_info(&env_deps, SHARES_HOLDER_ACCOUNT_2, 2),
		Ok(CourtUserVoteInfoJsonable {
			active_votes: 150000u128.into(),
			vote: CourtUserVoteStatus::Approve
		})
	);
}

#[test]
pub fn token_recipiant_check() {
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

	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 100000);
	let execute_result = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
			funds: vec![],
		}),
		CourtExecuteMsg::ProposeTransaction {
			msgs: vec![ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_EVM_ACCOUNT_1.into(),
				denom: "usei".into(),
				amount: 1337u128.into(),
			}],
			expiry_time_seconds: 1200,
		},
	);
	assert!(execute_result.is_err_and(|err| {
		err.to_string()
			.contains("an address beginning with \"sei1\" is required")
	}));

	let execute_result = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
			funds: vec![],
		}),
		CourtExecuteMsg::ProposeTransaction {
			msgs: vec![ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_ACCOUNT_1.into(),
				denom: format!("erc20/{}", RANDOM_EVM_ACCOUNT_1).into(),
				amount: 1337u128.into(),
			}],
			expiry_time_seconds: 1200,
		},
	);
	assert!(
		execute_result.is_err_and(|err| { err.to_string().contains("an address beginning with \"0x\" is required") })
	);
	helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
			funds: vec![],
		}),
		CourtExecuteMsg::ProposeTransaction {
			msgs: vec![ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_EVM_ACCOUNT_1.into(),
				denom: format!("erc20/{}", RANDOM_EVM_ACCOUNT_1).into(),
				amount: 1337u128.into(),
			}],
			expiry_time_seconds: 1200,
		},
	)
	.unwrap();
}

#[test]
pub fn user_vote_only_shares_check() {}
