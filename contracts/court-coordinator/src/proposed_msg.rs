use borsh::{BorshDeserialize, BorshSerialize};
use cosmwasm_schema::schemars::{self, JsonSchema};
use cosmwasm_std::{Addr, Binary, Coin, CosmosMsg, StdError, Uint128, WasmMsg};
use crownfi_cw_common::data_types::{asset::FungibleAssetKindString, canonical_addr::SeiCanonicalAddr};
use sei_cosmwasm::SeiMsg;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize)]
pub struct BorshableCoin {
	pub denom: String,
	pub amount: u128
}
impl From<Coin> for BorshableCoin {
	fn from(value: Coin) -> Self {
		Self {
			denom: value.denom,
			amount: value.amount.into()
		}
	}
}
impl From<BorshableCoin> for Coin {
	fn from(value: BorshableCoin) -> Self {
		Self {
			denom: value.denom,
			amount: value.amount.into()
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize)]
pub enum ProposedCourtMsg {
	/// Sends a coin. Native or contract-driven
	SendCoin {
		to: SeiCanonicalAddr,
		denom: FungibleAssetKindString,
		amount: u128
	},
	ExecuteWasmContract {
		contract: SeiCanonicalAddr,
		msg: Vec<u8>,
		funds: Vec<BorshableCoin>
	},
	UpgradeWasmContract {
		contract: SeiCanonicalAddr,
		new_code_id: u64,
		msg: Vec<u8>,
	},
	ChangeWasmContractAdmin {
		contract: SeiCanonicalAddr,
		new_admin: SeiCanonicalAddr
	},
	ClearWasmContractAdmin {
		contract: SeiCanonicalAddr,
	},
	TokenfactoryMint {
		tokens: BorshableCoin
	}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProposedCourtMsgJsonable {
	/// Sends a coin
	SendCoin {
		to: Addr,
		denom: FungibleAssetKindString,
		amount: Uint128
	},
	ExecuteWasmContract {
		contract: Addr,
		msg: Binary,
		funds: Vec<Coin>
	},
	UpgradeWasmContract {
		contract: Addr,
		new_code_id: u64,
		msg: Binary,
	},
	ChangeWasmContractAdmin {
		contract: Addr,
		new_admin: Addr
	},
	ClearWasmContractAdmin {
		contract: Addr,
	},
	TokenfactoryMint {
		tokens: Coin
	}
}
impl TryFrom<ProposedCourtMsg> for CosmosMsg<SeiMsg> {
	type Error = StdError;

	fn try_from(value: ProposedCourtMsg) -> Result<Self, Self::Error> {
		match value {
			ProposedCourtMsg::SendCoin { to, denom, amount } => {
				Ok(
					denom.into_asset(amount).transfer_to_msg(
						&Addr::try_from(to)?
					)
				)
			},
			ProposedCourtMsg::ExecuteWasmContract { contract, msg, funds } => {
				Ok(
					WasmMsg::Execute {
						contract_addr: Addr::try_from(contract)?.into_string(),
						msg: msg.into(),
						funds: funds.into_iter().map(|v| {v.into()}).collect()
					}.into()
				)
			},
			ProposedCourtMsg::UpgradeWasmContract { contract, new_code_id, msg } => {
				Ok(
					WasmMsg::Migrate {
						contract_addr: Addr::try_from(contract)?.into_string(),
						new_code_id,
						msg: msg.into()
					}.into()
				)
			},
			ProposedCourtMsg::ChangeWasmContractAdmin { contract, new_admin } => {
				Ok(
					WasmMsg::UpdateAdmin {
						contract_addr: Addr::try_from(contract)?.into_string(),
						admin: Addr::try_from(new_admin)?.into_string()
					}.into()
				)
			},
			ProposedCourtMsg::ClearWasmContractAdmin { contract } => {
				Ok(
					WasmMsg::ClearAdmin {
						contract_addr: Addr::try_from(contract)?.into_string()
					}.into()
				)
			},
			ProposedCourtMsg::TokenfactoryMint { tokens } => {
				Ok(
					SeiMsg::MintTokens { amount: tokens.into() }.into()
				)
			},
		}
	}
}
impl TryFrom<ProposedCourtMsg> for ProposedCourtMsgJsonable {
	type Error = StdError;

	fn try_from(value: ProposedCourtMsg) -> Result<Self, Self::Error> {
		Ok(
			match value {
				ProposedCourtMsg::SendCoin { to, denom, amount } => {
					ProposedCourtMsgJsonable::SendCoin {
						to: Addr::try_from(to)?,
						denom,
						amount: amount.into()
					}
				},
				ProposedCourtMsg::ExecuteWasmContract {contract, msg, funds } => {
					ProposedCourtMsgJsonable::ExecuteWasmContract {
						contract: Addr::try_from(contract)?,
						msg: msg.into(),
						funds: funds.into_iter().map(|v| {v.into()}).collect()
					}
				},
				ProposedCourtMsg::UpgradeWasmContract { contract, new_code_id, msg } => {
					ProposedCourtMsgJsonable::UpgradeWasmContract {
						contract: Addr::try_from(contract)?,
						new_code_id,
						msg: msg.into()
					}
				},
				ProposedCourtMsg::ChangeWasmContractAdmin { contract, new_admin } => {
					ProposedCourtMsgJsonable::ChangeWasmContractAdmin {
						contract: Addr::try_from(contract)?,
						new_admin: Addr::try_from(new_admin)?
					}
				},
				ProposedCourtMsg::ClearWasmContractAdmin { contract } => {
					ProposedCourtMsgJsonable::ClearWasmContractAdmin {
						contract: Addr::try_from(contract)?
					}
				},
				ProposedCourtMsg::TokenfactoryMint { tokens } => {
					ProposedCourtMsgJsonable::TokenfactoryMint { tokens: tokens.into() }
				},
			}
		)
	}
}
impl TryFrom<ProposedCourtMsgJsonable> for ProposedCourtMsg {
	type Error = StdError;

	fn try_from(value: ProposedCourtMsgJsonable) -> Result<Self, Self::Error> {
		Ok(
			match value {
				ProposedCourtMsgJsonable::SendCoin { to, denom, amount } => {
					ProposedCourtMsg::SendCoin {
						to: to.try_into()?,
						denom,
						amount: amount.into()
					}
				},
				ProposedCourtMsgJsonable::ExecuteWasmContract { contract, msg, funds } => {
					ProposedCourtMsg::ExecuteWasmContract {
						contract: contract.try_into()?,
						msg: msg.0,
						funds: funds.into_iter().map(|v| {v.into()}).collect()
					}
				},
				ProposedCourtMsgJsonable::UpgradeWasmContract { contract, new_code_id, msg } => {
					ProposedCourtMsg::UpgradeWasmContract {
						contract: contract.try_into()?,
						new_code_id,
						msg: msg.0
					}
				},
				ProposedCourtMsgJsonable::ChangeWasmContractAdmin { contract, new_admin } => {
					ProposedCourtMsg::ChangeWasmContractAdmin {
						contract: contract.try_into()?,
						new_admin: new_admin.try_into()?
					}
				},
				ProposedCourtMsgJsonable::ClearWasmContractAdmin { contract } => {
					ProposedCourtMsg::ClearWasmContractAdmin {
						contract: contract.try_into()?
					}
				},
				ProposedCourtMsgJsonable::TokenfactoryMint { tokens } => {
					ProposedCourtMsg::TokenfactoryMint { tokens: tokens.into() }
				},
			}
		)
	}
}
