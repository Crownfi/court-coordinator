import { seiUtilEventEmitter } from "@crownfi/sei-utils";
import { TxConfirmedModalAutogen } from "./_autogen.js";

// PopupModalAutogen extends HTMLDialogElement
class PopupModalElement extends TxConfirmedModalAutogen {
	untilClosed: Promise<void>;
	#untilCloseCallback: () => void;
	constructor(content?: {chain: string, txhash: string}){
		super();
		if (content) {
			// content will be undefined if the element was already added to the DOM before it was registered
			this.chain = content.chain;
			this.txhash = content.txhash;
		}
		this.#untilCloseCallback = () => {}; // Gotta satisfy TS until 2 lines down
		this.untilClosed = new Promise(resolve => {
			this.#untilCloseCallback = resolve;
		});
		this.addEventListener("close", (ev) => {
			this.remove();
		});
		this.refs.dismissBtn.onclick = () => {this.close()};
	}
	protected onChainChanged(oldValue: string | null, newValue: string | null) {
		this.#updateLink();
	}
	protected onTxhashChanged(oldValue: string | null, newValue: string | null) {
		this.#updateLink();
	}
	#updateLink() {
		this.refs.txLink.href =`https://www.seiscan.app/${
			encodeURIComponent(this.chain + "")
		}/txs/${
			encodeURIComponent(this.txhash + "")
		}`;
	}
	connectedCallback() {
		this.showModal();
	}
	disconnectedCallback() {
		this.#untilCloseCallback();
		this.untilClosed = new Promise(resolve => {
			this.#untilCloseCallback = resolve;
		});
	}
}
PopupModalElement.registerElement();
export function transactionConfirmedDialog(chain: string, txhash: string) {
	const newModal = new PopupModalElement({
		chain,
		txhash
	});
	document.body.appendChild(newModal);
}

seiUtilEventEmitter.on("transactionConfirmed", (ev) => {
	transactionConfirmedDialog(ev.chainId, ev.result.transactionHash);
});
