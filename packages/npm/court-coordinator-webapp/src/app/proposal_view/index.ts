import { isElementInViewport } from "@aritz-cracker/browser-utils";
import { CourtProposalAutogen, CourtProposalMsgMintAutogen, CourtProposalMsgSendCoinAutogen, CourtProposalMsgWasmChAdminAutogen, CourtProposalMsgWasmClAdminAutogen, CourtProposalMsgWasmExecAutogen, CourtProposalMsgWasmUpgradeAutogen, CourtProposalPlaceholderAutogen } from "./_autogen.js";
import { ClientEnv, UIAmount, bigIntToStringDecimal, getUserTokenInfo } from "@crownfi/sei-utils";
import { getCourtCoordinatorFromChainId, isProposalFinalized } from "@crownfi/court-coordinator-sdk";
import "../timer_text/index.js";
import isUtf8 from "is-utf8";
import { TimerTextElement } from "../timer_text/index.js";
import { errorDialogIfRejected } from "../dialogs/error.js";
import { setLoading } from "../loading.js";
import { ClientEnvWithModals } from "../fullscreen_client_env.js";

export class CourtProposalElement extends CourtProposalAutogen {
	isLatest: boolean;
	proposalWasFinalized: boolean;
	constructor() {
		super();
		this.isLatest = false;
		this.proposalWasFinalized = false;
		this.refs.voteAgainstButton.addEventListener("click", (ev) => {
			ev.preventDefault();
			const thisProposalId = this.proposalIdAsNumber();
			if (thisProposalId == undefined || this.proposalWasFinalized){
				return;
			}
			errorDialogIfRejected(async () => {
				try{
					setLoading(true, "Connecting to wallet...");
					const client = await ClientEnvWithModals.get();
					const contract = await getCourtCoordinatorFromChainId(client.wasmClient, client.chainId);
					await client.executeContract(contract.buildExecuteProposalIx({id: thisProposalId}));
				}finally{
					setLoading(false);
				}
			});
		});
		this.refs.voteForButton.addEventListener("click", (ev) => {
			ev.preventDefault();
			const thisProposalId = this.proposalIdAsNumber();
			if (thisProposalId == undefined || this.proposalWasFinalized){
				return;
			}
			errorDialogIfRejected(async () => {
				try{
					setLoading(true, "Connecting to wallet...");
					const client = await ClientEnvWithModals.get();
					const contract = await getCourtCoordinatorFromChainId(client.wasmClient, client.chainId);
					await client.executeContract(contract.buildVoteIx({id: thisProposalId, approval: true}));
				}finally{
					setLoading(false);
				}
			});
		});
		this.refs.voteAgainstButton.addEventListener("click", (ev) => {
			ev.preventDefault();
			const thisProposalId = this.proposalIdAsNumber();
			if (thisProposalId == undefined || this.proposalWasFinalized){
				return;
			}
			errorDialogIfRejected(async () => {
				try{
					setLoading(true, "Connecting to wallet...");
					const client = await ClientEnvWithModals.get();
					const contract = await getCourtCoordinatorFromChainId(client.wasmClient, client.chainId);
					await client.executeContract(contract.buildVoteIx({id: thisProposalId, approval: true}));
				}finally{
					setLoading(false);
				}
			});
		});
	}
	proposalIdAsNumber(): number | undefined {
		if (this.proposalId == null) {
			return undefined;
		}
		return Number(this.proposalId);
	}

