import { alert, confirm, FullscreenLoadingTask, msgBoxIfThrow } from "@crownfi/css-gothic-fantasy";
import { CourtProposalCreatorAutogen, CourtProposalCreatorDaoAdminAutogen, CourtProposalCreatorExecuteEvmAutogen, CourtProposalCreatorExecuteWasmAutogen, CourtProposalCreatorExecuteWasmCoinAutogen, CourtProposalCreatorSendCoinAutogen } from "./_autogen.js";
import { preventDefault, q, qa } from "@aritz-cracker/browser-utils";
import Sortable from "sortablejs";
import { WebClientEnv } from "@crownfi/sei-webui-utils";
import { CourtExecuteMsg, getCourtCoordinatorAddressFromChainId, getCourtCoordinatorFromChainId, ProposedCourtMsgJsonable } from "@crownfi/court-coordinator-sdk";
import { CourtProposalElement } from "./proposal_view.js";
import { ClientEnv, getDefaultNetworkConfig, isValidEvmAddress, nativeDenomSortCompare, seiUtilEventEmitter } from "@crownfi/sei-utils";
import { humanReadableTimeAmount, parseTimeAmount } from "../time_format.js";
import { Coin } from "@cosmjs/proto-signing";

export class CourtProposalCreatorElement extends CourtProposalCreatorAutogen {
	static showModalAndDoTransaction() {
		const dialog = q("dialog[is=court-proposal-creator]") as CourtProposalCreatorElement | null;
		if (dialog == null) {
			const newDialog = new CourtProposalCreatorElement();
			document.body.append(newDialog);
			return newDialog.showModalAndDoTransaction();
		} else {
			return dialog.showModalAndDoTransaction();
		}
	}
	#sortable: Sortable | null = null;
	constructor() {
		super();
		this.addEventListener("submit", (ev) => {
			ev.preventDefault();
		});
		this.refs.cancelButton.addEventListener("click", (ev) => {
			ev.preventDefault();
			this.close();
		});
		this.addEventListener("dropdownSelect", (ev) => {
			switch (ev.detail.selectedValue) {
				case "send_coin":
					this.refs.instructionContainer.appendChild(
						new CourtProposalCreatorSendCoinElement()
					);
					break;
				case "execute_evm":
					this.refs.instructionContainer.appendChild(
						new CourtProposalCreatorExecuteEvmElement()
					);
					break;
				case "execute_wasm":
					this.refs.instructionContainer.appendChild(
						new CourtProposalCreatorExecuteWasmElement()
					);
					break;
				case "change_court_admin":
					this.refs.instructionContainer.appendChild(
						new CourtProposalCreatorDaoAdminElement()
					);
					break;
				case "upgrade_wasm_contract":
					alert("TODO: handle upgrade_wasm_contract");
					break;
				case "change_wasm_contract_admin":
					alert("TODO: handle change_wasm_contract_admin");
					break;
				case "clear_wasm_contract_admin":
					alert("TODO: handle clear_wasm_contract_admin");
					break;
				case "tokenfactory_mint":
					alert("TODO: handle tokenfactory_mint");
					break;
				default:
					alert("Unknown option value " + ev.detail.selectedValue);
			}
		});
	}
	proposeTransactionMsgs(): ProposedCourtMsgJsonable[] | null {
		let result = [];
		for (const elem of this.refs.instructionContainer.children) {
			if (!("proposeTransactionMsg" in elem)) {
				continue;
			}
			const msg = (elem as CourtProposalMessageCreator).proposeTransactionMsg();
			if (msg == null) {
				return null;
			}
			result.push(msg);
		}
		return result;
	}
	proposeTransactionExpiryTimeSeconds(): number {
		if (!this.refs.expiry.reportValidity()) {
			return NaN;
		}
		return Math.floor(parseTimeAmount(this.refs.expiry.value || this.refs.expiry.placeholder) / 1000);
	}
	proposeTransactionParams(): {msgs: ProposedCourtMsgJsonable[], expiry_time_seconds: number} | null {
		const msgs = this.proposeTransactionMsgs();
		if (msgs == null) {
			return null;
		}
		const expiry_time_seconds = this.proposeTransactionExpiryTimeSeconds();
		if (isNaN(expiry_time_seconds)) {
			return null;
		}
		return {
			msgs,
			expiry_time_seconds
		};
	}
	isDirty(): boolean {
		return this.refs.instructionContainer.childElementCount > 0 || this.refs.expiry.value != "";
	}
	reset() {
		this.refs.instructionContainer.innerHTML = "";
		this.refs.executeButton.disabled = true;
		this.refs.expiry.value = "";
	}
	async showModalAndGetProposeTransactionParams(): Promise<{msgs: ProposedCourtMsgJsonable[], expiry_time_seconds: number} | null> {
		this.refreshConfig();
		this.showModal();
		if (this.isDirty() && await confirm(
			"New proposal",
			"Would you like to keep the proposal information already entered or reset all inputs?",
			"question",
			undefined,
			"Keep",
			"Reset"
		)) {
			this.refs.instructionContainer.innerHTML = "";
			this.refs.executeButton.disabled = true;
			this.refs.expiry.value = "";
		}
		return new Promise(resolve => {
			const executeClickCallback = (_: Event) => {
				const result = this.proposeTransactionParams();
				if (result != null) {
					resolve(result);
					this.removeEventListener("close", closeCallback);
					this.refs.executeButton.removeEventListener("click", executeClickCallback);
					this.close();
				}
			};
			const closeCallback = (_: Event) => {
				resolve(null);
				this.refs.executeButton.removeEventListener("click", executeClickCallback);
			};
			this.refs.executeButton.addEventListener("click", executeClickCallback, {passive: true});
			this.addEventListener("close", closeCallback, {once: true, passive: true});
		});
	}
	showModalAndDoTransaction() {
		msgBoxIfThrow(async () => {
			const inputs = await this.showModalAndGetProposeTransactionParams();
			if (inputs == null) {
				return;
			}
			const task = new FullscreenLoadingTask();
			try {
				task.text = "Connecting to wallet...";
				const client = await WebClientEnv.get();
				task.text = "";
				const contract = getCourtCoordinatorFromChainId(client.queryClient, client.chainId);
				await client.executeContract(
					contract.buildProposeTransactionIx(inputs)
				);
			} finally {
				task.hide();
				// The first court-proposal element will create a new sibling when it realizes it's no longer the
				// newest proposal.
				(q("[is=\"court-proposal\"]") as CourtProposalElement | null)?.refresh();
			}
		});
	}
	#isRefreshingConfig: boolean = false;
	#shouldRefreshConfig: boolean = false
	refreshConfig() {
		this.#shouldRefreshConfig = true;
		if (this.#isRefreshingConfig) {
			return;
		}
		this.#isRefreshingConfig = true;
		(async () => {
			this.refs.maxExpiry.classList.add("loading-spinner-inline");
			this.refs.maxExpiry.innerText = "\u200B";
			do {
				this.#shouldRefreshConfig = false;
				const client = await ClientEnv.get();
				const contract = getCourtCoordinatorFromChainId(client.queryClient, client.chainId);
				const config = await contract.queryConfig();
				this.refs.expiry.placeholder = humanReadableTimeAmount(config.max_proposal_expiry_time_seconds * 1000);
				this.refs.maxExpiry.innerText = this.refs.expiry.placeholder;
			}while(this.#shouldRefreshConfig);
		})().catch(ex => {
			this.refs.maxExpiry.innerText = "";
			console.error("Could not show the user the max proposal time", ex);
		}).finally(() => {
			this.refs.maxExpiry.classList.remove("loading-spinner-inline");
			this.#isRefreshingConfig = false;
		});
	}
	connectedCallback() {
		if (this.#sortable != null) {
			return;
		}
		this.#sortable = new Sortable(this.refs.instructionContainer, {
			handle: '.drag-handle',
			animation: 175,
			ghostClass: "draggable-placeholder"
		});
		this.refreshConfig();
	}
}
CourtProposalCreatorElement.registerElement();
seiUtilEventEmitter.on("defaultNetworkChanged", (ev) => {
	(qa("[is=\"court-proposal-creator\"]") as NodeListOf<CourtProposalCreatorElement>).forEach(elem => {
		elem.refreshConfig();
	});
});
seiUtilEventEmitter.on("defaultProviderChanged", (ev) => {
	(qa("[is=\"court-proposal-creator\"]") as NodeListOf<CourtProposalCreatorElement>).forEach(elem => {
		elem.refreshConfig();
	});
});

