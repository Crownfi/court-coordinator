use super::*;
pub fn new_env_and_instantiate(msg: Option<CourtInstantiateMsg>) -> MutexGuard<'static, (Env, SeiMockEnvDeps)> {
	let mut env_deps = new_global_env();
	instantiate(&mut env_deps, None, msg.clone()).unwrap();
	assert_eq!(
		query_config(&env_deps).unwrap(),
		// Config is what's applied
		CourtAppConfigJsonable {
			allow_new_proposals: true,
			minimum_vote_proposal_percent: msg
				.as_ref()
				.map(|msg| { msg.minimum_vote_proposal_percent })
				.unwrap_or(10),
			minimum_vote_turnout_percent: msg
				.as_ref()
				.map(|msg| { msg.minimum_vote_turnout_percent })
				.unwrap_or(20),
			minimum_vote_pass_percent: msg.as_ref().map(|msg| { msg.minimum_vote_pass_percent }).unwrap_or(50),
			max_proposal_expiry_time_seconds: msg
				.as_ref()
				.map(|msg| { msg.max_proposal_expiry_time_seconds })
				.unwrap_or(7200),
			execution_expiry_time_seconds: msg
				.as_ref()
				.map(|msg| { msg.execution_expiry_time_seconds })
				.unwrap_or(3600),
			last_config_change_timestamp_ms: env_deps.0.block.time.millis(),
			admin: Addr::unchecked(ADMIN_ACCOUNT)
		}
	);
	env_deps
}

