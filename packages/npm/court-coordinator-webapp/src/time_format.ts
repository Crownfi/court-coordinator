
function maybePrependNegative(str: string, neg: boolean): string {
	if (neg) {
		return "-" + str;
	}
	return str;
}

function maybePrependIfNonZero(curVal: string, num: number, suffix: string): string {
	if (num == 0) {
		return curVal;
	}
	return num + suffix + curVal;
}

/**
 * Makes an amount of milliseconds more human-readable
 * 
 * @param timeAmount amount of time in milliseconds
 * @param showMs whether or not to show "ms" in the returned string
 */
export function humanReadableTimeAmount(timeAmount: number, showMs: boolean = false): string {
	const isNegative = timeAmount < 0;
	if (isNegative) {
		timeAmount *= -1;
	}
	let result = "";
	if (showMs) {
		if (timeAmount == 0) {
			return "0ms";
		}
		result = maybePrependIfNonZero(result, timeAmount % 1000, "ms");
	} else if (timeAmount < 1000) {
		return "0s";
	}
	timeAmount = Math.floor(timeAmount / 1000);
	if (timeAmount == 0) {
		return maybePrependNegative(result, isNegative);
	}
	
	result = maybePrependIfNonZero(result, timeAmount % 60, "s");
	timeAmount = Math.floor(timeAmount / 60);
	if (timeAmount == 0) {
		return maybePrependNegative(result, isNegative);
	}

	result = maybePrependIfNonZero(result, timeAmount % 60, "m");
	timeAmount = Math.floor(timeAmount / 60);
	if (timeAmount == 0) {
		return maybePrependNegative(result, isNegative);
	}

	result = maybePrependIfNonZero(result, timeAmount % 24, "h");
	timeAmount = Math.floor(timeAmount / 24);
	if (timeAmount == 0) {
		return maybePrependNegative(result, isNegative);
	}

	result = maybePrependIfNonZero(result, timeAmount % 7, "d");
	timeAmount = Math.floor(timeAmount / 7);
	if (timeAmount == 0) {
		return maybePrependNegative(result, isNegative);
	}

	result = maybePrependIfNonZero(result, timeAmount, "w");
	return maybePrependNegative(result, isNegative);
}

export function parseTimeAmount(timeAmount: string) {
	const match = timeAmount.match(/^\s*([-+])?\s*(?:(\d+)w)?\s*(?:(\d+)d)?\s*(?:(\d+)h)?\s*(?:(\d+)m)?\s*(?:(\d+)s)?\s*(?:(\d)+ms)?\s*$/);
	if (match == null) {
		return NaN;
	}
	const [_, sign, weeks, days, hours, minutes, seconds, milliseconds] = match;
	let result = 0;
	result += Number(weeks) || 0;
	result *= 7;
	result += Number(days) || 0;
	result *= 24;
	result += Number(hours) || 0;
	result *= 60;
	result += Number(minutes) || 0;
	result *= 60;
	result += Number(seconds) || 0;
	result *= 1000;
	result += Number(milliseconds) || 0;
	if (sign == "-") {
		result *= -1;
	}
	return result;
}