type CourtProposalMessageCreator = CourtProposalCreatorSendCoinElement | CourtProposalCreatorExecuteEvmElement;

export class CourtProposalCreatorSendCoinElement extends CourtProposalCreatorSendCoinAutogen {
	constructor() {
		super();
		this.refs.form.addEventListener("submit", preventDefault);
		this.refs.deleteButton.addEventListener("click", (ev) => {
			this.remove();
		});
	}
	proposeTransactionMsg(): Extract<ProposedCourtMsgJsonable, {send_coin: any}> | null {
		if (!this.refs.form.elements.recipient.value) {
			this.refs.form.elements.recipient.required = true;
		}
		if (!this.refs.form.elements.denom.value) {
			this.refs.form.elements.denom.required = true;
		}
		if (!this.refs.form.elements.amount.value) {
			this.refs.form.elements.amount.value = "0";
			this.refs.form.elements.amount.required = true;
		}
		if (!this.refs.form.reportValidity()) {
			return null;
		}
		return {
			send_coin: {
				amount: this.refs.form.elements.amount.value,
				denom: this.refs.form.elements.denom.value,
				to: this.refs.form.elements.recipient.value
			}
		};
	}
	connectedCallback(){
		this.classList.add("draggable-with-handle");
	}
}
CourtProposalCreatorSendCoinElement.registerElement();

