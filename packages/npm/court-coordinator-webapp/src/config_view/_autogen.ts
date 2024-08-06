// auto-generated by C.E.W.T.
// DO NOT EDIT BY HAND!!
import { normalizeFormValues } from "@aritz-cracker/browser-utils";
export class CourtConfigRefs {
	#element: HTMLElement | ShadowRoot;
	constructor(element: HTMLElement | ShadowRoot) {
		this.#element = element;
	}
	#configAdmin?: HTMLTableCellElement;
	get configAdmin() {
		if (this.#configAdmin === undefined) {
			this.#configAdmin = this.#element.querySelector("[cewt-ref=\"config-admin\"]:not(:scope [is] *)")!;
		}
		return this.#configAdmin;
	}
	#totalShares?: HTMLTableCellElement;
	get totalShares() {
		if (this.#totalShares === undefined) {
			this.#totalShares = this.#element.querySelector("[cewt-ref=\"total-shares\"]:not(:scope [is] *)")!;
		}
		return this.#totalShares;
	}
	#configProposalsAllowed?: HTMLInputElement;
	get configProposalsAllowed() {
		if (this.#configProposalsAllowed === undefined) {
			this.#configProposalsAllowed = this.#element.querySelector("[cewt-ref=\"config-proposals-allowed\"]:not(:scope [is] *)")!;
		}
		return this.#configProposalsAllowed;
	}
	#configMinVotesNewPercent?: HTMLSpanElement;
	get configMinVotesNewPercent() {
		if (this.#configMinVotesNewPercent === undefined) {
			this.#configMinVotesNewPercent = this.#element.querySelector("[cewt-ref=\"config-min-votes-new-percent\"]:not(:scope [is] *)")!;
		}
		return this.#configMinVotesNewPercent;
	}
	#configMinVotesNewAmount?: HTMLSpanElement;
	get configMinVotesNewAmount() {
		if (this.#configMinVotesNewAmount === undefined) {
			this.#configMinVotesNewAmount = this.#element.querySelector("[cewt-ref=\"config-min-votes-new-amount\"]:not(:scope [is] *)")!;
		}
		return this.#configMinVotesNewAmount;
	}
	#configMinTurnoutPercent?: HTMLSpanElement;
	get configMinTurnoutPercent() {
		if (this.#configMinTurnoutPercent === undefined) {
			this.#configMinTurnoutPercent = this.#element.querySelector("[cewt-ref=\"config-min-turnout-percent\"]:not(:scope [is] *)")!;
		}
		return this.#configMinTurnoutPercent;
	}
	#configMinTurnoutAmount?: HTMLSpanElement;
	get configMinTurnoutAmount() {
		if (this.#configMinTurnoutAmount === undefined) {
			this.#configMinTurnoutAmount = this.#element.querySelector("[cewt-ref=\"config-min-turnout-amount\"]:not(:scope [is] *)")!;
		}
		return this.#configMinTurnoutAmount;
	}
	#configMinApprovalPercent?: HTMLSpanElement;
	get configMinApprovalPercent() {
		if (this.#configMinApprovalPercent === undefined) {
			this.#configMinApprovalPercent = this.#element.querySelector("[cewt-ref=\"config-min-approval-percent\"]:not(:scope [is] *)")!;
		}
		return this.#configMinApprovalPercent;
	}
	#configVotingTime?: HTMLTableCellElement;
	get configVotingTime() {
		if (this.#configVotingTime === undefined) {
			this.#configVotingTime = this.#element.querySelector("[cewt-ref=\"config-voting-time\"]:not(:scope [is] *)")!;
		}
		return this.#configVotingTime;
	}
	#configExecutionWindow?: HTMLTableCellElement;
	get configExecutionWindow() {
		if (this.#configExecutionWindow === undefined) {
			this.#configExecutionWindow = this.#element.querySelector("[cewt-ref=\"config-execution-window\"]:not(:scope [is] *)")!;
		}
		return this.#configExecutionWindow;
	}
	#configChangeTime?: HTMLTableCellElement;
	get configChangeTime() {
		if (this.#configChangeTime === undefined) {
			this.#configChangeTime = this.#element.querySelector("[cewt-ref=\"config-change-time\"]:not(:scope [is] *)")!;
		}
		return this.#configChangeTime;
	}
	#adminNote?: HTMLDivElement;
	get adminNote() {
		if (this.#adminNote === undefined) {
			this.#adminNote = this.#element.querySelector("[cewt-ref=\"admin-note\"]:not(:scope [is] *)")!;
		}
		return this.#adminNote;
	}
	#adminConfigButton?: HTMLButtonElement;
	get adminConfigButton() {
		if (this.#adminConfigButton === undefined) {
			this.#adminConfigButton = this.#element.querySelector("[cewt-ref=\"admin-config-button\"]:not(:scope [is] *)")!;
		}
		return this.#adminConfigButton;
	}
	#adminProposalsButton?: HTMLInputElement;
	get adminProposalsButton() {
		if (this.#adminProposalsButton === undefined) {
			this.#adminProposalsButton = this.#element.querySelector("[cewt-ref=\"admin-proposals-button\"]:not(:scope [is] *)")!;
		}
		return this.#adminProposalsButton;
	}
	#adminMintButton?: HTMLButtonElement;
	get adminMintButton() {
		if (this.#adminMintButton === undefined) {
			this.#adminMintButton = this.#element.querySelector("[cewt-ref=\"admin-mint-button\"]:not(:scope [is] *)")!;
		}
		return this.#adminMintButton;
	}
	#adminAbdicateButton?: HTMLButtonElement;
	get adminAbdicateButton() {
		if (this.#adminAbdicateButton === undefined) {
			this.#adminAbdicateButton = this.#element.querySelector("[cewt-ref=\"admin-abdicate-button\"]:not(:scope [is] *)")!;
		}
		return this.#adminAbdicateButton;
	}
}
let _templateCourtConfig: HTMLTemplateElement | null = null;
function getCourtConfigTemplate(): HTMLTemplateElement {
	if (_templateCourtConfig == null) {
		 _templateCourtConfig = document.createElement("template")
		 _templateCourtConfig.innerHTML = "\n\t<h3 class=\"text-primary text-fantasy\">DAO Config</h3>\n\t<p>\n\t\tThese are the current configuration settings for the DAO. The admin, if set, can change the configuration options while there are no proposals in progress.\n\t</p>\n\t<table>\n\t\t<tbody><tr>\n\t\t\t<th>Admin</th><td style=\"word-break: break-word;\" cewt-ref=\"config-admin\">-</td>\n\t\t</tr>\n\t\t<tr>\n\t\t\t<th>Total shares minted</th>\n\t\t\t<td cewt-ref=\"total-shares\">-</td>\n\t\t</tr>\n\t\t<tr>\n\t\t\t<th>Proposals enabled</th>\n\t\t\t<td>\n\t\t\t\t<label class=\"checkbox\">\n\t\t\t\t\t<input type=\"checkbox\" disabled=\"\" cewt-ref=\"config-proposals-allowed\"><span>&nbsp;</span>\n\t\t\t\t</label>\n\t\t\t</td>\n\t\t</tr>\n\t\t<tr>\n\t\t\t<th>Minimum votes for new proposal</th>\n\t\t\t<td>\n\t\t\t\t<span cewt-ref=\"config-min-votes-new-percent\">--</span>% of total supply\n\t\t\t\t(<span cewt-ref=\"config-min-votes-new-amount\">--</span>)\n\t\t\t</td>\n\t\t</tr>\n\t\t<tr>\n\t\t\t<th>Minimum vote turnout for proposal approval</th>\n\t\t\t<td>\n\t\t\t\t<span cewt-ref=\"config-min-turnout-percent\">--</span>% of total supply\n\t\t\t\t(<span cewt-ref=\"config-min-turnout-amount\">--</span>)\n\t\t\t</td>\n\t\t</tr>\n\t\t<tr>\n\t\t\t<th>Minimum proposal approval rating</th>\n\t\t\t<td><span cewt-ref=\"config-min-approval-percent\">--</span>%</td>\n\t\t</tr>\n\t\t<tr>\n\t\t\t<th>Maximum proposal voting time</th><td cewt-ref=\"config-voting-time\">-</td>\n\t\t</tr>\n\t\t<tr>\n\t\t\t<th>Proposal execution window</th><td cewt-ref=\"config-execution-window\">-</td>\n\t\t</tr>\n\t\t<tr>\n\t\t\t<th>Last config change time</th><td cewt-ref=\"config-change-time\">-</td>\n\t\t</tr>\n\t</tbody></table>\n\t<h3 class=\"text-primary text-fantasy\">Admin controls</h3>\n\t<div cewt-ref=\"admin-note\" class=\"important-note warning\">\n\t\tYou\'re not the admin, so the options below are useless to you.\n\t</div>\n\t<div style=\"display: flex; flex-direction: row; align-items: center; justify-content: space-around; flex-wrap: wrap; gap: 4px\">\n\t\t<button class=\"fantasy-ornamental\" cewt-ref=\"admin-config-button\">Change config</button>\n\t\t<label class=\"button fantasy-ornamental\"><span>Allow new proposals</span><input cewt-ref=\"admin-proposals-button\" type=\"checkbox\"></label>\n\t\t<button class=\"fantasy-ornamental\" cewt-ref=\"admin-mint-button\">Mint shares</button>\n\t\t<button class=\"fantasy-ornamental danger\" cewt-ref=\"admin-abdicate-button\">Relinquish adminship</button>\n\t</div>\n";
	}
	return _templateCourtConfig;
}
export class CourtConfigAutogen extends HTMLDivElement {
	readonly refs: CourtConfigRefs;
	constructor() {
		super();
		if (this.childElementCount == 0) {
			this.appendChild(
				getCourtConfigTemplate()
					.content
					.cloneNode(true)
			);
		}
		this.setAttribute("is", "court-config"); // allow for easy query selecting
		this.refs = new CourtConfigRefs(this);
	}
	connectedCallback() {
		// To be overridden by child class
	}
	disconnectedCallback() {
		// To be overridden by child class
	}
	adoptedCallback() {
		// To be overridden by child class
	}
	public static registerElement() {
		customElements.define("court-config", this, { extends: "div"});
	}
}
export class CourtConfigModalRefs {
	#element: HTMLElement | ShadowRoot;
	constructor(element: HTMLElement | ShadowRoot) {
		this.#element = element;
	}
	#form?: HTMLFormElementKnownControls<CourtConfigModalFormCollection1, CourtConfigModalFormValues1>;
	get form() {
		if (this.#form === undefined) {
			this.#form = this.#element.querySelector("[cewt-ref=\"form\"]:not(:scope [is] *)")!;
			this.#form.values = normalizeFormValues.bind(this.#form, this.#form) as any;
		}
		return this.#form;
	}
	#cancelBtn?: HTMLButtonElement;
	get cancelBtn() {
		if (this.#cancelBtn === undefined) {
			this.#cancelBtn = this.#element.querySelector("[cewt-ref=\"cancel-btn\"]:not(:scope [is] *)")!;
		}
		return this.#cancelBtn;
	}
}
let _templateCourtConfigModal: HTMLTemplateElement | null = null;
function getCourtConfigModalTemplate(): HTMLTemplateElement {
	if (_templateCourtConfigModal == null) {
		 _templateCourtConfigModal = document.createElement("template")
		 _templateCourtConfigModal.innerHTML = "\n\t<h1>Change configuration</h1>\n\t<form cewt-ref=\"form\" method=\"dialog\">\n\t\t<label>\n\t\t\t<span>Minimum votes for new proposal (as percentage of total supply)</span>\n\t\t\t<input name=\"minimum_vote_proposal_percent\" type=\"number\" min=\"0\" max=\"100\" placeholder=\"unchanged\">\n\t\t</label>\n\t\t<label>\n\t\t\t<span>Minimum voter turnout (percentage)</span>\n\t\t\t<input name=\"minimum_vote_turnout_percent\" type=\"number\" min=\"0\" max=\"100\" placeholder=\"unchanged\">\n\t\t</label>\n\t\t<label>\n\t\t\t<span>Minimum proposal approval rating (percentage)</span>\n\t\t\t<input name=\"minimum_vote_pass_percent\" type=\"number\" min=\"0\" max=\"100\" placeholder=\"unchanged\">\n\t\t</label>\n\t\t<label>\n\t\t\t<span>Maximum proposal voting time</span>\n\t\t\t<input name=\"max_proposal_expiry_time_seconds\" type=\"text\" placeholder=\"unchanged\" title=\"Enter duration (e.g., 5w4d3h2m1s for 5 weeks, 4 days, 3 hours, 2 minutes, 1 second).\" pattern=\"^\\s*(\\d+w)?\\s*(\\d+d)?\\s*(\\d+h)?\\s*(\\d+m)?\\s*(\\d+s)?\\s*(\\d+ms)?\\s*$\">\n\t\t</label>\n\t\t<label>\n\t\t\t<span>Proposal execution window</span>\n\t\t\t<input name=\"execution_expiry_time_seconds\" type=\"text\" placeholder=\"unchanged\" title=\"Enter duration (e.g., 5w4d3h2m1s for 5 weeks, 4 days, 3 hours, 2 minutes, 1 second).\" pattern=\"^\\s*(\\d+w)?\\s*(\\d+d)?\\s*(\\d+h)?\\s*(\\d+m)?\\s*(\\d+s)?\\s*(\\d+ms)?\\s*$\">\n\t\t</label>\n\t\t\n\t\t<div class=\"button-row equal-width\">\n\t\t\t<button class=\"primary small\">Apply changes</button>\n\t\t\t<button class=\"small\" cewt-ref=\"cancel-btn\">Cancel</button>\n\t\t</div>\n\t</form>\n";
	}
	return _templateCourtConfigModal;
}
export class CourtConfigModalAutogen extends HTMLDialogElement {
	readonly refs: CourtConfigModalRefs;
	constructor() {
		super();
		if (this.childElementCount == 0) {
			this.appendChild(
				getCourtConfigModalTemplate()
					.content
					.cloneNode(true)
			);
		}
		this.setAttribute("is", "court-config-modal"); // allow for easy query selecting
		this.refs = new CourtConfigModalRefs(this);
	}
	connectedCallback() {
		// To be overridden by child class
	}
	disconnectedCallback() {
		// To be overridden by child class
	}
	adoptedCallback() {
		// To be overridden by child class
	}
	public static registerElement() {
		customElements.define("court-config-modal", this, { extends: "dialog"});
	}
}
export class AdminMintSharesModalRefs {
	#element: HTMLElement | ShadowRoot;
	constructor(element: HTMLElement | ShadowRoot) {
		this.#element = element;
	}
	#form?: HTMLFormElementKnownControls<AdminMintSharesModalFormCollection2, AdminMintSharesModalFormValues2>;
	get form() {
		if (this.#form === undefined) {
			this.#form = this.#element.querySelector("[cewt-ref=\"form\"]:not(:scope [is] *)")!;
			this.#form.values = normalizeFormValues.bind(this.#form, this.#form) as any;
		}
		return this.#form;
	}
	#cancelBtn?: HTMLButtonElement;
	get cancelBtn() {
		if (this.#cancelBtn === undefined) {
			this.#cancelBtn = this.#element.querySelector("[cewt-ref=\"cancel-btn\"]:not(:scope [is] *)")!;
		}
		return this.#cancelBtn;
	}
}
let _templateAdminMintSharesModal: HTMLTemplateElement | null = null;
function getAdminMintSharesModalTemplate(): HTMLTemplateElement {
	if (_templateAdminMintSharesModal == null) {
		 _templateAdminMintSharesModal = document.createElement("template")
		 _templateAdminMintSharesModal.innerHTML = "\n\t<h1>Dilute voting power</h1>\n\t<form cewt-ref=\"form\" method=\"dialog\">\n\t\t<label>\n\t\t\t<span>Shares to mint</span>\n\t\t\t<input name=\"amount\" type=\"number\" min=\"1\" placeholder=\"0\">\n\t\t</label>\n\t\t<label>\n\t\t\t<span>Recipiant (leave blank for yourself)</span>\n\t\t\t<input name=\"recipient\" type=\"text\" title=\"A valid 0x or sei1 address\" pattern=\"^(0x[a-fA-F0-9]{40}|sei1(?:[a-z0-9]{38}|[a-z0-9]{58}))$\">\n\t\t</label>\n\t\t<div class=\"button-row equal-width\">\n\t\t\t<button class=\"primary small\">money printer go brrrrr</button>\n\t\t\t<button class=\"small\" cewt-ref=\"cancel-btn\">Cancel</button>\n\t\t</div>\n\t</form>\n";
	}
	return _templateAdminMintSharesModal;
}
export class AdminMintSharesModalAutogen extends HTMLDialogElement {
	readonly refs: AdminMintSharesModalRefs;
	constructor() {
		super();
		if (this.childElementCount == 0) {
			this.appendChild(
				getAdminMintSharesModalTemplate()
					.content
					.cloneNode(true)
			);
		}
		this.setAttribute("is", "admin-mint-shares-modal"); // allow for easy query selecting
		this.refs = new AdminMintSharesModalRefs(this);
	}
	connectedCallback() {
		// To be overridden by child class
	}
	disconnectedCallback() {
		// To be overridden by child class
	}
	adoptedCallback() {
		// To be overridden by child class
	}
	public static registerElement() {
		customElements.define("admin-mint-shares-modal", this, { extends: "dialog"});
	}
}
export type CourtConfigModalFormCollection1 = HTMLFormControlsCollection & {
	"minimum_vote_proposal_percent": HTMLInputElement;
	namedItem(name: "minimum_vote_proposal_percent"): HTMLInputElement;
	"minimum_vote_turnout_percent": HTMLInputElement;
	namedItem(name: "minimum_vote_turnout_percent"): HTMLInputElement;
	"minimum_vote_pass_percent": HTMLInputElement;
	namedItem(name: "minimum_vote_pass_percent"): HTMLInputElement;
	"max_proposal_expiry_time_seconds": HTMLInputElement;
	namedItem(name: "max_proposal_expiry_time_seconds"): HTMLInputElement;
	"execution_expiry_time_seconds": HTMLInputElement;
	namedItem(name: "execution_expiry_time_seconds"): HTMLInputElement;
};
export type CourtConfigModalFormValues1 = {
	"minimum_vote_proposal_percent": number;
	"minimum_vote_turnout_percent": number;
	"minimum_vote_pass_percent": number;
	"max_proposal_expiry_time_seconds": string;
	"execution_expiry_time_seconds": string;
};
export type AdminMintSharesModalFormCollection2 = HTMLFormControlsCollection & {
	"amount": HTMLInputElement;
	namedItem(name: "amount"): HTMLInputElement;
	"recipient": HTMLInputElement;
	namedItem(name: "recipient"): HTMLInputElement;
};
export type AdminMintSharesModalFormValues2 = {
	"amount": number;
	"recipient": string;
};
interface HTMLFormElementKnownControls<C extends HTMLFormControlsCollection, V> extends HTMLFormElement {
	readonly elements: C;
	values: () => V;
};