	#isRefreshing: boolean = false;
	#shouldRefresh: boolean = false;
	refresh() {
		const thisProposalId = this.proposalIdAsNumber();
		if (thisProposalId == undefined || this.proposalWasFinalized){
			return;
		}
		this.#shouldRefresh = true;
		if (this.#isRefreshing) {
			return;
		}
		this.#isRefreshing = true;
		this.inert = true;
		this.classList.add("lazy-loading-covered");
		(async () => {
			const client = await ClientEnv.get();
			const contract = await getCourtCoordinatorFromChainId(client.wasmClient, client.chainId);
			do {
				while (!isElementInViewport(this)) {
					// This is probably better as an intersection event but w/e
					await new Promise(resolve => {setTimeout(resolve, 1000 + 2000 * Math.random())});
				}
				this.#shouldRefresh = false;
				const backgroundPromise = (async () => {
					if (!this.isLatest) {
						return;
					}
					if ((await contract.queryProposalAmount() - 1) > thisProposalId) {
						const newElement = new CourtProposalElement();
						newElement.isLatest = true; // Even if it isn't, it will soon realize this.
						newElement.proposalId = (thisProposalId + 1) + "";
						this.parentElement?.insertBefore(this, newElement);
						this.isLatest = false;
					}
				})();
				const [contractConfig, proposalInfo] = await Promise.all([
					contract.queryConfig(),
					contract.queryGetProposal({id: thisProposalId})
				]);
				if (proposalInfo == null) {
					continue;
				}
				this.proposalWasFinalized = isProposalFinalized(proposalInfo.status);
				this.refs.status.classList.value = "important-note"; // Clear modifier classes
				switch (proposalInfo.status) {
					case "cancelled":
						this.refs.status.innerText = "Proposal cancelled.";
						this.refs.status.classList.add("danger");
						break;
					case "executed":
						this.refs.status.innerText = "Proposal passed and executed.";
						this.refs.status.classList.add("success");
						break;
					case "execution_expired":
						this.refs.status.innerText = "Proposal passed but expired before it could be executed. " +
							"Finalizing this proposal will result in it being marked as cancelled.";
						this.refs.status.classList.add("warning");
						break;
					case "passed":
						this.refs.status.innerHTML = `Proposal passed but has not been executed yet. \
							Expires in <span is="timer-text" end-timestamp="${
								proposalInfo.info.expiry_timestamp_ms + contractConfig.execution_expiry_time_seconds * 1000
							}"></span>`;
						this.refs.status.classList.add("info");
						break;
					case "pending":
						this.refs.status.innerHTML = `Voting period ends in <span is="timer-text" end-timestamp="${
							proposalInfo.info.expiry_timestamp_ms
						}"></span>`;
						break;
					case "rejected":
						this.refs.status.innerText = "Proposal rejected.";
						this.refs.status.classList.add("danger");
						break;
					default:
						this.refs.status.innerText = "Unknown status";
				}
				this.refs.proposer.innerText = proposalInfo.info.proposer;
				this.refs.msgList.innerHTML = "";
				for (let i = 0; i < proposalInfo.messages.length; i += 1) {
					const proposalMsg = proposalInfo.messages[i];
					if ("send_coin" in proposalMsg) {
						const {send_coin: {amount, denom, to}} = proposalMsg;
						const newElem = new CourtProposalMsgSendCoinElement();
						newElem.amount = amount;
						newElem.denom = denom;
						newElem.recipient = to;
						this.refs.msgList.appendChild(newElem);
					} else if ("execute_wasm_contract" in proposalMsg) {
						const {execute_wasm_contract: {contract, funds, msg}} = proposalMsg;
						const newElem = new CourtProposalMsgWasmExecElement();
						newElem.contract = contract;
						newElem.funds = funds.map(v => UIAmount(v.amount, v.denom, true)).join(",");
						
						const decodedMsg = Buffer.from(msg, "base64");
						if (isUtf8(decodedMsg)) {
							try{
								newElem.payload = JSON.stringify(
									JSON.parse(decodedMsg.toString("utf8")),
									undefined,
									"    "
								);
							}catch(ex: any) {
								newElem.payload = decodedMsg.toString("utf8");
							}
						}else{
							newElem.payload = msg;
						}
						this.refs.msgList.appendChild(newElem);
					} else if ("upgrade_wasm_contract" in proposalMsg) {
						const {upgrade_wasm_contract: {contract, msg, new_code_id}} = proposalMsg;
						const newElem = new CourtProposalMsgWasmUpgradeElement();
						newElem.contract = contract;
						
						const decodedMsg = Buffer.from(msg, "base64");
						if (isUtf8(decodedMsg)) {
							try{
								newElem.payload = JSON.stringify(
									JSON.parse(decodedMsg.toString("utf8")),
									undefined,
									"    "
								);
							}catch(ex: any) {
								newElem.payload = decodedMsg.toString("utf8");
							}
						}else{
							newElem.payload = msg;
						}
						newElem.codeId = new_code_id + "";
						this.refs.msgList.appendChild(newElem);
					} else if ("change_wasm_contract_admin" in proposalMsg) {
						const {change_wasm_contract_admin: {contract, new_admin}} = proposalMsg;
						const newElem = new CourtProposalMsgWasmChAdminElement();
						newElem.contract = contract;
						newElem.admin = new_admin;
						this.refs.msgList.appendChild(newElem);
					} else if ("clear_wasm_contract_admin" in proposalMsg) {
						const {clear_wasm_contract_admin: {contract}} = proposalMsg;
						const newElem = new CourtProposalMsgWasmClAdminElement();
						newElem.contract = contract;
						this.refs.msgList.appendChild(newElem);
					} else if ("tokenfactory_mint" in proposalMsg) {
						const {tokenfactory_mint: {tokens: {amount, denom}}} = proposalMsg;
						const newElem = new CourtProposalMsgMintElement();
						newElem.amount = amount;
						newElem.denom = denom;
						this.refs.msgList.appendChild(newElem);
					} else {
						const newElem = document.createElement("li");
						newElem.innerText = JSON.stringify(proposalMsg);
						this.refs.msgList.appendChild(newElem);
					}
				}
				const votePermyriadFor = BigInt(proposalInfo.info.votes_for) * 10000n / (
					BigInt(proposalInfo.info.votes_for) + BigInt(proposalInfo.info.votes_against)
				);
				const votePercentForStr = bigIntToStringDecimal(votePermyriadFor, 2) + "%";
				const votePercentAgainstStr = bigIntToStringDecimal(10000n - votePermyriadFor, 2) + "%";

				// This works pretty well as long as the parent element is a flexbox. Otherwise we'd have to account
				// for border with, etc. ourselves.
				this.refs.votesFor.style.width = votePercentForStr;
				this.refs.votesAgainst.style.width = votePercentAgainstStr;

				this.refs.votesFor.innerText = proposalInfo.info.votes_for + " (" + votePercentForStr + ")";
				this.refs.votesAgainst.innerText = "(" + votePercentAgainstStr + ") " + proposalInfo.info.votes_against;
				
				// Reset the state of the user vote section
				this.refs.votesUser.innerText = "You didn't vote on this";
				this.refs.votesUser.classList.value = "important-note";

				if (client.hasAccount()) {
					const userVotes = await contract.queryUserVoteInfo({
						proposal_id: thisProposalId,
						user: client.account.address
					});
					if (userVotes.active_votes != "0") {
						if (userVotes.voted_for) {
							this.refs.votesUser.innerText = `You voted for this proposal with ${userVotes.active_votes} shares.`;
							this.refs.votesUser.classList.add("success");
						}else{
							this.refs.votesUser.classList.add("danger");
						}
					}
				}
				this.refs.voteForButton.disabled = proposalInfo.info.expiry_timestamp_ms >= Date.now();
				this.refs.voteAgainstButton.disabled = proposalInfo.info.expiry_timestamp_ms >= Date.now();
				this.refs.finalizeButton.disabled = proposalInfo.status != "passed" && proposalInfo.status != "execution_expired";

				const timerElem = this.querySelector("span[is=\"timer-text\"]") as TimerTextElement | null;
				if (timerElem != null) {
					// If a timer exists, then the status of this proposal will change when the timer elapses.
					timerElem.addTimerCallback(this.refresh.bind(this));
				}
				await backgroundPromise;
			}while(this.#shouldRefresh);
		})().catch(ex => {
			console.error(ex);
		}).finally(() => {
			this.inert = false;
			this.classList.remove("lazy-loading-covered");
			this.#isRefreshing = false;
		})
	}
	connectedCallback() {
		this.refresh();
	}
	protected onProposalIdChanged(oldValue: string | null, newValue: string | null) {
		if (oldValue != null) {
			// for whatever reason the proposal ID changed, so we gotta make sure everything is refreshed
			this.proposalWasFinalized = false;
		}
		this.refs.proposalId.innerText = newValue + "";
	}
}
CourtProposalElement.registerElement();

export class CourtProposalPlaceholderElement extends CourtProposalPlaceholderAutogen {
	viewportObserver: IntersectionObserver;
	lastProposalCreated: number | undefined;
	constructor() {
		super();
		this.viewportObserver = new IntersectionObserver((entries) => {
			if (entries[0].intersectionRatio > 0) {
				this.addNewStuff();
			}
		});
	}

