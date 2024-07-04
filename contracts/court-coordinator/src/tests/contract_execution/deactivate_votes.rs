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
		CourtExecuteMsg::DeactivateVotes { user:None, limit: None },
	);
}

#[test]
pub fn deactivates_senders_votes() {
	let mut env_deps = new_env_and_instantiate(None);
	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 500000);
	helpers::execute_propose_transaction(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, vec![
		ProposedCourtMsgJsonable::SendCoin {
			to: RANDOM_ACCOUNT_1.into(),
			denom: "usei".into(),
			amount: 1337u128.into(),
		}
	], 420);

	helpers::advance_time_to_execution_expiry(&mut env_deps, 0);

	let execute_response = helpers::execute(
		&mut env_deps,
		Some(
			MessageInfo { sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), funds: vec![] }
		),
		CourtExecuteMsg::DeactivateVotes { user:None, limit: None }
	).unwrap();
	assert_eq!(execute_response.events.len(), 0);
	assert_eq!(execute_response.messages.len(), 0);

	helpers::execute(
		&mut env_deps,
		Some(
			MessageInfo { sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), funds: vec![] }
		),
		CourtExecuteMsg::Unstake
	).unwrap();
}

#[test]
pub fn permissionless() {
	let mut env_deps = new_env_and_instantiate(None);
	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 500000);
	helpers::execute_propose_transaction(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, vec![
		ProposedCourtMsgJsonable::SendCoin {
			to: RANDOM_ACCOUNT_1.into(),
			denom: "usei".into(),
			amount: 1337u128.into(),
		}
	], 420);

	helpers::advance_time_to_execution_expiry(&mut env_deps, 0);

	let execute_response = helpers::execute(
		&mut env_deps,
		Some(
			MessageInfo { sender: Addr::unchecked(RANDOM_ACCOUNT_2), funds: vec![] }
		),
		CourtExecuteMsg::DeactivateVotes { user: Some(Addr::unchecked(SHARES_HOLDER_ACCOUNT_1)), limit: None }
	).unwrap();
	assert_eq!(execute_response.events.len(), 0);
	assert_eq!(execute_response.messages.len(), 0);

	helpers::execute(
		&mut env_deps,
		Some(
			MessageInfo { sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), funds: vec![] }
		),
		CourtExecuteMsg::Unstake
	).unwrap();
}

#[test]
pub fn non_finalized_proposal_check() {
	let mut env_deps = new_env_and_instantiate(None);
	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, 500000);
	helpers::execute_propose_transaction(&mut env_deps, SHARES_HOLDER_ACCOUNT_1, vec![
		ProposedCourtMsgJsonable::SendCoin {
			to: RANDOM_ACCOUNT_1.into(),
			denom: "usei".into(),
			amount: 1337u128.into(),
		}
	], 420);
	let execute_response = helpers::execute(
		&mut env_deps,
		Some(
			MessageInfo { sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1), funds: vec![] }
		),
		CourtExecuteMsg::DeactivateVotes { user: None, limit: None }
	);
	assert!(execute_response.is_err_and(|err| {err.to_string() == "Proposal 0 must have failed or have been passed and executed"}))
}
