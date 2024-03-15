import { TimerTextAutogen } from "./_autogen.js";

export class TimerTextElement extends TimerTextAutogen {
	#endTimestampAsNumber: number = NaN;
	#callbackFunctions: Set<Function> = new Set();
	timerInterval: ReturnType<typeof setInterval> | undefined
	#renderTime() {
		let diff = Math.round((Date.now() - this.#endTimestampAsNumber) / 1000);
		if (isNaN(diff)) {
			this.innerText = "NaN";
			return;
		}else if (diff <= 0) {
			this.innerText = "0s";
			this.disconnectedCallback(); // clear the timer
			this.#callbackFunctions.forEach(v => v());
			return;
		}
		this.innerText = (diff % 60) + "s";
		diff = Math.floor(diff / 60);
		if (diff <= 0) {
			return;
		}
		this.innerText = (diff % 60) + "m" + this.innerText;
		diff = Math.floor(diff / 60);
		if (diff <= 0) {
			return;
		}
		this.innerText = (diff % 24) + "h" + this.innerText;
		diff = Math.floor(diff / 24);
		if (diff <= 0) {
			return;
		}
		this.innerText = (diff % 7) + "d" + this.innerText;
		diff = Math.floor(diff / 7);
		if (diff <= 0) {
			return;
		}
		this.innerText = diff + "w" + this.innerText;
	}
	connectedCallback() {
		if (this.timerInterval != undefined) {
			return;
		}
		this.timerInterval = setInterval(() => {
			this.#renderTime();
		}, 999);
		this.#renderTime();
	}
	disconnectedCallback() {
		clearInterval(this.timerInterval);
		this.timerInterval = undefined;
	}
	protected onEndTimestampChanged(_: string | null, newValue: string | null) {
		this.#endTimestampAsNumber = Number(newValue);
		this.connectedCallback(); // If the previous time has elapsed, we want to re-start the interval.
	}
	/**
	 * If you want to know when the timer reaches 0, this is where to do it.
	 * 
	 * Note:
	 *   * The callback _won't_ be called if this element has been removed from the DOM
	 *   * The callback _will_ be called multiple times if this element is re-used for multiple end-times.
	 * @param func the function to call when the timer reaches 0
	 */
	addTimerCallback(func: Function) {
		this.#callbackFunctions.add(func);
	}
	removeTimerCallback(func: Function) {
		this.#callbackFunctions.delete(func);
	}
	clearTimerCallbacks() {
		this.#callbackFunctions.clear();
	}
}
TimerTextElement.registerElement();