pub fn instantiate(
	env_deps: &mut (Env, SeiMockEnvDeps),
	msg_info: Option<MessageInfo>,
	msg: Option<CourtInstantiateMsg>,
) -> Result<Response<sei_cosmwasm::SeiMsg>, CourtContractError> {
	let msg_info = msg_info.unwrap_or(MessageInfo {
		sender: Addr::unchecked(ADMIN_ACCOUNT),
		funds: vec![],
	});
	let msg = msg.unwrap_or(CourtInstantiateMsg {
		admin: msg_info.sender.clone(),
		shares_mint_amount: 1000000u128.into(),
		shares_mint_receiver: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
		minimum_vote_proposal_percent: 10,
		minimum_vote_turnout_percent: 20,
		minimum_vote_pass_percent: 50,
		max_proposal_expiry_time_seconds: 7200,
		execution_expiry_time_seconds: 3600,
		vote_share_name: "Test Votes".into(),
		vote_share_symbol: "TST".into(),
		vote_share_description: "Test vortessadbjhk,sdfgvgjhlksdfjhgbksdv".into(),
	});
	let env = env_deps.0.clone();
	crate::contract::instantiate(env_deps.1.as_mut(), env, msg_info, msg)
}
pub fn execute(
	env_deps: &mut (Env, SeiMockEnvDeps),
	msg_info: Option<MessageInfo>,
	msg: CourtExecuteMsg,
) -> Result<Response<sei_cosmwasm::SeiMsg>, CourtContractError> {
	let msg_info = msg_info.unwrap_or(MessageInfo {
		sender: Addr::unchecked(RANDOM_ACCOUNT_5),
		funds: vec![],
	});
	let env = env_deps.0.clone();
	crate::contract::execute(env_deps.1.as_mut(), env, msg_info, msg)
}
pub fn query_config(env_deps: &(Env, SeiMockEnvDeps)) -> Result<CourtAppConfigJsonable, CourtContractError> {
	let env = env_deps.0.clone();
	Ok(from_json(crate::contract::query(
		env_deps.1.as_ref().into_empty(),
		env,
		CourtQueryMsg::Config,
	)?)?)
}
pub fn query_denom(env_deps: &(Env, SeiMockEnvDeps)) -> Result<CourtQueryResponseDenom, CourtContractError> {
	let env = env_deps.0.clone();
	Ok(from_json(crate::contract::query(
		env_deps.1.as_ref().into_empty(),
		env,
		CourtQueryMsg::Denom,
	)?)?)
}
pub fn query_proposal_amount(env_deps: &(Env, SeiMockEnvDeps)) -> Result<u32, CourtContractError> {
	let env = env_deps.0.clone();
	Ok(from_json(crate::contract::query(
		env_deps.1.as_ref().into_empty(),
		env,
		CourtQueryMsg::ProposalAmount,
	)?)?)
}
pub fn query_get_proposal(
	env_deps: &(Env, SeiMockEnvDeps),
	id: u32,
) -> Result<Option<CourtQueryResponseTransactionProposal>, CourtContractError> {
	let env = env_deps.0.clone();
	Ok(from_json(crate::contract::query(
		env_deps.1.as_ref().into_empty(),
		env,
		CourtQueryMsg::GetProposal { id },
	)?)?)
}
pub fn query_get_proposals(
	env_deps: &(Env, SeiMockEnvDeps),
	skip: Option<u32>,
	limit: Option<u32>,
	descending: bool,
) -> Result<Vec<CourtQueryResponseTransactionProposal>, CourtContractError> {
	let env = env_deps.0.clone();
	Ok(from_json(crate::contract::query(
		env_deps.1.as_ref().into_empty(),
		env,
		CourtQueryMsg::GetProposals {
			skip,
			limit,
			descending,
		},
	)?)?)
}
pub fn query_user_stats(
	env_deps: &(Env, SeiMockEnvDeps),
	user: &str,
) -> Result<CourtUserStatsJsonable, CourtContractError> {
	let env = env_deps.0.clone();
	Ok(from_json(crate::contract::query(
		env_deps.1.as_ref().into_empty(),
		env,
		CourtQueryMsg::UserStats {
			user: Addr::unchecked(user),
		},
	)?)?)
}
pub fn query_user_vote_info(
	env_deps: &(Env, SeiMockEnvDeps),
	user: &str,
	proposal_id: u32,
) -> Result<CourtUserVoteInfoJsonable, CourtContractError> {
	let env = env_deps.0.clone();
	Ok(from_json(crate::contract::query(
		env_deps.1.as_ref().into_empty(),
		env,
		CourtQueryMsg::UserVoteInfo {
			user: Addr::unchecked(user),
			proposal_id,
		},
	)?)?)
}
pub fn query_get_users_with_active_proposals(
	env_deps: &(Env, SeiMockEnvDeps),
	after: Option<CourtQueryUserWithActiveProposal>,
	limit: Option<u32>,
	descending: bool,
) -> Result<Vec<CourtQueryUserWithActiveProposal>, CourtContractError> {
	let env = env_deps.0.clone();
	Ok(from_json(crate::contract::query(
		env_deps.1.as_ref().into_empty(),
		env,
		CourtQueryMsg::GetUsersWithActiveProposals {
			after,
			limit,
			descending,
		},
	)?)?)
}
pub fn query_get_user_active_proposals(
	env_deps: &(Env, SeiMockEnvDeps),
	user: &str,
	skip: Option<u32>,
	limit: Option<u32>,
	descending: bool,
) -> Result<Vec<u32>, CourtContractError> {
	let env = env_deps.0.clone();
	Ok(from_json(crate::contract::query(
		env_deps.1.as_ref().into_empty(),
		env,
		CourtQueryMsg::GetUserActiveProposals {
			user: Addr::unchecked(user),
			skip,
			limit,
			descending,
		},
	)?)?)
}
pub fn query_get_proposal_user_votes(
	env_deps: &(Env, SeiMockEnvDeps),
	proposal_id: u32,
	after: Option<&str>,
	limit: Option<u32>,
	descending: bool,
) -> Result<Vec<CourtQueryResponseUserVote>, CourtContractError> {
	let env = env_deps.0.clone();
	Ok(from_json(crate::contract::query(
		env_deps.1.as_ref().into_empty(),
		env,
		CourtQueryMsg::GetProposalUserVotes {
			proposal_id,
			after: after.map(|addr| Addr::unchecked(addr)),
			limit,
			descending,
		},
	)?)?)
}

pub fn assert_only_authorized_instruction(
	env_deps: &mut (Env, SeiMockEnvDeps),
	funds: &[Coin],
	unauthorized_users: &[&str],
	authorized_users: &[&str],
	msg: CourtExecuteMsg,
) {
	for user_str in unauthorized_users.iter() {
		let execute_response = execute(
			env_deps,
			Some(MessageInfo {
				sender: Addr::unchecked(user_str.to_string()),
				funds: funds.into(),
			}),
			msg.clone(),
		);
		assert!(execute_response.is_err_and(|err| {
			// Typo won't be fixed until we upgrade to cosmwasm-std 2.x
			err.to_string().starts_with("Permission denied:")
		}));
	}
	for user_str in authorized_users.iter() {
		let execute_response = execute(
			env_deps,
			Some(MessageInfo {
				sender: Addr::unchecked(user_str.to_string()),
				funds: funds.into(),
			}),
			msg.clone(),
		);
		assert!(execute_response.is_ok());
	}
}

