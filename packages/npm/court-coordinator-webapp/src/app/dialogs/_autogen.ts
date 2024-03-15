// auto-generated by C.E.W.T.
// DO NOT EDIT BY HAND!!
export class PopupModalRefs {
	#element: HTMLElement | ShadowRoot;
	constructor(element: HTMLElement | ShadowRoot) {
		this.#element = element;
	}
	#heading?: HTMLParagraphElement;
	get heading() {
		if (this.#heading === undefined) {
			this.#heading = this.#element.querySelector("[cewt-ref=\"heading\"]:not(:scope [is] *)")!;
		}
		return this.#heading;
	}
	#icon?: HTMLImageElement;
	get icon() {
		if (this.#icon === undefined) {
			this.#icon = this.#element.querySelector("[cewt-ref=\"icon\"]:not(:scope [is] *)")!;
		}
		return this.#icon;
	}
	#message?: HTMLParagraphElement;
	get message() {
		if (this.#message === undefined) {
			this.#message = this.#element.querySelector("[cewt-ref=\"message\"]:not(:scope [is] *)")!;
		}
		return this.#message;
	}
	#dismissBtn?: HTMLButtonElement;
	get dismissBtn() {
		if (this.#dismissBtn === undefined) {
			this.#dismissBtn = this.#element.querySelector("[cewt-ref=\"dismiss-btn\"]:not(:scope [is] *)")!;
		}
		return this.#dismissBtn;
	}
}
let _templatePopupModal: HTMLTemplateElement | null = null;
function getPopupModalTemplate(): HTMLTemplateElement {
	if (_templatePopupModal == null) {
		 _templatePopupModal = document.createElement("template")
		 _templatePopupModal.innerHTML = "\n\t<div class=\"title\">\n\t\t<p cewt-ref=\"heading\">undefined</p>\n\t</div>\n\t<div class=\"content\">\n\t\t<div class=\"message\">\n\t\t\t<img cewt-ref=\"icon\" class=\"notio-icon-info\">\n\t\t\t<div>\n\t\t\t\t<p cewt-ref=\"message\">null</p>\n\t\t\t</div>\n\t\t</div>\n\t\t<menu class=\"buttonRowRight\"><button cewt-ref=\"dismiss-btn\">OK</button></menu>\n\t</div>\n";
	}
	return _templatePopupModal;
}
export class PopupModalAutogen extends HTMLDialogElement {
	readonly refs: PopupModalRefs;
	static get observedAttributes() {
		return ["heading", "icon", "message"];
	}
	#attributeHeadingValue: string | null = null;
	get heading(): string | null {
		return this.#attributeHeadingValue;
	}
	set heading(v: string | null) {
		if (v == null) {
			this.removeAttribute("heading");
		}else{
			this.setAttribute("heading", v);
		}
	}
	protected onHeadingChanged(oldValue: string | null, newValue: string | null) {
		// To be overridden by child class
	}
	#attributeIconValue: string | null = null;
	get icon(): string | null {
		return this.#attributeIconValue;
	}
	set icon(v: string | null) {
		if (v == null) {
			this.removeAttribute("icon");
		}else{
			this.setAttribute("icon", v);
		}
	}
	protected onIconChanged(oldValue: string | null, newValue: string | null) {
		// To be overridden by child class
	}
	#attributeMessageValue: string | null = null;
	get message(): string | null {
		return this.#attributeMessageValue;
	}
	set message(v: string | null) {
		if (v == null) {
			this.removeAttribute("message");
		}else{
			this.setAttribute("message", v);
		}
	}
	protected onMessageChanged(oldValue: string | null, newValue: string | null) {
		// To be overridden by child class
	}
	attributeChangedCallback(name: string, oldValue: string | null, newValue: string | null) {
		switch(name) {
			case "heading":
				this.#attributeHeadingValue = newValue;
				this.onHeadingChanged(oldValue, newValue);
				break;
			case "icon":
				this.#attributeIconValue = newValue;
				this.onIconChanged(oldValue, newValue);
				break;
			case "message":
				this.#attributeMessageValue = newValue;
				this.onMessageChanged(oldValue, newValue);
				break;
			default:
				// Shouldn't happen
		}
	}
	constructor() {
		super();
		if (this.childElementCount == 0) {
			this.appendChild(
				getPopupModalTemplate()
					.content
					.cloneNode(true)
			);
		}
		this.setAttribute("is", "popup-modal"); // allow for easy query selecting
		this.refs = new PopupModalRefs(this);
		if (!this.getAttribute("class")) {
			this.setAttribute("class", "titledBox");
		}
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
		customElements.define("popup-modal", this, { extends: "dialog"});
	}
}
export class ErrorModalRefs {
	#element: HTMLElement | ShadowRoot;
	constructor(element: HTMLElement | ShadowRoot) {
		this.#element = element;
	}
	#heading?: HTMLParagraphElement;
	get heading() {
		if (this.#heading === undefined) {
			this.#heading = this.#element.querySelector("[cewt-ref=\"heading\"]:not(:scope [is] *)")!;
		}
		return this.#heading;
	}
	#message?: HTMLParagraphElement;
	get message() {
		if (this.#message === undefined) {
			this.#message = this.#element.querySelector("[cewt-ref=\"message\"]:not(:scope [is] *)")!;
		}
		return this.#message;
	}
	#errorDetails?: HTMLTextAreaElement;
	get errorDetails() {
		if (this.#errorDetails === undefined) {
			this.#errorDetails = this.#element.querySelector("[cewt-ref=\"error-details\"]:not(:scope [is] *)")!;
		}
		return this.#errorDetails;
	}
	#dismissBtn?: HTMLButtonElement;
	get dismissBtn() {
		if (this.#dismissBtn === undefined) {
			this.#dismissBtn = this.#element.querySelector("[cewt-ref=\"dismiss-btn\"]:not(:scope [is] *)")!;
		}
		return this.#dismissBtn;
	}
}
let _templateErrorModal: HTMLTemplateElement | null = null;
function getErrorModalTemplate(): HTMLTemplateElement {
	if (_templateErrorModal == null) {
		 _templateErrorModal = document.createElement("template")
		 _templateErrorModal.innerHTML = "\n\t<div class=\"title\">\n\t\t<p cewt-ref=\"heading\">undefined</p>\n\t</div>\n\t<div class=\"content\">\n\t\t<div class=\"message\">\n\t\t\t<img class=\"notio-icon-error\">\n\t\t\t<div>\n\t\t\t\t<p cewt-ref=\"message\">null</p>\n\t\t\t\t<textarea cewt-ref=\"error-details\" readonly=\"\"></textarea>\n\t\t\t</div>\n\t\t</div>\n\t\t<menu class=\"buttonRowRight\"><button cewt-ref=\"dismiss-btn\">OK</button></menu>\n\t</div>\n";
	}
	return _templateErrorModal;
}
export class ErrorModalAutogen extends HTMLDialogElement {
	readonly refs: ErrorModalRefs;
	static get observedAttributes() {
		return ["details", "heading", "message"];
	}
	#attributeDetailsValue: string | null = null;
	get details(): string | null {
		return this.#attributeDetailsValue;
	}
	set details(v: string | null) {
		if (v == null) {
			this.removeAttribute("details");
		}else{
			this.setAttribute("details", v);
		}
	}
	protected onDetailsChanged(oldValue: string | null, newValue: string | null) {
		// To be overridden by child class
	}
	#attributeHeadingValue: string | null = null;
	get heading(): string | null {
		return this.#attributeHeadingValue;
	}
	set heading(v: string | null) {
		if (v == null) {
			this.removeAttribute("heading");
		}else{
			this.setAttribute("heading", v);
		}
	}
	protected onHeadingChanged(oldValue: string | null, newValue: string | null) {
		// To be overridden by child class
	}
	#attributeMessageValue: string | null = null;
	get message(): string | null {
		return this.#attributeMessageValue;
	}
	set message(v: string | null) {
		if (v == null) {
			this.removeAttribute("message");
		}else{
			this.setAttribute("message", v);
		}
	}
	protected onMessageChanged(oldValue: string | null, newValue: string | null) {
		// To be overridden by child class
	}
	attributeChangedCallback(name: string, oldValue: string | null, newValue: string | null) {
		switch(name) {
			case "details":
				this.#attributeDetailsValue = newValue;
				this.onDetailsChanged(oldValue, newValue);
				break;
			case "heading":
				this.#attributeHeadingValue = newValue;
				this.onHeadingChanged(oldValue, newValue);
				break;
			case "message":
				this.#attributeMessageValue = newValue;
				this.onMessageChanged(oldValue, newValue);
				break;
			default:
				// Shouldn't happen
		}
	}
	constructor() {
		super();
		if (this.childElementCount == 0) {
			this.appendChild(
				getErrorModalTemplate()
					.content
					.cloneNode(true)
			);
		}
		this.setAttribute("is", "error-modal"); // allow for easy query selecting
		this.refs = new ErrorModalRefs(this);
		if (!this.getAttribute("class")) {
			this.setAttribute("class", "titledBox");
		}
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
		customElements.define("error-modal", this, { extends: "dialog"});
	}
}
export class TxConfirmedModalRefs {
	#element: HTMLElement | ShadowRoot;
	constructor(element: HTMLElement | ShadowRoot) {
		this.#element = element;
	}
	#txLink?: HTMLAnchorElement;
	get txLink() {
		if (this.#txLink === undefined) {
			this.#txLink = this.#element.querySelector("[cewt-ref=\"tx-link\"]:not(:scope [is] *)")!;
		}
		return this.#txLink;
	}
	#dismissBtn?: HTMLButtonElement;
	get dismissBtn() {
		if (this.#dismissBtn === undefined) {
			this.#dismissBtn = this.#element.querySelector("[cewt-ref=\"dismiss-btn\"]:not(:scope [is] *)")!;
		}
		return this.#dismissBtn;
	}
}
let _templateTxConfirmedModal: HTMLTemplateElement | null = null;
function getTxConfirmedModalTemplate(): HTMLTemplateElement {
	if (_templateTxConfirmedModal == null) {
		 _templateTxConfirmedModal = document.createElement("template")
		 _templateTxConfirmedModal.innerHTML = "\n\t<div class=\"title\">\n\t\t<p>Transaction confirmed</p>\n\t</div>\n\t<div class=\"content\">\n\t\t<div class=\"message\">\n\t\t\t<img class=\"notio-icon-ok\">\n\t\t\t<div>\n\t\t\t\t<p>Your transaction has been successfully processed.</p>\n\t\t\t\t<p><a href=\"https://www.seiscan.app/\" target=\"_blank\" cewt-ref=\"tx-link\">You may view its details here.</a></p>\n\t\t\t</div>\n\t\t</div>\n\t\t<menu class=\"buttonRowRight\"><button cewt-ref=\"dismiss-btn\">OK</button></menu>\n\t</div>\n";
	}
	return _templateTxConfirmedModal;
}
export class TxConfirmedModalAutogen extends HTMLDialogElement {
	readonly refs: TxConfirmedModalRefs;
	static get observedAttributes() {
		return ["chain", "txhash"];
	}
	#attributeChainValue: string | null = null;
	get chain(): string | null {
		return this.#attributeChainValue;
	}
	set chain(v: string | null) {
		if (v == null) {
			this.removeAttribute("chain");
		}else{
			this.setAttribute("chain", v);
		}
	}
	protected onChainChanged(oldValue: string | null, newValue: string | null) {
		// To be overridden by child class
	}
	#attributeTxhashValue: string | null = null;
	get txhash(): string | null {
		return this.#attributeTxhashValue;
	}
	set txhash(v: string | null) {
		if (v == null) {
			this.removeAttribute("txhash");
		}else{
			this.setAttribute("txhash", v);
		}
	}
	protected onTxhashChanged(oldValue: string | null, newValue: string | null) {
		// To be overridden by child class
	}
	attributeChangedCallback(name: string, oldValue: string | null, newValue: string | null) {
		switch(name) {
			case "chain":
				this.#attributeChainValue = newValue;
				this.onChainChanged(oldValue, newValue);
				break;
			case "txhash":
				this.#attributeTxhashValue = newValue;
				this.onTxhashChanged(oldValue, newValue);
				break;
			default:
				// Shouldn't happen
		}
	}
	constructor() {
		super();
		if (this.childElementCount == 0) {
			this.appendChild(
				getTxConfirmedModalTemplate()
					.content
					.cloneNode(true)
			);
		}
		this.setAttribute("is", "tx-confirmed-modal"); // allow for easy query selecting
		this.refs = new TxConfirmedModalRefs(this);
		if (!this.getAttribute("class")) {
			this.setAttribute("class", "titledBox");
		}
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
		customElements.define("tx-confirmed-modal", this, { extends: "dialog"});
	}
}
export class NewRewardsDenomModalRefs {
	#element: HTMLElement | ShadowRoot;
	constructor(element: HTMLElement | ShadowRoot) {
		this.#element = element;
	}
	#textInput?: HTMLInputElement;
	get textInput() {
		if (this.#textInput === undefined) {
			this.#textInput = this.#element.querySelector("[cewt-ref=\"text-input\"]:not(:scope [is] *)")!;
		}
		return this.#textInput;
	}
	#submitBtn?: HTMLButtonElement;
	get submitBtn() {
		if (this.#submitBtn === undefined) {
			this.#submitBtn = this.#element.querySelector("[cewt-ref=\"submit-btn\"]:not(:scope [is] *)")!;
		}
		return this.#submitBtn;
	}
}
let _templateNewRewardsDenomModal: HTMLTemplateElement | null = null;
function getNewRewardsDenomModalTemplate(): HTMLTemplateElement {
	if (_templateNewRewardsDenomModal == null) {
		 _templateNewRewardsDenomModal = document.createElement("template")
		 _templateNewRewardsDenomModal.innerHTML = "\n\t<div class=\"title\">\n\t\t<p>New rewards token</p>\n\t</div>\n\t<form class=\"content\" method=\"dialog\">\n\t\t<div class=\"message\">\n\t\t\t<img class=\"notio-icon-add\">\n\t\t\t<div>\n\t\t\t\t<p>Native tokens should be entered as is, and CW20 contracts should prefixed with \"cw20/\".</p>\n\t\t\t\t<input type=\"text\" cewt-ref=\"text-input\">\n\t\t\t</div>\n\t\t</div>\n\t\t<menu class=\"buttonRowRight\"><button value=\"\" cewt-ref=\"submit-btn\">OK</button><button value=\"\">Cancel</button></menu>\n\t</form>\n";
	}
	return _templateNewRewardsDenomModal;
}
export class NewRewardsDenomModalAutogen extends HTMLDialogElement {
	readonly refs: NewRewardsDenomModalRefs;
	constructor() {
		super();
		if (this.childElementCount == 0) {
			this.appendChild(
				getNewRewardsDenomModalTemplate()
					.content
					.cloneNode(true)
			);
		}
		this.setAttribute("is", "new-rewards-denom-modal"); // allow for easy query selecting
		this.refs = new NewRewardsDenomModalRefs(this);
		if (!this.getAttribute("class")) {
			this.setAttribute("class", "titledBox");
		}
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
		customElements.define("new-rewards-denom-modal", this, { extends: "dialog"});
	}
}