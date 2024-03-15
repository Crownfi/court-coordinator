// auto-generated by C.E.W.T.
// DO NOT EDIT BY HAND!!
export class TimerTextSlots {
	#element: HTMLElement;
	constructor(element: HTMLElement) {
		this.#element = element;
	}
}
export class TimerTextRefs {
	#element: HTMLElement | ShadowRoot;
	constructor(element: HTMLElement | ShadowRoot) {
		this.#element = element;
	}
}
let _templateTimerText: HTMLTemplateElement | null = null;
function getTimerTextTemplate(): HTMLTemplateElement {
	if (_templateTimerText == null) {
		 _templateTimerText = document.createElement("template")
		 _templateTimerText.innerHTML = "";
	}
	return _templateTimerText;
}
export class TimerTextAutogen extends HTMLElement {
	readonly slots: TimerTextSlots;
	readonly refs: TimerTextRefs;
	static get observedAttributes() {
		return ["end-timestamp"];
	}
	#attributeEndTimestampValue: string | null = null;
	get endTimestamp(): string | null {
		return this.#attributeEndTimestampValue;
	}
	set endTimestamp(v: string | null) {
		if (v == null) {
			this.removeAttribute("end-timestamp");
		}else{
			this.setAttribute("end-timestamp", v);
		}
	}
	protected onEndTimestampChanged(oldValue: string | null, newValue: string | null) {
		// To be overridden by child class
	}
	attributeChangedCallback(name: string, oldValue: string | null, newValue: string | null) {
		switch(name) {
			case "end-timestamp":
				this.#attributeEndTimestampValue = newValue;
				this.onEndTimestampChanged(oldValue, newValue);
				break;
			default:
				// Shouldn't happen
		}
	}
	constructor() {
		super();
		const shadowRoot = this.attachShadow({ mode: "closed" });
		shadowRoot.appendChild(
			getTimerTextTemplate()
				.content
				.cloneNode(true)
		);
		this.slots = new TimerTextSlots(this);
		this.refs = new TimerTextRefs(shadowRoot);
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
		customElements.define("timer-text", this);
	}
}