pub fn assert_unfunded_instruction(
	env_deps: &mut (Env, SeiMockEnvDeps),
	msg_info: Option<MessageInfo>,
	msg: CourtExecuteMsg,
) {
	let vote_shares_denom = query_denom(env_deps).unwrap().votes;
	let mut msg_info = msg_info.unwrap_or(MessageInfo {
		sender: Addr::unchecked(RANDOM_ACCOUNT_5),
		funds: vec![],
	});
	msg_info.funds.push(coin(31337, "usei"));
	let execute_response = execute(env_deps, Some(msg_info.clone()), msg.clone());
	assert!(execute_response.is_err());
	assert!(execute_response.is_err_and(|err| {
		// Typo won't be fixed until we upgrade to cosmwasm-std 2.x
		err.to_string().contains("does no accept funds") || err.to_string().contains("does not accept funds")
	}));
	msg_info.funds.pop();
	msg_info.funds.push(coin(31337, &vote_shares_denom));
	let execute_response = execute(env_deps, Some(msg_info.clone()), msg.clone());
	assert!(execute_response.is_err());
	assert!(execute_response.is_err_and(|err| {
		// Typo won't be fixed until we upgrade to cosmwasm-std 2.x
		err.to_string().contains("does no accept funds") || err.to_string().contains("does not accept funds")
	}));
	msg_info.funds.push(coin(31337, &vote_shares_denom));
	let execute_response = execute(env_deps, Some(msg_info.clone()), msg.clone());
	assert!(execute_response.is_err());
	assert!(execute_response.is_err_and(|err| {
		// Typo won't be fixed until we upgrade to cosmwasm-std 2.x
		err.to_string().contains("does no accept funds") || err.to_string().contains("does not accept funds")
	}));
}

pub fn assert_must_pay(env_deps: &mut (Env, SeiMockEnvDeps), sender: &str, msg: CourtExecuteMsg, accepted_denom: &str) {
	let random_denom_1 = format!("factory/{}/ayylmao", RANDOM_CONTRACT);
	let random_denom_2 = format!("factory/{}/ayylmao", env_deps.0.contract.address);

	let mut msg_info = MessageInfo {
		sender: Addr::unchecked(sender),
		funds: vec![],
	};

	let execute_response = execute(env_deps, Some(msg_info.clone()), msg.clone());
	assert!(execute_response.is_err());
	assert!(execute_response.is_err_and(|err| { err.to_string().contains("No funds sent") }));

	msg_info.funds.push(coin(0, accepted_denom));
	let execute_response = execute(env_deps, Some(msg_info.clone()), msg.clone());
	assert!(execute_response.is_err());
	assert!(execute_response.is_err_and(|err| { err.to_string().contains("No funds sent") }));

	msg_info.funds.pop();

	msg_info.funds.push(coin(31337u128, &random_denom_1));
	let execute_response = execute(env_deps, Some(msg_info.clone()), msg.clone());
	assert!(execute_response.is_err());
	assert!(execute_response.is_err_and(|err| {
		// Must send reserve token
		err.to_string().contains("Must send") && err.to_string().contains(accepted_denom)
	}));

	msg_info.funds.pop();

	msg_info.funds.push(coin(31337u128, &random_denom_2));
	let execute_response = execute(env_deps, Some(msg_info.clone()), msg.clone());
	assert!(execute_response.is_err());
	assert!(execute_response.is_err_and(|err| {
		// Must send reserve token
		err.to_string().contains("Must send") && err.to_string().contains(accepted_denom)
	}));

	msg_info.funds.push(coin(31337u128, &random_denom_1));
	let execute_response = execute(env_deps, Some(msg_info.clone()), msg.clone());
	assert!(execute_response.is_err());
	assert!(execute_response.is_err_and(|err| {
		// Must send reserve token
		err.to_string().contains("more than one denomination")
	}));

	msg_info.funds.pop();
	msg_info.funds.pop();
	msg_info.funds.push(coin(1337u128, accepted_denom));
	msg_info.funds.push(coin(1337u128, &random_denom_1));

	let execute_response = execute(env_deps, Some(msg_info.clone()), msg.clone());
	assert!(execute_response.is_err());
	assert!(execute_response.is_err_and(|err| {
		// Must send reserve token
		err.to_string().contains("more than one denomination")
	}));

	msg_info.funds.pop();

	let execute_response = execute(env_deps, Some(msg_info.clone()), msg.clone());
	assert!(execute_response.is_ok());
}