	#isAddingNewStuff: boolean = false;
	addNewStuff() {
		if (this.#isAddingNewStuff || this.parentElement == null || !isElementInViewport(this)) {
			return;
		}
		this.#isAddingNewStuff = true;
		(async () => {
			const client = await ClientEnv.get();
			const contract = await getCourtCoordinatorFromChainId(client.wasmClient, client.chainId);
			do {
				let isLatest = false;
				if (this.lastProposalCreated == undefined) {
					this.lastProposalCreated = await contract.queryProposalAmount();
					isLatest = true;
				}
				this.lastProposalCreated -= 1;
				// we could have just deployed the contract
				if (this.lastProposalCreated > 0) {
					const newElement = new CourtProposalElement();
					newElement.isLatest = isLatest;
					newElement.proposalId = this.lastProposalCreated + "";
					this.parentElement?.insertBefore(this, newElement);
				}
				if (this.lastProposalCreated <= 0) {
					this.viewportObserver.disconnect();
					this.remove();
					break;
				}
				await new Promise(resolve => {setTimeout(resolve, 100)});
			}while(isElementInViewport(this))
		})().catch(ex => {
			console.error(ex);
		}).finally(() => {
			this.#isAddingNewStuff = false;
		})
	}

	connectedCallback() {
		this.viewportObserver.observe(this);
		this.addNewStuff();
	}
}
CourtProposalPlaceholderElement.registerElement();

export class CourtProposalMsgSendCoinElement extends CourtProposalMsgSendCoinAutogen {
	protected refreshDenom() {
		this.refs.coins.innerText = UIAmount(this.amount + "", this.denom + "", true);
	}
	protected onDenomChanged(_: string | null, __: string | null) {
		this.refreshDenom();
	}
	protected onAmountChanged(_: string | null, __: string | null) {
		this.refreshDenom();
	}
	protected onRecipientChanged(_: string | null, newValue: string | null) {
		this.refs.recipient.innerText = newValue = "";
	}
}
CourtProposalMsgSendCoinElement.registerElement();

export class CourtProposalMsgWasmExecElement extends CourtProposalMsgWasmExecAutogen {
	protected onContractChanged(_: string | null, newValue: string | null) {
		this.refs.contract.innerText = newValue + "";
	}
	#getPayloadListItem() {
		const maybeCodeElement = this.refs.list.lastElementChild?.firstElementChild;
		if (maybeCodeElement != null && maybeCodeElement.tagName == "PRE") {
			return this.refs.list.lastElementChild as HTMLLIElement;
		}
		return null;
	}
	protected onFundsChanged(_: string | null, newValue: string | null) {
		const payloadListItem = this.#getPayloadListItem();
		while (this.refs.list.childElementCount > (payloadListItem == null ? 0 : 1)) {
			this.refs.list.firstElementChild!.remove();
		}
		if (!newValue) {
			return;
		}
		newValue.split(",").map(v => v.trim()).forEach(coinAmount => {
			const newElem = document.createElement("li");
			newElem.innerText = coinAmount;
			this.refs.list.insertBefore(newElem, payloadListItem);
		});
	}
	protected onPayloadChanged(_: string | null, newValue: string | null) {
		let payloadListItem = this.#getPayloadListItem();
		if (!newValue && payloadListItem != null) {
			payloadListItem.remove();
		}
		if (payloadListItem == null) {
			payloadListItem = document.createElement("li");
			payloadListItem.appendChild(document.createElement("pre"));
			this.appendChild(payloadListItem);
		}
		(payloadListItem.firstElementChild as HTMLPreElement).innerText = newValue + "";
	}
}
CourtProposalMsgWasmExecElement.registerElement();

