import { disableFormInputs, enableFormInputs, isElementInViewport, qa } from "@aritz-cracker/browser-utils";
import { CourtProposalAutogen, CourtProposalMsgEvmExecAutogen, CourtProposalMsgMintAutogen, CourtProposalMsgSendCoinAutogen, CourtProposalMsgWasmChAdminAutogen, CourtProposalMsgWasmClAdminAutogen, CourtProposalMsgWasmExecAutogen, CourtProposalMsgWasmUpgradeAutogen, CourtProposalPlaceholderAutogen, CourtProposalVoteDetailsAutogen } from "./_autogen.js";
import { ClientEnv, UIAmount, addUserTokenInfo, bigIntToStringDecimal, getUserTokenInfo, seiUtilEventEmitter } from "@crownfi/sei-utils";
import { getCourtCoordinatorFromChainId, isProposalFinalized } from "@crownfi/court-coordinator-sdk";
import "../timer_text/index.js";
import isUtf8 from "is-utf8";
import { TimerTextElement } from "../timer_text/index.js";
import { alert, errorMsgBox, FullscreenLoadingTask, HTMLProgressStackedElement, HTMLProgressStackedFillElement, msgBoxIfThrow } from "@crownfi/css-gothic-fantasy";
import { WebClientEnv } from "@crownfi/sei-webui-utils";