// Contract stores the number of tokens it minted here
pub fn get_known_vote_supply(env_deps: &(Env, SeiMockEnvDeps)) -> u128 {
	use cosmwasm_std::Storage;

	let vote_shares_denom = format!("factory/{}/votes", &env_deps.0.contract.address);
	env_deps
		.1
		.storage
		.get(vote_shares_denom.as_bytes())
		.and_then(|bytes| Some(<u128>::from_le_bytes(bytes.try_into().ok()?)))
		.unwrap_or_default()
}

pub fn execute_change_admin(env_deps: &mut (Env, SeiMockEnvDeps), sender: Option<&str>, new_admin: &str) {
	execute(
		env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(sender.unwrap_or(ADMIN_ACCOUNT)),
			funds: vec![],
		}),
		CourtExecuteMsg::Admin(CourtAdminExecuteMsg::ChangeAdmin {
			admin: Addr::unchecked(new_admin),
		}),
	)
	.unwrap();
}

pub fn execute_allow_new_proposals(env_deps: &mut (Env, SeiMockEnvDeps), sender: Option<&str>, allowed: bool) {
	execute(
		env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(sender.unwrap_or(ADMIN_ACCOUNT)),
			funds: vec![],
		}),
		CourtExecuteMsg::Admin(CourtAdminExecuteMsg::AllowNewProposals { allowed }),
	)
	.unwrap();
}

pub fn execute_stake_votes(env_deps: &mut (Env, SeiMockEnvDeps), sender: &str, amount: u128) {
	let vote_shares_denom = query_denom(&env_deps).unwrap().votes;
	let previous_stake_amount = query_user_stats(&env_deps, sender).unwrap().staked_votes.u128();
	execute(
		env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(sender),
			funds: vec![coin(amount, &vote_shares_denom)],
		}),
		CourtExecuteMsg::Stake,
	)
	.unwrap();

	let stake_amount_increase = query_user_stats(&env_deps, sender)
		.unwrap()
		.staked_votes
		.u128()
		.saturating_sub(previous_stake_amount);
	assert_eq!(amount, stake_amount_increase);
}

pub fn execute_deactivate_votes(env_deps: &mut (Env, SeiMockEnvDeps), voter: &str) {
	execute(
		env_deps,
		None,
		CourtExecuteMsg::DeactivateVotes {
			user: Some(Addr::unchecked(voter)),
			limit: None,
		},
	)
	.unwrap();
}

pub fn execute_stake_exact_amount(env_deps: &mut (Env, SeiMockEnvDeps), sender: &str, amount: u128) {
	let vote_shares_denom = query_denom(&env_deps).unwrap().votes;
	let mut previous_stake_amount = query_user_stats(&env_deps, sender).unwrap().staked_votes.u128();
	if previous_stake_amount > amount {
		execute(
			env_deps,
			Some(MessageInfo {
				sender: Addr::unchecked(sender),
				funds: vec![],
			}),
			CourtExecuteMsg::Unstake,
		)
		.unwrap();
		previous_stake_amount = 0;
	}
	let amount_to_stake = amount - previous_stake_amount;
	if amount_to_stake == 0 {
		return;
	}
	execute(
		env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(sender),
			funds: vec![coin(amount_to_stake, &vote_shares_denom)],
		}),
		CourtExecuteMsg::Stake,
	)
	.unwrap();
	assert_eq!(query_user_stats(&env_deps, sender).unwrap().staked_votes.u128(), amount);
}