export class CourtProposalMsgWasmUpgradeElement extends CourtProposalMsgWasmUpgradeAutogen {
	protected onContractChanged(_: string | null, newValue: string | null) {
		this.refs.contract.innerText = newValue + "";
	}
	protected onCodeIdChanged(_: string | null, newValue: string | null) {
		this.refs.codeId.innerText = newValue + "";
	}
	protected onPayloadChanged(_: string | null, newValue: string | null): void {
		this.refs.payload.innerText = newValue + "";
	}
}
CourtProposalMsgWasmUpgradeElement.registerElement();

export class CourtProposalMsgWasmChAdminElement extends CourtProposalMsgWasmChAdminAutogen {
	protected onContractChanged(_: string | null, newValue: string | null) {
		this.refs.contract.innerText = newValue + "";
	}
	protected onAdminChanged(_: string | null, newValue: string | null) {
		this.refs.admin.innerText = newValue + "";
	}
}
CourtProposalMsgWasmChAdminElement.registerElement();

export class CourtProposalMsgWasmClAdminElement extends CourtProposalMsgWasmClAdminAutogen {
	protected onContractChanged(_: string | null, newValue: string | null) {
		this.refs.contract.innerText = newValue + "";
	}
}
CourtProposalMsgWasmClAdminElement.registerElement();

export class CourtProposalMsgMintElement extends CourtProposalMsgMintAutogen {
	protected refreshDenom() {
		this.refs.coins.innerText = UIAmount(this.amount + "", this.denom + "", true);
	}
	protected onDenomChanged(_: string | null, __: string | null) {
		this.refreshDenom();
	}
	protected onAmountChanged(_: string | null, __: string | null) {
		this.refreshDenom();
	}
}
CourtProposalMsgMintElement.registerElement();