export class CourtProposalCreatorExecuteEvmElement extends CourtProposalCreatorExecuteEvmAutogen {
	constructor() {
		super();
		this.refs.form.addEventListener("submit", preventDefault);
		this.refs.deleteButton.addEventListener("click", (ev) => {
			this.remove();
		});
	}
	proposeTransactionMsg(): Extract<ProposedCourtMsgJsonable, {execute_evm_contract: any}> | null {
		if (!this.refs.form.elements.recipient.value) {
			this.refs.form.elements.recipient.required = true;
		}
		if (!this.refs.form.elements.amount.value) {
			this.refs.form.elements.amount.value = "0";
			this.refs.form.elements.amount.required = true;
		}
		if (!this.refs.form.reportValidity()) {
			return null;
		}
		const contract = this.refs.form.elements.recipient.value;
		const value = this.refs.form.elements.amount.value;
		if (!isValidEvmAddress(this.refs.form.elements.recipient.value, true)) {
			alert(
				this.refs.form.elements.recipient.value +
				" does not have the proper capitalization.\nPlease check the address and try again."
			);
			return null;
		}
		let msg = this.refs.form.elements.data.value.replace(/\s/, "");
		if (msg.startsWith("0x")) {
			msg = msg.substring(2);
		}	
		// The regex alowing empty strings is intentional
		if (/^[0-9a-fA-F]*$/.test(msg)) {
			msg = Buffer.from(msg, "hex").toString("base64");
		} else if (
			!/^(?:[A-Za-z0-9+\/]{4})*(?:[A-Za-z0-9+\/]{4}|[A-Za-z0-9+\/]{3}=?|[A-Za-z0-9+\/]{2}={0,2}?|[A-Za-z0-9+\/]={0,3}?)$/.test(msg)
		) {
			alert("EVM Execute instruction does not contain a valid hex or base64 string.");
			return null;
		}
		return {
			execute_evm_contract: {
				contract,
				value,
				msg
			}
		};
	}
	connectedCallback(){
		this.classList.add("draggable-with-handle");
	}
}
CourtProposalCreatorExecuteEvmElement.registerElement();

export class CourtProposalCreatorExecuteWasmCoinElement extends CourtProposalCreatorExecuteWasmCoinAutogen {
	constructor() {
		super();
		this.refs.form.addEventListener("submit", preventDefault);
		this.refs.deleteButton.addEventListener("click", (ev) => {
			this.remove();
		});
	}
	coin(): Coin | null {
		if (!this.refs.form.elements.amount.value) {
			this.refs.form.elements.amount.value = "0";
			this.refs.form.elements.amount.required = true;
		}
		if (!this.refs.form.elements.denom.value) {
			this.refs.form.elements.amount.required = true;
		}
		if (!this.refs.form.reportValidity()) {
			return null;
		}
		return {
			denom: this.refs.form.elements.denom.value,
			amount: this.refs.form.elements.amount.value
		};
	}
	connectedCallback(): void {
		this.classList.add("draggable-with-handle");
	}
}
CourtProposalCreatorExecuteWasmCoinElement.registerElement();