pub fn execute_propose_transaction(
	env_deps: &mut (Env, SeiMockEnvDeps),
	sender: &str,
	msgs: Vec<ProposedCourtMsgJsonable>,
	expiry_time_seconds: u32,
) {
	let user_staked_votes = query_user_stats(&env_deps, sender).unwrap().staked_votes;
	let new_proposal_id = query_proposal_amount(&env_deps).unwrap();
	let execute_result = execute(
		env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(sender),
			funds: vec![],
		}),
		CourtExecuteMsg::ProposeTransaction {
			msgs,
			expiry_time_seconds,
		},
	)
	.unwrap();

	assert_eq!(execute_result.messages.len(), 0);
	assert_eq!(
		execute_result.events,
		vec![
			cosmwasm_std::Event::new("proposal")
				.add_attribute("proposal_id", new_proposal_id.to_string())
				.add_attribute("proposer", sender),
			cosmwasm_std::Event::new("vote")
				.add_attribute("proposal_id", new_proposal_id.to_string())
				.add_attribute("voter", sender)
				.add_attribute("votes", user_staked_votes)
				.add_attribute("vote", "approve")
		]
	);
}

pub fn execute_vote(env_deps: &mut (Env, SeiMockEnvDeps), sender: &str, id: u32, vote: CourtUserVoteStatus) {
	let user_votes = query_user_stats(&env_deps, sender).unwrap().staked_votes;
	execute(
		env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(sender),
			funds: vec![],
		}),
		CourtExecuteMsg::Vote { id, vote },
	)
	.unwrap();
	assert_eq!(
		query_user_vote_info(&env_deps, sender, id),
		Ok(CourtUserVoteInfoJsonable {
			active_votes: user_votes,
			vote
		})
	);
}

pub fn advance_time_to_vote_end(env_deps: &mut (Env, SeiMockEnvDeps), proposal_id: u32) {
	let proposal = query_get_proposal(&env_deps, proposal_id).unwrap().unwrap();
	let proposal_expiry_timestamp = Timestamp::from_millis(proposal.info.expiry_timestamp_ms);
	if env_deps.0.block.time < proposal_expiry_timestamp {
		env_deps.0.block.time = proposal_expiry_timestamp;
	}
}
pub fn advance_time_to_execution_expiry(env_deps: &mut (Env, SeiMockEnvDeps), proposal_id: u32) {
	let config = query_config(&env_deps).unwrap();
	let proposal = query_get_proposal(&env_deps, proposal_id).unwrap().unwrap();
	let mut proposal_expiry_timestamp = Timestamp::from_millis(proposal.info.expiry_timestamp_ms);
	proposal_expiry_timestamp = proposal_expiry_timestamp
		.plus_seconds(config.execution_expiry_time_seconds as u64)
		.plus_nanos(1000000);
	if env_deps.0.block.time < proposal_expiry_timestamp {
		env_deps.0.block.time = proposal_expiry_timestamp;
	}
}

fn minimum_proposal_vote_amount(env_deps: &(Env, SeiMockEnvDeps)) -> u128 {
	let config = query_config(env_deps).unwrap();
	let total_supply = get_known_vote_supply(env_deps);
	(total_supply * config.minimum_vote_proposal_percent as u128).div_ceil(100)
}

fn minimum_vote_turnout_amount(env_deps: &(Env, SeiMockEnvDeps)) -> u128 {
	let config = query_config(env_deps).unwrap();
	let total_supply = get_known_vote_supply(env_deps);
	(total_supply * config.minimum_vote_turnout_percent as u128).div_ceil(100)
}

/// How many votes would a proposal need to passes (pass percent of minimum turnout)
fn minimum_votes_for_pass(env_deps: &(Env, SeiMockEnvDeps)) -> u128 {
	let config = query_config(env_deps).unwrap();
	let minimum_turnout = minimum_vote_turnout_amount(env_deps);

	(minimum_turnout * config.minimum_vote_pass_percent as u128).div_ceil(100)
}

fn minimum_votes_for_guaranteed_pass(env_deps: &(Env, SeiMockEnvDeps)) -> u128 {
	let config = query_config(env_deps).unwrap();
	let total_supply = get_known_vote_supply(env_deps);

	(total_supply * config.minimum_vote_pass_percent as u128).div_ceil(100)
}