export class CourtProposalElement extends CourtProposalAutogen {
	isLatest: boolean;
	proposalWasFinalized: boolean;
	constructor() {
		super();
		this.isLatest = false;
		this.proposalWasFinalized = false;
		for (const elem of this.refs.userVoteForm.elements) {
			if (!(elem instanceof HTMLInputElement) || elem.type != "radio") {
				continue;
			}
			// The radio input still emits a click event after the label's been clicked
			// The value has also already been changed before this event is emitted
			elem.addEventListener("click", (_) => {
				const selectedOption = this.refs.userVoteForm.values().vote;
				if (!selectedOption) {
					// This _should_ never happen, but return early just in case
					return;
				}
				const thisProposalId = this.proposalIdAsNumber();
				if (thisProposalId == undefined || this.proposalWasFinalized){
					return;
				}
				msgBoxIfThrow(async () => {
					const task = new FullscreenLoadingTask();
					try{
						task.text = "Connecting to wallet...";
						task.show();
						const client = await WebClientEnv.get();
						const contract = getCourtCoordinatorFromChainId(client.queryClient, client.chainId);
						task.text = "";
						await client.executeContract(contract.buildVoteIx({id: thisProposalId, vote: selectedOption}));
					}finally{
						task.hide();
						this.refresh();
					}
				});
			});
		}
		this.refs.executeButton.addEventListener("click", (ev) => {
			ev.preventDefault();
			const thisProposalId = this.proposalIdAsNumber();
			if (thisProposalId == undefined || this.proposalWasFinalized){
				return;
			}
			msgBoxIfThrow(async () => {
				const task = new FullscreenLoadingTask();
				try{
					task.text = "Connecting to wallet...";
					task.show();
					const client = await WebClientEnv.get();
					const contract = getCourtCoordinatorFromChainId(client.queryClient, client.chainId);
					task.text = "";
					await client.executeContract(contract.buildExecuteProposalIx({id: thisProposalId}));
				}finally{
					task.hide();
					this.refresh();
				}
			});
		});
		this.refs.voterDetailsButton.addEventListener("click", (_) => {
			const proposalId = this.proposalIdAsNumber();
			if (proposalId == undefined) {
				return;
			}
			CourtProposalVoteDetailsElement.showProposalModal(proposalId);
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
	refresh(evenIfFinalized: boolean = false) {
		const thisProposalId = this.proposalIdAsNumber();
		if (thisProposalId == undefined){
			return;
		}
		this.#shouldRefresh = true;
		if (this.#isRefreshing) {
			return;
		}
		this.#isRefreshing = true;
		this.inert = true;
		this.classList.add("lazy-loading");
		(async () => {
			do {
				while (!isElementInViewport(this)) {
					if (!this.isConnected) {
						break;
					}
					// This is probably better as an intersection event but w/e
					await new Promise(resolve => {setTimeout(resolve, 1000 + 2000 * Math.random())});
				}
				const client = await ClientEnv.get();
				const contract = getCourtCoordinatorFromChainId(client.queryClient, client.chainId);
				this.#shouldRefresh = false;
				const backgroundPromise = (async () => {
					if (!this.isLatest) {
						return;
					}
					if ((await contract.queryProposalAmount() - 1) > thisProposalId) {
						const newElement = new CourtProposalElement();
						newElement.isLatest = true; // Even if it isn't, it will soon realize this.
						newElement.proposalId = (thisProposalId + 1) + "";
						this.parentElement?.insertBefore(newElement, this);
						this.isLatest = false;
					}
				})();
				if (this.proposalWasFinalized && !evenIfFinalized) {
					await backgroundPromise;
					continue;
				}
				const [contractConfig, proposalInfo] = await Promise.all([
					contract.queryConfig(),
					contract.queryGetProposal({id: thisProposalId})
				]);
				if (proposalInfo == null) {
					await backgroundPromise;
					continue;
				}
				this.proposalWasFinalized = isProposalFinalized(proposalInfo.status);
				const configIsRelevant = proposalInfo.status != "rejected_or_expired" &&
					proposalInfo.status != "executed";

				const totalOpinionatedVotes = BigInt(proposalInfo.info.votes_for) +
					BigInt(proposalInfo.info.votes_against);
				const totalVotes = totalOpinionatedVotes + BigInt(proposalInfo.info.votes_abstain);

				let votesForBps = BigInt(proposalInfo.info.votes_for) * 10000n / (
					totalVotes
				);
				let votesAbstainBps = BigInt(proposalInfo.info.votes_abstain) * 10000n / (
					totalVotes
				);
				let votesAgainstBps = BigInt(proposalInfo.info.votes_against) * 10000n / (
					totalVotes
				);
				const votesBpsRemainder = 10000n - (votesForBps + votesAbstainBps + votesAgainstBps);
				if (votesAbstainBps > 0n) {
					votesAbstainBps += votesBpsRemainder;
				} else if (votesAgainstBps > 0n) {
					votesAgainstBps += votesBpsRemainder;
				} else {
					votesForBps += votesBpsRemainder;
				}
				const approvalRatingBps = totalOpinionatedVotes == 0n ? 0n : (
					BigInt(proposalInfo.info.votes_for) * 10000n / totalOpinionatedVotes
				);

				// Note: This is only applicable if status is not "rejected_or_expired"
				const minimumTotalVotes = BigInt((await contract.queryTotalSupply()).votes) *
						BigInt(contractConfig.minimum_vote_turnout_percent) / 100n;

				this.refs.votesApproveAmount.innerText = proposalInfo.info.votes_for;
				this.refs.votesApprovePercent.innerText = bigIntToStringDecimal(votesForBps, 2);

				this.refs.votesOpposeAmount.innerText = proposalInfo.info.votes_against;
				this.refs.votesOpposePercent.innerText = bigIntToStringDecimal(votesAgainstBps, 2);

				this.refs.votesAbstainAmount.innerText = proposalInfo.info.votes_abstain;
				this.refs.votesAbstainPercent.innerText = bigIntToStringDecimal(votesAbstainBps, 2);

				const votesTurnoutProgressElem = this.refs.votesTurnoutProgress as HTMLProgressStackedElement;
				votesTurnoutProgressElem.innerHTML = "";
				const approvalRatingProgressElem = this.refs.approvalRatingProgress as HTMLProgressStackedElement;
				approvalRatingProgressElem.innerHTML = "";
				approvalRatingProgressElem.max = 10000; // just use the calculated bps number
				
				this.refs.votesTurnoutAmount.innerText = totalVotes + "";
				this.refs.approvalRatingPercent.innerText = bigIntToStringDecimal(approvalRatingBps, 2);

				// Clear modifiers
				this.refs.voterTurnoutContainer.classList.value = "important-note";
				this.refs.approvalRatingContainer.classList.value = "important-note";
				if (configIsRelevant) {
					this.refs.votesTurnoutNeededAmount.innerText = minimumTotalVotes + "";
					votesTurnoutProgressElem.max = Number(minimumTotalVotes);
					if (totalVotes < minimumTotalVotes) {
						votesTurnoutProgressElem.classList.value = "danger";
						this.refs.voterTurnoutContainer.classList.add("danger");
					} else {
						votesTurnoutProgressElem.classList.value = "success";
						this.refs.voterTurnoutContainer.classList.add("success");
					}
					approvalRatingProgressElem.max = contractConfig.minimum_vote_pass_percent * 100;
					if (Number(approvalRatingBps) >= (contractConfig.minimum_vote_pass_percent * 100)) {
						approvalRatingProgressElem.classList.value = "success";
					} else {
						approvalRatingProgressElem.classList.value = "danger";
					}
					this.refs.approvalRatingPercentNeeded.innerText = contractConfig.minimum_vote_pass_percent + "";
				} else {
					this.refs.votesTurnoutNeededAmount.innerHTML = "";
					votesTurnoutProgressElem.max = Number(totalVotes);
					votesTurnoutProgressElem.classList.value = "";

					this.refs.approvalRatingPercentNeeded.innerHTML = "";
					approvalRatingProgressElem.classList.value = "";
				}
				
				const votesTurnoutForElem = new HTMLProgressStackedFillElement();
				votesTurnoutForElem.value = Number(proposalInfo.info.votes_for);
				votesTurnoutForElem.classList.add("success");
				votesTurnoutProgressElem.appendChild(votesTurnoutForElem);
				const votesTurnoutAgaintElem = new HTMLProgressStackedFillElement();
				votesTurnoutAgaintElem.value = Number(proposalInfo.info.votes_against);
				votesTurnoutAgaintElem.classList.add("danger");
				votesTurnoutProgressElem.appendChild(votesTurnoutAgaintElem);
				const votesTurnoutAbstainElem = new HTMLProgressStackedFillElement();
				votesTurnoutAbstainElem.value = Number(proposalInfo.info.votes_abstain);
				votesTurnoutAbstainElem.classList.add("primary");
				votesTurnoutProgressElem.appendChild(votesTurnoutAbstainElem);

				if (totalOpinionatedVotes > 0n) {
					const votesRatingForElem = new HTMLProgressStackedFillElement();
					votesRatingForElem.value = Number(approvalRatingBps);
					votesRatingForElem.classList.add("success");
					approvalRatingProgressElem.appendChild(votesRatingForElem);
					const votesRatingAgaintElem = new HTMLProgressStackedFillElement();
					votesRatingAgaintElem.value = 10000 - votesRatingForElem.value;
					votesRatingAgaintElem.classList.add("danger");
					approvalRatingProgressElem.appendChild(votesRatingAgaintElem);
				}

				this.refs.status.classList.value = "important-note"; // Clear modifier classes
				switch (proposalInfo.status) {
					case "rejected_or_expired":
						this.refs.status.innerText = "Proposal rejected/expired.";
						this.refs.status.classList.add("danger");
						break;
					case "executed":
						this.refs.status.innerText = "Proposal passed and executed.";
						this.refs.status.classList.add("success");
						break;
					case "execution_expired":
						this.refs.status.innerText = "Proposal passed but expired before it could be executed.";
						this.refs.status.classList.add("warning");
						break;
					case "passed":
						this.refs.status.innerHTML = `Proposal passed but has not been executed yet. \
							Expires in <span is="timer-text" end-timestamp="${
								proposalInfo.info.expiry_timestamp_ms + contractConfig.execution_expiry_time_seconds * 1000
							}"></span>`;
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

				this.refs.msgList.innerHTML = "";

				for (let i = 0; i < proposalInfo.messages.length; i += 1) {
					const proposalMsg = proposalInfo.messages[i];
					if ("send_coin" in proposalMsg) {
						const {send_coin: {amount, denom, to}} = proposalMsg;
						const newElem = new CourtProposalMsgSendCoinElement();
						await addUserTokenInfo(client.queryClient, client.chainId, denom);
						newElem.coinAmount = amount;
						newElem.denom = denom;
						newElem.recipient = to;
						this.refs.msgList.appendChild(newElem);
					} else if ("execute_evm_contract" in proposalMsg) {
						const {execute_evm_contract: {contract, value, msg}} = proposalMsg;
						const newElem = new CourtProposalMsgEvmExecElement();
						newElem.contract = contract;
						newElem.aseiAmount = value;
						newElem.payload = "0x" + Buffer.from(msg, "base64").toString("hex");
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

				
				if (client.hasAccount()) {
					const [userVoteInfo, userStats] = await Promise.all([
						contract.queryUserVoteInfo({
							proposal_id: thisProposalId,
							user: client.account.seiAddress
						}),
						contract.queryUserStats({
							user: client.account.seiAddress
						})
					]);
					this.refs.userProposalVotes.innerText = userVoteInfo.active_votes;
					if (userVoteInfo.active_votes == "0") {
						this.refs.userVoteForm.elements.vote.value = "";
						this.refs.userVoteForm.elements.vote.forEach((elem) => {
							(elem as HTMLInputElement).checked = false;
						});
					} else {
						this.refs.userVoteForm.elements.vote.value = userVoteInfo.vote;
					}
					this.refs.userTotalVotes.innerText = userStats.staked_votes;
					if (proposalInfo.status == "pending") {
						enableFormInputs(this.refs.userVoteForm);
					} else {
						disableFormInputs(this.refs.userVoteForm);
					}
				} else {
					this.refs.userVoteForm.elements.vote.value = "";
					this.refs.userVoteForm.elements.vote.forEach((elem) => {
						(elem as HTMLInputElement).checked = false;
					});
					this.refs.userProposalVotes.innerHTML = "";
					this.refs.userTotalVotes.innerHTML = "";
					disableFormInputs(this.refs.userVoteForm);
				}
				this.refs.executeButton.disabled = proposalInfo.status != "passed";
				const timerElem = this.querySelector("span[is=\"timer-text\"]") as TimerTextElement | null;
				if (timerElem != null) {
					// If a timer exists, then the status of this proposal will change when the timer elapses.
					timerElem.addTimerCallback(this.refresh.bind(this));
				}
				await backgroundPromise;
			}while(this.#shouldRefresh);
			this.classList.remove("lazy-loading");
			this.inert = false;
		})().catch(ex => {
			this.classList.add("danger");
			errorMsgBox(ex);
		}).finally(() => {
			this.#isRefreshing = false;
		})
	}
	connectedCallback() {
		this.classList.add("framed-box-small");
		this.refresh();
	}
	protected onProposalIdChanged(oldValue: string | null, newValue: string | null) {
		if (oldValue != null) {
			// for whatever reason the proposal ID changed, so we gotta make sure everything is refreshed
			this.proposalWasFinalized = false;
		}
		this.refs.proposalId.innerText = newValue + "";
		this.refresh();
	}
}
CourtProposalElement.registerElement();
seiUtilEventEmitter.on("defaultNetworkChanged", (ev) => {
	(qa("[is=\"court-proposal\"]") as NodeListOf<CourtProposalElement>).forEach(elem => {
		elem.refresh(true);
	});
});
seiUtilEventEmitter.on("defaultProviderChanged", (ev) => {
	(qa("[is=\"court-proposal\"]") as NodeListOf<CourtProposalElement>).forEach(elem => {
		elem.refresh(true);
	});
});

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
			const contract = getCourtCoordinatorFromChainId(client.queryClient, client.chainId);
			do {
				let isLatest = false;
				if (this.lastProposalCreated == undefined) {
					this.lastProposalCreated = await contract.queryProposalAmount();
					isLatest = true;
				}
				this.lastProposalCreated -= 1;
				// we could have just deployed the contract
				if (this.lastProposalCreated >= 0) {
					const newElement = new CourtProposalElement();
					newElement.isLatest = isLatest;
					newElement.proposalId = this.lastProposalCreated + "";
					this.parentElement?.insertBefore(newElement, this);
				}
				if (this.lastProposalCreated < 0) {
					this.viewportObserver.disconnect();
					this.remove();
					break;
				}
				await new Promise(resolve => {setTimeout(resolve, 100)});
			}while(isElementInViewport(this))
		})().catch(ex => {
			this.classList.add("danger");
			errorMsgBox(ex);
			console.error(ex);
		}).finally(() => {
			this.#isAddingNewStuff = false;
		})
	}

	connectedCallback() {
		this.classList.add("framed-box-small");
		this.classList.add("lazy-loading");
		this.viewportObserver.observe(this);
		this.addNewStuff();
	}
}
CourtProposalPlaceholderElement.registerElement();

export class CourtProposalMsgSendCoinElement extends CourtProposalMsgSendCoinAutogen {
	protected refreshDenom() {
		this.refs.coins.innerText = UIAmount(this.coinAmount + "", this.denom + "", true);
	}
	protected onDenomChanged(_: string | null, __: string | null) {
		this.refreshDenom();
	}
	protected onAmountChanged(_: string | null, __: string | null) {
		this.refreshDenom();
	}
	protected onRecipientChanged(_: string | null, newValue: string | null) {
		this.refs.recipient.innerText = newValue + "";
	}
}
CourtProposalMsgSendCoinElement.registerElement();

export class CourtProposalMsgEvmExecElement extends CourtProposalMsgEvmExecAutogen {
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
	protected onAseiAmountChanged(_: string | null, newValue: string | null) {
		const payloadListItem = this.#getPayloadListItem();
		while (this.refs.list.childElementCount > (payloadListItem == null ? 0 : 1)) {
			this.refs.list.firstElementChild!.remove();
		}
		if (!newValue) {
			return;
		}
		const newElem = document.createElement("li");
		newElem.innerText = bigIntToStringDecimal(BigInt(newValue), 18, true) + " SEI";
		this.refs.list.insertBefore(newElem, payloadListItem);
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
CourtProposalMsgEvmExecElement.registerElement();

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

export class CourtProposalVoteDetailsElement extends CourtProposalVoteDetailsAutogen {
	static showProposalModal(proposalId: number) {
		const elem = new CourtProposalVoteDetailsElement();
		elem.proposalId = proposalId + "";
		document.body.appendChild(elem);
		elem.addEventListener("close", _ => {
			elem.remove();
		});
		elem.showModal();
	}
	constructor() {
		super();
		this.refs.closeButton.addEventListener("click", (ev) => {
			ev.preventDefault();
			this.close();
		});
	}
	protected onProposalIdChanged(_: string | null, newValue: string | null): void {
		this.refs.proposalId.innerText = newValue + "";
		this.refreshList();
	}
	#isRefreshing: boolean = false;
	#after: string | null = null;
	refreshList() {
		const proposalId = Number(this.proposalId);
		if (isNaN(proposalId)) {
			return;
		}
		this.refs.voteList.innerHTML = "";
		if (this.#isRefreshing) {
			return;
		}
		const placeholderListItem = document.createElement("li");
		placeholderListItem.classList.add("loading-spinner-inline");
		this.refs.voteList.appendChild(placeholderListItem);
		msgBoxIfThrow(async () => {
			try {
				const client = await ClientEnv.get();
				const contract = getCourtCoordinatorFromChainId(client.queryClient, client.chainId);
				const proposer = (await contract.queryGetProposal({id: proposalId}))?.info.proposer;
				if (!proposer) {
					this.close();
					return;
				}
				let userVotes = [];
				do {
					userVotes = await contract.queryGetProposalUserVotes({
						after: this.#after,
						descending: false,
						limit: 5,
						proposal_id: proposalId
					});
					if (this.refs.voteList.childElementCount == 0) {
						// refreshList() was called so we have to start again.
						// We can't just "continue" here since userVotes may be empty, leaving the list blank
						break;
					}
					for (const userVote of userVotes) {
						const listItem = document.createElement("li");
						const listItemInner = document.createElement("span");
						listItem.appendChild(listItemInner);
						listItemInner.innerText = userVote.user + " - " + userVote.info.active_votes + " ";
						const listItemIcon = document.createElement("span");
						listItemIcon.classList.value = "cicon cicon-size-small";
						switch (userVote.info.vote) {
							case "abstain":
								listItemIcon.classList.add("primary");
								listItemIcon.classList.add("cicon-minus");
								break;
							case "approve":
								listItemIcon.classList.add("success");
								listItemIcon.classList.add("cicon-smile");
								break;
							case "oppose":
								listItemIcon.classList.add("danger");
								listItemIcon.classList.add("cicon-close");
								break;
							default:
								break;
						}
						listItemInner.appendChild(listItemIcon);
						if (userVote.user == proposer) {
							const proposerIcon = document.createElement("span");
							proposerIcon.classList.value = "cicon cicon-size-small cicon-gradient primary cicon-edit";
							listItemInner.appendChild(proposerIcon);
							this.refs.voteList.prepend(listItem);
						} else {
							this.refs.voteList.insertBefore(listItem, placeholderListItem);
						}
						//this.refs.voteList.prepend
						
						this.#after = userVote.user;
					}
				}while(userVotes.length);
				placeholderListItem.remove();
			} finally {
				this.#isRefreshing = false;
				if (this.refs.voteList.childElementCount == 0) {
					// refreshList was previously called while it was being populated. We can now try again.
					this.refreshList();
				}
			}
		});
	}
}
CourtProposalVoteDetailsElement.registerElement();
