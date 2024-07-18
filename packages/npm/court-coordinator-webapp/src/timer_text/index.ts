import { humanReadableTimeAmount } from "../time_format.js";
import { TimerTextAutogen } from "./_autogen.js";

export class TimerTextElement extends TimerTextAutogen {
	#endTimestampAsNumber: number = NaN;
	#callbackFunctions: Set<Function> = new Set();
	#timerInterval: ReturnType<typeof setInterval> | undefined
	#think() {
		const timeRemaining = this.#endTimestampAsNumber - Date.now();
		if (timeRemaining > 0) {
			this.innerText = humanReadableTimeAmount(timeRemaining);
			return;
		}
		this.#stopInterval();
		this.#callbackFunctions.forEach(func => {
			func();
		})
	}
	#startInterval() {
		if (this.#timerInterval != undefined || isNaN(this.#endTimestampAsNumber)) {
			return;
		}
		this.#timerInterval = setInterval(() => {
			this.#think();
		}, 990);
		this.#think();
	}
	#stopInterval() {
		if (this.#timerInterval == undefined) {
			return;
		}
		clearInterval(this.#timerInterval);
		this.#timerInterval = undefined;
	}
	connectedCallback() {
		this.#startInterval();
	}
	disconnectedCallback() {
		this.#stopInterval();
	}
	protected onEndTimestampChanged(_: string | null, newValue: string | null) {
		this.#endTimestampAsNumber = Number(newValue);
		if (isNaN(this.#endTimestampAsNumber)) {
			this.innerText = "NaN";
			this.#stopInterval();
		}
		this.#startInterval(); // If the previous time has elapsed, we want to re-start the interval.
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
