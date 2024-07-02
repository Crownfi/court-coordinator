use super::*;

#[test]
pub fn user_unstake_votes_unfunded_check() {
	let mut env_deps = new_env_and_instantiate(None);

	helpers::assert_unfunded_instruction(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
			funds: vec![],
		}),
		CourtExecuteMsg::Unstake,
	);
}

#[test]
pub fn user_unstake_votes_must_have_staked_check() {
	let mut env_deps = new_env_and_instantiate(None);
	assert!(helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
			funds: vec![]
		}),
		CourtExecuteMsg::Unstake
	)
	.is_err_and(|err| { err.to_string() == "No user votes staked" }));
}

#[test]
pub fn user_unstake_votes_only_when_not_in_pending_proposals() {
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
	assert!(helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
			funds: vec![]
		}),
		CourtExecuteMsg::Unstake
	)
	.is_err_and(|err| { err.to_string() == "Staked votes must not be tied to any proposals" }));
}

#[test]
pub fn user_unstake_votes_tokens_actually_returned() {
	let mut env_deps = new_env_and_instantiate(None);
	let vote_shares_denom = helpers::query_denom(&env_deps).unwrap().votes;
	let user1_stake_amount_1 = 2147u128;
	let user1_stake_amount_2 = 2563u128;
	let user1_stake_amount_total = 4710u128;

	let user2_stake_amount_1 = 567u128;
	let user2_stake_amount_2 = 2531u128;
	let user2_stake_amount_total = 3098u128;

	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, user1_stake_amount_1);
	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_2, user2_stake_amount_1);
	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, user1_stake_amount_2);
	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_2, user2_stake_amount_2);

	let execute_response = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
			funds: vec![],
		}),
		CourtExecuteMsg::Unstake,
	)
	.unwrap();

	// Sends the staked amount of tokens back to sender
	assert!(execute_response.messages.iter().any(|sub_msg| {
		match &sub_msg.msg {
			cosmwasm_std::CosmosMsg::Bank(sub_msg) => {
				*sub_msg
					== cosmwasm_std::BankMsg::Send {
						to_address: SHARES_HOLDER_ACCOUNT_2.into(),
						amount: vec![coin(user2_stake_amount_total, &vote_shares_denom)],
					}
			}
			_ => false,
		}
	}));
	assert_eq!(
		execute_response.events,
		vec![cosmwasm_std::Event::new("unstake")
			.add_attribute("user", SHARES_HOLDER_ACCOUNT_2)
			.add_attribute("user_total_votes", user2_stake_amount_total.to_string())]
	);
	assert_eq!(
		helpers::query_user_stats(&mut env_deps, SHARES_HOLDER_ACCOUNT_2).unwrap(),
		CourtUserStatsJsonable {
			staked_votes: 0u128.into()
		}
	);
	assert_eq!(
		helpers::query_user_stats(&mut env_deps, SHARES_HOLDER_ACCOUNT_1).unwrap(),
		CourtUserStatsJsonable {
			staked_votes: user1_stake_amount_total.into()
		}
	);

	let execute_response = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
			funds: vec![],
		}),
		CourtExecuteMsg::Unstake,
	)
	.unwrap();

	// Sends the staked amount of tokens back to sender
	assert!(execute_response.messages.iter().any(|sub_msg| {
		match &sub_msg.msg {
			cosmwasm_std::CosmosMsg::Bank(sub_msg) => {
				*sub_msg
					== cosmwasm_std::BankMsg::Send {
						to_address: SHARES_HOLDER_ACCOUNT_1.into(),
						amount: vec![coin(user1_stake_amount_total, &vote_shares_denom)],
					}
			}
			_ => false,
		}
	}));
	assert_eq!(
		execute_response.events,
		vec![cosmwasm_std::Event::new("unstake")
			.add_attribute("user", SHARES_HOLDER_ACCOUNT_1)
			.add_attribute("user_total_votes", user1_stake_amount_total.to_string())]
	);
	assert_eq!(
		helpers::query_user_stats(&mut env_deps, SHARES_HOLDER_ACCOUNT_1).unwrap(),
		CourtUserStatsJsonable {
			staked_votes: 0u128.into()
		}
	);
}
