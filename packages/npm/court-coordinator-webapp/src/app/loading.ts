import { q } from "@aritz-cracker/browser-utils";

export function setLoading(isLoading: boolean, text: string = "Loading..."){
	const loadCover = q("#loading-cover") as HTMLElement;
	const loadingText = q("#loading-text") as (HTMLElement | null);
	// TODO: Block tab navigation
	// TOOD: Fade animation
	if (isLoading) {
		loadCover.hidden = false;
		loadCover.style.display = "";
		if (loadingText) {
			loadingText.innerText = text;
		}
	}else{
		loadCover.hidden = true;
		loadCover.style.display = "none";
		if (loadingText) {
			loadingText.innerText = "";
		}
	}
}
