import { alert, confirm, FullscreenLoadingTask, msgBoxIfThrow } from "@crownfi/css-gothic-fantasy";
import { AdminMintSharesModalAutogen, AdminMintSharesModalFormValues2, CourtConfigAutogen, CourtConfigModalAutogen, CourtConfigModalFormValues1 } from "./_autogen.js";
import { CourtAdminExecuteMsg, getCourtCoordinatorFromChainId } from "@crownfi/court-coordinator-sdk";
import { humanReadableTimeAmount, parseTimeAmount } from "../time_format.js";
import { WebClientEnv } from "@crownfi/sei-webui-utils";
import { seiUtilEventEmitter } from "@crownfi/sei-utils";
import { applyCustomElementsWorkaround, q, qa } from "@aritz-cracker/browser-utils";
await applyCustomElementsWorkaround();

export class CourtConfigElement extends CourtConfigAutogen {
	constructor() {
		super();
		this.refs.adminAbdicateButton.addEventListener("click", (ev) => {
			ev.preventDefault();
			msgBoxIfThrow(async () => {
				if (!(await confirm(
					"Relinquish adminship",
					"Are you sure you wish to relinquish your adminship?\nIf you do this, then a new admin must be " +
					"elected to change the DAO configuration.",
					"question",
					undefined,
					"No",
					"Yes"
				))) {
					return;
				}
				const task = new FullscreenLoadingTask();
				try {
					task.text = "Connecting to wallet...";
					const client = await WebClientEnv.get();
					task.text = "";
					const contract = getCourtCoordinatorFromChainId(client.queryClient, client.chainId);
					await client.executeContract(
						contract.buildAdminIx({
							"change_admin": {
								admin: contract.address
							}
						})
					);
				} finally {
					task.hide();
					this.refresh();
				}
			});
		});
		this.refs.adminConfigButton.addEventListener("click", (ev) => {
			ev.preventDefault();
			CourtConfigModalElement.showModalAndDoTransaction();
		});
		this.refs.adminProposalsButton.addEventListener("change", (_) => {
			msgBoxIfThrow((async() => {
				const task = new FullscreenLoadingTask();
				try {
					this.refs.adminProposalsButton.disabled = true;
					task.text = "Connecting to wallet...";
					const client = await WebClientEnv.get();
					task.text = "";
					const contract = getCourtCoordinatorFromChainId(client.queryClient, client.chainId);
					await client.executeContract(
						contract.buildAdminIx({
							"allow_new_proposals": {
								allowed: this.refs.adminProposalsButton.checked
							}
						})
					);
				} finally {
					task.hide();
					this.refresh();
				}
			}));
		});
		this.refs.adminMintButton.addEventListener("click", (ev) => {
			ev.preventDefault();
			AdminMintSharesModalElement.showModalAndDoTransaction();
		});
	}
	#shouldRefresh: boolean = false;
	#isRefreshing: boolean = false;
	refresh() {
		this.#shouldRefresh = true;
		if (this.#isRefreshing) {
			return;
		}
		this.#isRefreshing = true;
		this.classList.remove("danger");
		this.classList.add("lazy-loading");
		msgBoxIfThrow((async () => {
			try {
				do {
					this.#shouldRefresh = false;
					this.refs.adminAbdicateButton.disabled = true;
					this.refs.adminConfigButton.disabled = true;
					this.refs.adminProposalsButton.disabled = true;
					this.refs.adminMintButton.disabled = true;
					const client = await WebClientEnv.get();
					const contract = getCourtCoordinatorFromChainId(client.queryClient, client.chainId);
					const votesDenom = (await contract.queryDenom()).votes;
					const votesSupply = BigInt((await contract.queryTotalSupply()).votes);
					const config = await contract.queryConfig();

					this.refs.totalVoteTokens.amount = votesSupply + "";
					this.refs.totalVoteTokens.denom = votesDenom;

					if (config.admin == contract.address) {
						this.refs.configAdmin.innerText = "[None, one can be proposed]";
					} else {
						this.refs.configAdmin.innerText = config.admin;
					}
					
					this.refs.configProposalsAllowed.checked = config.allow_new_proposals;
					this.refs.adminProposalsButton.checked = config.allow_new_proposals;
					this.refs.configMinVotesNewPercent.innerText = config.minimum_vote_proposal_percent + "";
					this.refs.configMinVotesNewAmountTokens.amount = (
						votesSupply * BigInt(config.minimum_vote_proposal_percent) / 100n
					) + "";
					this.refs.configMinVotesNewAmountTokens.denom = votesDenom;
					this.refs.configMinTurnoutPercent.innerText = config.minimum_vote_turnout_percent + "";
					this.refs.configMinTurnoutTokens.amount = (
						votesSupply * BigInt(config.minimum_vote_turnout_percent) / 100n
					) + "";
					this.refs.configMinTurnoutTokens.denom = votesDenom;
					this.refs.configMinApprovalPercent.innerText = config.minimum_vote_pass_percent + "";
					this.refs.configVotingTime.innerText = humanReadableTimeAmount(config.max_proposal_expiry_time_seconds * 1000);
					this.refs.configExecutionWindow.innerText = humanReadableTimeAmount(config.execution_expiry_time_seconds * 1000);
					this.refs.configChangeTime.innerText = (new Date(config.last_config_change_timestamp_ms)).toLocaleString();

					if (client.account?.seiAddress == config.admin) {
						this.refs.adminNote.hidden = true;
						this.refs.adminAbdicateButton.disabled = false;
						this.refs.adminConfigButton.disabled = false;
						this.refs.adminProposalsButton.disabled = false;
						this.refs.adminMintButton.disabled = false;
					} else {
						this.refs.adminNote.hidden = false;
					}
				}while(this.#shouldRefresh);
				this.classList.remove("lazy-loading");
			} catch(ex: any) {
				this.classList.add("danger");
				throw ex;
			} finally {
				this.#isRefreshing = false;
			}
		}));
	}
	connectedCallback() {
		this.classList.add("framed-box-small");
		this.refresh();
	}
}
CourtConfigElement.registerElement();

seiUtilEventEmitter.on("defaultNetworkChanged", (ev) => {
	(qa("[is=\"court-config\"]") as NodeListOf<CourtConfigElement>).forEach(elem => {
		elem.refresh();
	});
});

seiUtilEventEmitter.on("defaultProviderChanged", (ev) => {
	(qa("[is=\"court-config\"]") as NodeListOf<CourtConfigElement>).forEach(elem => {
		elem.refresh();
	});
});

class CourtConfigModalElement extends CourtConfigModalAutogen {
	static showModalAndGetValues(): Promise<CourtConfigModalFormValues1 | null> {
		const dialog = q("dialog[is=court-config-modal]") as CourtConfigModalElement | null;
		if (dialog == null) {
			const newDialog = new CourtConfigModalElement();
			document.body.append(newDialog);
			return newDialog.showModalAndGetValues();
		} else {
			return dialog.showModalAndGetValues();
		}
	}
	static showModalAndDoTransaction() {
		const dialog = q("dialog[is=court-config-modal]") as CourtConfigModalElement | null;
		if (dialog == null) {
			const newDialog = new CourtConfigModalElement();
			document.body.append(newDialog);
			return newDialog.showModalAndDoTransaction();
		} else {
			return dialog.showModalAndDoTransaction();
		}
	}
	constructor() {
		super();
		this.refs.cancelBtn.addEventListener("click", (ev) => {
			ev.preventDefault();
			this.close();
		})
	}
	showModalAndGetValues(): Promise<CourtConfigModalFormValues1 | null> {
		this.showModal();
		this.refs.form.reset();
		return new Promise(resolve => {
			const submitCallback = (_: Event) => {
				resolve(this.refs.form.values());
				this.removeEventListener("close", closeCallback);
			};
			const closeCallback = (_: Event) => {
				resolve(null);
				this.removeEventListener("submit", submitCallback);
			};
			this.addEventListener("submit", submitCallback, {once: true, passive: true});
			this.addEventListener("close", closeCallback, {once: true, passive: true});
		});
	}
	showModalAndDoTransaction() {
		msgBoxIfThrow(async () => {
			const inputs = await this.showModalAndGetValues();
			if (inputs == null) {
				return;
			}
			const configOptions: Extract<CourtAdminExecuteMsg, {change_config: {}}>["change_config"] = {};
			for (const k of [
				"minimum_vote_proposal_percent",
				"minimum_vote_turnout_percent",
				"minimum_vote_pass_percent"
			] as const) {
				if (!isNaN(inputs[k])) {
					configOptions[k] = inputs[k];
				}
			}
			for (const k of [
				"max_proposal_expiry_time_seconds",
				"execution_expiry_time_seconds"
			] as const) {
				const timeSeconds = Math.floor(parseTimeAmount(inputs[k]) / 1000);
				if (!isNaN(timeSeconds) && timeSeconds > 0) {
					configOptions[k] = timeSeconds;
				}
			}
			
			const task = new FullscreenLoadingTask();
			try {
				task.text = "Connecting to wallet...";
				const client = await WebClientEnv.get();
				task.text = "";
				const contract = getCourtCoordinatorFromChainId(client.queryClient, client.chainId);
				await client.executeContract(
					contract.buildAdminIx({
						change_config: configOptions
					})
				);
			} finally {
				task.hide();
				(qa("[is=\"court-config\"]") as NodeListOf<CourtConfigElement>).forEach(elem => {
					elem.refresh();
				});
			}
		});
	}
}
CourtConfigModalElement.registerElement();


class AdminMintSharesModalElement extends AdminMintSharesModalAutogen {
	static showModalAndGetValues(): Promise<AdminMintSharesModalFormValues2 | null> {
		const dialog = q("dialog[is=admin-mint-shares-modal]") as AdminMintSharesModalElement | null;
		if (dialog == null) {
			const newDialog = new AdminMintSharesModalElement();
			document.body.append(newDialog);
			return newDialog.showModalAndGetValues();
		} else {
			return dialog.showModalAndGetValues();
		}
	}
	static showModalAndDoTransaction() {
		const dialog = q("dialog[is=admin-mint-shares-modal]") as AdminMintSharesModalElement | null;
		if (dialog == null) {
			const newDialog = new AdminMintSharesModalElement();
			document.body.append(newDialog);
			return newDialog.showModalAndDoTransaction();
		} else {
			return dialog.showModalAndDoTransaction();
		}
	}
	constructor() {
		super();
		this.refs.cancelBtn.addEventListener("click", (ev) => {
			ev.preventDefault();
			this.close();
		})
	}
	showModalAndGetValues(): Promise<AdminMintSharesModalFormValues2 | null> {
		this.showModal();
		this.refs.form.reset();
		return new Promise(resolve => {
			const submitCallback = (ev: Event) => {
				if (!this.refs.form.elements.amount.value) {
					ev.preventDefault();
					this.refs.form.elements.amount.value = "0";
					this.refs.form.reportValidity();
					return;
				}
				resolve(this.refs.form.values());
				this.removeEventListener("close", closeCallback);
				this.removeEventListener("submit", submitCallback);
			};
			const closeCallback = (_: Event) => {
				resolve(null);
				this.removeEventListener("submit", submitCallback);
			};
			this.addEventListener("submit", submitCallback);
			this.addEventListener("close", closeCallback, {once: true, passive: true});
		});
	}
	showModalAndDoTransaction() {
		msgBoxIfThrow(async () => {
			const inputs = await this.showModalAndGetValues();
			if (inputs == null) {
				return;
			}

			const task = new FullscreenLoadingTask();
			try {
				task.text = "Connecting to wallet...";
				const client = await WebClientEnv.get();
				if (inputs.recipient.startsWith("0x")) {
					inputs.recipient = (await client.queryClient.evm.seiAddressByEVMAddress({
						evmAddress: inputs.recipient
					})).seiAddress;
					if (!inputs.recipient) {
						alert(
							"Unknown recipient",
							"New voting shares may not be minted to EVM accounts with no transaction history.",
							"warning",
							"warning"
						);
						return;
					}
				}
				task.text = "";
				const contract = getCourtCoordinatorFromChainId(client.queryClient, client.chainId);
				await client.executeContract(
					contract.buildAdminIx({
						mint_shares: {
							amount: inputs.amount + "",
							receiver: inputs.recipient ? inputs.recipient : client.getAccount().seiAddress
						}
					})
				);
			} finally {
				task.hide();
				(qa("[is=\"court-config\"]") as NodeListOf<CourtConfigElement>).forEach(elem => {
					elem.refresh();
				});
			}
		});
	}
}
AdminMintSharesModalElement.registerElement();