pub fn execute_create_unanimous_proposal(
	env_deps: &mut (Env, SeiMockEnvDeps),
	msgs: Vec<ProposedCourtMsgJsonable>,
	expiry_time_seconds: u32,
	voter_for: &str,
) -> u32 {
	let new_proposal_id = query_proposal_amount(&env_deps).unwrap();
	let pass_votes_required = minimum_proposal_vote_amount(env_deps).max(minimum_vote_turnout_amount(env_deps));
	execute_stake_exact_amount(env_deps, voter_for, pass_votes_required);
	execute_propose_transaction(env_deps, voter_for, msgs, expiry_time_seconds);

	new_proposal_id
}
pub fn execute_create_unanimous_proposal_with_low_turnout(
	env_deps: &mut (Env, SeiMockEnvDeps),
	msgs: Vec<ProposedCourtMsgJsonable>,
	expiry_time_seconds: u32,
	voter_for: &str,
) -> u32 {
	let new_proposal_id = query_proposal_amount(&env_deps).unwrap();
	let minimum_proposal_vote_amount = minimum_proposal_vote_amount(env_deps);
	assert!(
		minimum_proposal_vote_amount < minimum_vote_turnout_amount(env_deps),
		"unanimous proposals with low turnout cannot be tested with current config"
	);
	execute_stake_exact_amount(env_deps, voter_for, minimum_proposal_vote_amount);
	execute_propose_transaction(env_deps, voter_for, msgs, expiry_time_seconds);
	new_proposal_id
}

pub fn execute_create_guaranteed_passing_proposal(
	env_deps: &mut (Env, SeiMockEnvDeps),
	msgs: Vec<ProposedCourtMsgJsonable>,
	expiry_time_seconds: u32,
	voter_for: &str,
) -> u32 {
	let new_proposal_id = query_proposal_amount(&env_deps).unwrap();

	let approve_vote_amount = minimum_votes_for_guaranteed_pass(&env_deps);
	execute_stake_exact_amount(env_deps, voter_for, approve_vote_amount);
	execute_propose_transaction(env_deps, voter_for, msgs, expiry_time_seconds);

	new_proposal_id
}

pub fn execute_create_passing_proposal(
	env_deps: &mut (Env, SeiMockEnvDeps),
	msgs: Vec<ProposedCourtMsgJsonable>,
	expiry_time_seconds: u32,
	voter_for: &str,
	voter_against: &str,
) -> u32 {
	let new_proposal_id = query_proposal_amount(env_deps).unwrap();
	let minimum_turnout = minimum_vote_turnout_amount(env_deps);
	let approve_vote_amount = minimum_votes_for_pass(env_deps);
	let oppose_vote_amount = minimum_turnout - approve_vote_amount;

	execute_stake_exact_amount(env_deps, voter_for, approve_vote_amount);
	execute_stake_exact_amount(env_deps, voter_against, oppose_vote_amount);
	execute_propose_transaction(env_deps, voter_for, msgs, expiry_time_seconds);
	execute_vote(env_deps, voter_against, new_proposal_id, CourtUserVoteStatus::Oppose);

	new_proposal_id
}
/*
pub fn execute_create_passing_proposal_low_turnout(
	env_deps: &mut (Env, SeiMockEnvDeps),
	msgs: Vec<ProposedCourtMsgJsonable>,
	expiry_time_seconds: u32,
	voter_for: &str,
	voter_against: &str,
) -> u32 {
	let new_proposal_id = query_proposal_amount(&env_deps).unwrap();

	new_proposal_id
}
*/

pub fn execute_create_failing_proposal(
	env_deps: &mut (Env, SeiMockEnvDeps),
	msgs: Vec<ProposedCourtMsgJsonable>,
	expiry_time_seconds: u32,
	voter_for: &str,
	voter_against: &str,
) -> u32 {
	let new_proposal_id = query_proposal_amount(&env_deps).unwrap();
	let config = query_config(env_deps).unwrap();

	let approve_vote_amount = minimum_proposal_vote_amount(env_deps);
	let oppose_vote_amount =
		(approve_vote_amount * 100).div_ceil(config.minimum_vote_pass_percent.into()) + 1 - approve_vote_amount;

	assert!(approve_vote_amount + oppose_vote_amount < get_known_vote_supply(env_deps));

	execute_stake_exact_amount(env_deps, voter_for, approve_vote_amount);
	execute_stake_exact_amount(env_deps, voter_against, oppose_vote_amount);
	execute_propose_transaction(env_deps, voter_for, msgs, expiry_time_seconds);
	execute_vote(env_deps, voter_against, new_proposal_id, CourtUserVoteStatus::Oppose);

	new_proposal_id
}