export class CourtProposalCreatorExecuteWasmElement extends CourtProposalCreatorExecuteWasmAutogen {
	constructor() {
		super();
		this.refs.form.addEventListener("submit", preventDefault);
		this.refs.deleteButton.addEventListener("click", (ev) => {
			this.remove();
		});
		this.refs.addCoinsButton.addEventListener("click", (ev) => {
			this.refs.coinsContainer.appendChild(
				new CourtProposalCreatorExecuteWasmCoinElement()
			)
		})
	}
	#sortable: Sortable | null = null;
	proposeTransactionMsg(): Extract<ProposedCourtMsgJsonable, {execute_wasm_contract: any}> | null {
		if (!this.refs.form.elements.recipient.value) {
			this.refs.form.elements.recipient.required = true;
		}
		if (!this.refs.form.reportValidity()) {
			return null;
		}
		const contract = this.refs.form.elements.recipient.value;
		const funds = [];
		for (const elem of this.refs.coinsContainer.children) {
			if (!(elem instanceof CourtProposalCreatorExecuteWasmCoinElement)){
				continue;
			}
			const coin = elem.coin();
			if (coin == null) {
				return null;
			}
			funds.push(coin)
		}
		funds.sort(nativeDenomSortCompare);
		
		if (!isValidEvmAddress(this.refs.form.elements.recipient.value, true)) {
			alert(
				this.refs.form.elements.recipient.value +
				" does not have the proper capitalization.\nPlease check the address and try again."
			);
			return null;
		}
		let msg = this.refs.form.elements.data.value;
		switch (this.refs.form.elements.data_type.value) {
			case "json":
				try {
					msg = JSON.stringify(JSON.parse(msg));
				}catch(ex: any) {
					alert("Execute wasm instruction does not contain valid JSON: " + ex.name + ": " + ex.message);
					return null;
				}
			case "printable":
				msg = Buffer.from(msg, "utf8").toString("base64");
				break;
			case "base64":
				msg = msg.replace(/\s/, "");
				if (
					!/^(?:[A-Za-z0-9+\/]{4})*(?:[A-Za-z0-9+\/]{4}|[A-Za-z0-9+\/]{3}=?|[A-Za-z0-9+\/]{2}={0,2}?|[A-Za-z0-9+\/]={0,3}?)$/.test(msg)
				) {
					alert("Execute wasm instruction does not contain a valid hex base64 string.");
					return null;
				}
				break;
			case "":
				return null;
		}
		return {
			execute_wasm_contract: {
				contract,
				funds,
				msg
			}
		};
	}
	connectedCallback(){
		this.classList.add("draggable-with-handle");
		if (this.#sortable != null) {
			return;
		}
		this.#sortable = new Sortable(this.refs.coinsContainer, {
			handle: '.drag-handle',
			animation: 175,
			ghostClass: "draggable-placeholder"
		});
	}
}
CourtProposalCreatorExecuteWasmElement.registerElement();

export class CourtProposalCreatorDaoAdminElement extends CourtProposalCreatorDaoAdminAutogen {
	constructor() {
		super();
		this.refs.form.addEventListener("submit", preventDefault);
		this.refs.deleteButton.addEventListener("click", (ev) => {
			this.remove();
		});
	}
	proposeTransactionMsg(): Extract<ProposedCourtMsgJsonable, {execute_wasm_contract: any}> | null {
		if (!this.refs.form.elements.admin.value) {
			this.refs.form.elements.admin.required = true;
		}
		if (!this.refs.form.reportValidity()) {
			return null;
		}
		return {
			execute_wasm_contract: {
				contract: getCourtCoordinatorAddressFromChainId(getDefaultNetworkConfig().chainId),
				funds: [],
				msg: Buffer.from(JSON.stringify({
					admin: {
						change_admin: {
							admin: this.refs.form.elements.admin.value
						}
					}
				} satisfies CourtExecuteMsg)).toString("base64")
			}
		};
	}
}
CourtProposalCreatorDaoAdminAutogen.registerElement();
