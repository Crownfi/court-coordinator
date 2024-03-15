import { NewRewardsDenomModalAutogen } from "./_autogen.js";

export class NewRewardsDenomModalElement extends NewRewardsDenomModalAutogen {
	untilClosed: Promise<string>;
	private untilCloseCallback: (result: string) => void;
	constructor() {
		super();
		this.untilCloseCallback = () => {}; // Gotta satisfy TS until 2 lines down
		this.untilClosed = new Promise(resolve => {
			this.untilCloseCallback = resolve;
		});
		this.addEventListener("close", (ev) => {
			this.untilCloseCallback(this.returnValue);
			this.untilClosed = new Promise(resolve => {
				this.untilCloseCallback = resolve;
			});
			this.remove();
		});
		this.refs.textInput.addEventListener("change", (ev) => {
			this.refs.submitBtn.value = this.refs.textInput.value;
		});
	}
	connectedCallback() {
		this.showModal();
	}
}
NewRewardsDenomModalElement.registerElement();
