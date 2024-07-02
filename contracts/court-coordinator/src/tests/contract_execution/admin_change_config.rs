use super::*;

#[test]
pub fn unfunded_check() {
	let mut env_deps = new_env_and_instantiate(None);

	helpers::assert_unfunded_instruction(
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
}

#[test]
pub fn authorized_check() {
	let mut env_deps = new_env_and_instantiate(None);

	helpers::assert_only_authorized_instruction(
		&mut env_deps,
		&[],
		&[
			RANDOM_ACCOUNT_1,
			RANDOM_ACCOUNT_2,
			RANDOM_ACCOUNT_3,
			RANDOM_ACCOUNT_4,
			RANDOM_ACCOUNT_5,
			SHARES_HOLDER_ACCOUNT_1,
			SHARES_HOLDER_ACCOUNT_2,
			SHARES_HOLDER_ACCOUNT_3,
			SHARES_HOLDER_ACCOUNT_4,
			SHARES_HOLDER_ACCOUNT_5,
		],
		&[ADMIN_ACCOUNT],
		CourtExecuteMsg::Admin(CourtAdminExecuteMsg::ChangeConfig {
			minimum_vote_proposal_percent: None,
			minimum_vote_turnout_percent: None,
			minimum_vote_pass_percent: None,
			max_proposal_expiry_time_seconds: None,
			execution_expiry_time_seconds: None,
		}),
	);
}

#[test]
pub fn correct() {
	// Nothing changes if nothing is specified to change
	let mut env_deps = helpers::new_env_and_instantiate(None);
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
		helpers::query_config(&env_deps).unwrap(),
		// Config is what's applied
		CourtAppConfigJsonable {
			allow_new_proposals: true,
			minimum_vote_proposal_percent: 10,
			minimum_vote_turnout_percent: 20,
			minimum_vote_pass_percent: 50,
			max_proposal_expiry_time_seconds: 7200,
			execution_expiry_time_seconds: 3600,
			last_config_change_timestamp_ms: env_deps.0.block.time.millis(),
			admin: Addr::unchecked(ADMIN_ACCOUNT)
		}
	);

	drop(env_deps); // Must drop otherwise we deadlock
	let mut env_deps = helpers::new_env_and_instantiate(None);
	helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(ADMIN_ACCOUNT),
			funds: vec![],
		}),
		CourtExecuteMsg::Admin(CourtAdminExecuteMsg::ChangeConfig {
			minimum_vote_proposal_percent: Some(69),
			minimum_vote_turnout_percent: None,
			minimum_vote_pass_percent: None,
			max_proposal_expiry_time_seconds: None,
			execution_expiry_time_seconds: None,
		}),
	)
	.unwrap();
	assert_eq!(
		helpers::query_config(&env_deps).unwrap(),
		// Config is what's applied
		CourtAppConfigJsonable {
			allow_new_proposals: true,
			minimum_vote_proposal_percent: 69,
			minimum_vote_turnout_percent: 20,
			minimum_vote_pass_percent: 50,
			max_proposal_expiry_time_seconds: 7200,
			execution_expiry_time_seconds: 3600,
			last_config_change_timestamp_ms: env_deps.0.block.time.millis(),
			admin: Addr::unchecked(ADMIN_ACCOUNT)
		}
	);

	drop(env_deps); // Must drop otherwise we deadlock
	let mut env_deps = helpers::new_env_and_instantiate(None);
	helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(ADMIN_ACCOUNT),
			funds: vec![],
		}),
		CourtExecuteMsg::Admin(CourtAdminExecuteMsg::ChangeConfig {
			minimum_vote_proposal_percent: None,
			minimum_vote_turnout_percent: Some(69),
			minimum_vote_pass_percent: None,
			max_proposal_expiry_time_seconds: None,
			execution_expiry_time_seconds: None,
		}),
	)
	.unwrap();
	assert_eq!(
		helpers::query_config(&env_deps).unwrap(),
		// Config is what's applied
		CourtAppConfigJsonable {
			allow_new_proposals: true,
			minimum_vote_proposal_percent: 10,
			minimum_vote_turnout_percent: 69,
			minimum_vote_pass_percent: 50,
			max_proposal_expiry_time_seconds: 7200,
			execution_expiry_time_seconds: 3600,
			last_config_change_timestamp_ms: env_deps.0.block.time.millis(),
			admin: Addr::unchecked(ADMIN_ACCOUNT)
		}
	);

	drop(env_deps); // Must drop otherwise we deadlock
	let mut env_deps = helpers::new_env_and_instantiate(None);
	helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(ADMIN_ACCOUNT),
			funds: vec![],
		}),
		CourtExecuteMsg::Admin(CourtAdminExecuteMsg::ChangeConfig {
			minimum_vote_proposal_percent: None,
			minimum_vote_turnout_percent: None,
			minimum_vote_pass_percent: Some(69),
			max_proposal_expiry_time_seconds: None,
			execution_expiry_time_seconds: None,
		}),
	)
	.unwrap();
	assert_eq!(
		helpers::query_config(&env_deps).unwrap(),
		// Config is what's applied
		CourtAppConfigJsonable {
			allow_new_proposals: true,
			minimum_vote_proposal_percent: 10,
			minimum_vote_turnout_percent: 20,
			minimum_vote_pass_percent: 69,
			max_proposal_expiry_time_seconds: 7200,
			execution_expiry_time_seconds: 3600,
			last_config_change_timestamp_ms: env_deps.0.block.time.millis(),
			admin: Addr::unchecked(ADMIN_ACCOUNT)
		}
	);

	drop(env_deps); // Must drop otherwise we deadlock
	let mut env_deps = helpers::new_env_and_instantiate(None);
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
			max_proposal_expiry_time_seconds: Some(69),
			execution_expiry_time_seconds: None,
		}),
	)
	.unwrap();
	assert_eq!(
		helpers::query_config(&env_deps).unwrap(),
		// Config is what's applied
		CourtAppConfigJsonable {
			allow_new_proposals: true,
			minimum_vote_proposal_percent: 10,
			minimum_vote_turnout_percent: 20,
			minimum_vote_pass_percent: 50,
			max_proposal_expiry_time_seconds: 69,
			execution_expiry_time_seconds: 3600,
			last_config_change_timestamp_ms: env_deps.0.block.time.millis(),
			admin: Addr::unchecked(ADMIN_ACCOUNT)
		}
	);

	drop(env_deps); // Must drop otherwise we deadlock
	let mut env_deps = helpers::new_env_and_instantiate(None);
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
			execution_expiry_time_seconds: Some(69),
		}),
	)
	.unwrap();
	assert_eq!(
		helpers::query_config(&env_deps).unwrap(),
		// Config is what's applied
		CourtAppConfigJsonable {
			allow_new_proposals: true,
			minimum_vote_proposal_percent: 10,
			minimum_vote_turnout_percent: 20,
			minimum_vote_pass_percent: 50,
			max_proposal_expiry_time_seconds: 7200,
			execution_expiry_time_seconds: 69,
			last_config_change_timestamp_ms: env_deps.0.block.time.millis(),
			admin: Addr::unchecked(ADMIN_ACCOUNT)
		}
	);
	env_deps.0.block.time = env_deps.0.block.time.plus_minutes(1);

	helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(ADMIN_ACCOUNT),
			funds: vec![],
		}),
		CourtExecuteMsg::Admin(CourtAdminExecuteMsg::ChangeConfig {
			minimum_vote_proposal_percent: None,
			minimum_vote_turnout_percent: None,
			minimum_vote_pass_percent: Some(69),
			max_proposal_expiry_time_seconds: None,
			execution_expiry_time_seconds: None,
		}),
	)
	.unwrap();
	assert_eq!(
		helpers::query_config(&env_deps).unwrap(),
		// Config is what's applied
		CourtAppConfigJsonable {
			allow_new_proposals: true,
			minimum_vote_proposal_percent: 10,
			minimum_vote_turnout_percent: 20,
			minimum_vote_pass_percent: 69,
			max_proposal_expiry_time_seconds: 7200,
			execution_expiry_time_seconds: 69,
			last_config_change_timestamp_ms: env_deps.0.block.time.millis(),
			admin: Addr::unchecked(ADMIN_ACCOUNT)
		}
	);
}

#[test]
pub fn only_when_no_pending_proposals() {
	let mut env_deps = helpers::new_env_and_instantiate(None);

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
	let execute_response = helpers::execute(
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
	);
	assert!(execute_response.is_err_and(|err| { err.to_string().contains("votes must not be tied to any proposals") }));
	env_deps.0.block.time = env_deps.0.block.time.plus_days(69);

	helpers::execute(
		&mut env_deps,
		None,
		CourtExecuteMsg::DeactivateVotes {
			user: Some(Addr::unchecked(SHARES_HOLDER_ACCOUNT_1)),
			limit: None,
		},
	)
	.unwrap();
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
}

#[test]
pub fn admin_change_admin_unfunded_check() {
	let mut env_deps = new_env_and_instantiate(None);

	helpers::assert_unfunded_instruction(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(ADMIN_ACCOUNT),
			funds: vec![],
		}),
		CourtExecuteMsg::Admin(CourtAdminExecuteMsg::ChangeAdmin {
			admin: Addr::unchecked(RANDOM_ACCOUNT_2),
		}),
	);
}
