import { humanReadableTimeAmount } from "../time_format.js";
import { TimerTextAutogen } from "./_autogen.js";

export class TimerTextElement extends TimerTextAutogen {
	#endTimestampAsNumber: number = NaN;
	#callbackFunctions: Set<Function> = new Set();
	timerInterval: ReturnType<typeof setInterval> | undefined
	#renderTime() {
		this.innerText = humanReadableTimeAmount(Date.now() - this.#endTimestampAsNumber);
	}
	connectedCallback() {
		if (this.timerInterval != undefined) {
			return;
		}
		this.timerInterval = setInterval(() => {
			this.#renderTime();
		}, 990);
		this.#renderTime();
	}
	disconnectedCallback() {
		clearInterval(this.timerInterval);
		this.timerInterval = undefined;
	}
	protected onEndTimestampChanged(_: string | null, newValue: string | null) {
		this.#endTimestampAsNumber = Number(newValue);
		if (isNaN(this.#endTimestampAsNumber)) {
			
		}
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
