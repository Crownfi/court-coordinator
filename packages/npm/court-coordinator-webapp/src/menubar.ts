// Minimal port of norstone's menu bar. Used to make the mobile layout work. Does not support nested links.
import { parseCSSTime, q } from "@aritz-cracker/browser-utils";
const s_dropdownAnimationTimeout = Symbol("dropdownAnimationTimeout");

const mainMenu = q(".menubar-main-items") as HTMLElement;

let dropDownDomClass = "dropped-down";
let raiseUpDomClass = "raised-up";

const mobileMenuIcon = q("#menubar-mobile-icon") as HTMLElement;
mobileMenuIcon.addEventListener("click", (e) => {
	mobileMenuIcon.classList.toggle("on");
	setDropdownListContainerVisibility(mainMenu, mobileMenuIcon.classList.contains("on"));
	e.preventDefault();
});
export function setMenubarDropdownVisibilityClass(droppedDownClass: string, raisedUpClass: string){
	if(!droppedDownClass || !raisedUpClass){
		throw new Error("2 arguments expected");
	}
	dropDownDomClass = droppedDownClass;
	raiseUpDomClass = raisedUpClass;
}
function setDropdownListContainerVisibility(listContainer: HTMLElement, show = false){
	if((listContainer as any)[s_dropdownAnimationTimeout]){
		clearTimeout((listContainer as any)[s_dropdownAnimationTimeout]);
	}
	if(dropDownDomClass){
		listContainer.classList.toggle(dropDownDomClass, show);
		void listContainer.offsetWidth; // For some reason the animation doesn't work unless I do this
		listContainer.classList.toggle(raiseUpDomClass, !show);
	}
	const {animationDelay, animationDuration} = getComputedStyle(listContainer);
	if(show){
		listContainer.hidden = false;
	}else{
		(listContainer as any)[s_dropdownAnimationTimeout] = setTimeout(() => {
			listContainer.hidden = true;
			delete (listContainer as any)[s_dropdownAnimationTimeout];
		}, parseCSSTime(animationDelay) + parseCSSTime(animationDuration));
	}
}

document.body.addEventListener("click", (e) => {
	if(
		// Do not close dropdown menus if the user was clicking in them
		mainMenu.contains(e.target as Node | null)
	){
		return;
	}
	mobileMenuIcon.classList.remove("on")
	setDropdownListContainerVisibility(mainMenu, false);
});
window.addEventListener("resize", (e) => {
	if(window.innerWidth < 1000 && !mobileMenuIcon.classList.contains("on")){
		setDropdownListContainerVisibility(mainMenu, false);
	}else{
		setDropdownListContainerVisibility(mainMenu, true);
		mobileMenuIcon.classList.remove("on")
	}
});
