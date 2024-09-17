//
//  Keyboard handling
//

const TAB_WIDTH = 4;

let sourceFocused = false;

source_text.addEventListener("mousedown", () => { sourceFocused = true; });
source_text.addEventListener("focusout", () => { sourceFocused = false; });

source_text.addEventListener("keydown", (event) => {
	if (event.key != "Tab") {
		sourceFocused = true;
	}

	if (event.key == "Enter" && event.shiftKey) {
		event.preventDefault();
		button_graph.click();
	} else if (event.key == "Escape") {
		sourceFocused = false;
	} else if (event.key == "Backspace" && !event.ctrlKey) {
		let selStart = source_text.selectionStart;
		let selEnd = source_text.selectionEnd;

		if (selStart == selEnd) {
			let pre = source_text.value.slice(0, selStart);
			let lineStart = pre.lastIndexOf("\n") + 1;
			let preLine = pre.slice(lineStart);

			if (preLine.length > 2 && preLine.trim() == "") {
				let count = (selStart - lineStart - 1)%TAB_WIDTH + 1;
				for (let i = 0; i < count; i++) {
					if (pre[pre.length - i - 1] != " ") {
						count = i;
					}
				}

				let post = source_text.value.slice(selStart);
				source_text.value = pre.slice(0, selStart - count) + post;
				source_text.selectionStart = selStart - count;
				source_text.selectionEnd = selEnd - count;
				event.preventDefault();
			}
		}
	} else if (event.key == "Tab" && sourceFocused) {
		event.preventDefault();

		let selStart = source_text.selectionStart;
		let selEnd = source_text.selectionEnd;
		let pre = source_text.value.slice(0, selStart);
		let post = source_text.value.slice(selStart);
		let lineStart = pre.lastIndexOf("\n") + 1;

		if (event.shiftKey) {
			let count = (selStart - lineStart - 1)%TAB_WIDTH + 1;
			for (let i = 0; i < count; i++) {
				if (pre[pre.length - i - 1] != " ") {
					count = i;
				}
			}

			if (count > 0) {
				source_text.value = pre.slice(0, selStart - count) + post;
				source_text.selectionStart = selStart - count;
				source_text.selectionEnd = selEnd - count;
			}
		} else {
			let count = TAB_WIDTH - (selStart - lineStart)%TAB_WIDTH;
			source_text.value = pre + " ".repeat(count) + post;
			source_text.selectionStart = selStart + count;
			source_text.selectionEnd = selEnd + count;
		}
	}
});

//
//  Special characters
//

export let charMap = {
	"alpha":    "\u03b1",
	"beta":     "\u03b2",
	"gamma":    "\u03b3",
	"delta":    "\u03b4",
	"epsilon":  "\u03b5",
	"zeta":     "\u03b6",
	"eta":      "\u03b7",
	"theta":    "\u03b8",
	"iota":     "\u03b9",
	"kappa":    "\u03ba",
	"lambda":   "\u03bb",
	"mu":       "\u03bc",
	"nu":       "\u03bd",
	"xi":       "\u03be",
	"omicron":  "\u03bf",
	"pi":       "\u03c0",
	"rho":      "\u03c1",
	"fsigma":   "\u03c2",
	"sigma":    "\u03c3",
	"tau":      "\u03c4",
	"upsilon":  "\u03c5",
	"phi":      "\u03c6",
	"chi":      "\u03c7",
	"psi":      "\u03c8",
	"omega":    "\u03c9",
	"Alpha":    "\u0391",
	"Beta":     "\u0392",
	"Gamma":    "\u0393",
	"Delta":    "\u0394",
	"Epsilon":  "\u0395",
	"Zeta":     "\u0396",
	"Eta":      "\u0397",
	"Theta":    "\u0398",
	"Iota":     "\u0399",
	"Kappa":    "\u039a",
	"Lambda":   "\u039b",
	"Mu":       "\u039c",
	"Nu":       "\u039d",
	"Xi":       "\u039e",
	"Omicron":  "\u039f",
	"Pi":       "\u03a0",
	"Rho":      "\u03a1",
	"Sigma":    "\u03a3",
	"Tau":      "\u03a4",
	"Upsilon":  "\u03a5",
	"Phi":      "\u03a6",
	"Chi":      "\u03a7",
	"Psi":      "\u03a8",
	"Omega":    "\u03a9",
	"vartheta": "\u03d1",
	"0":        "\u2080",
	"1":        "\u2081",
	"2":        "\u2082",
	"3":        "\u2083",
	"4":        "\u2084",
	"5":        "\u2085",
	"6":        "\u2086",
	"7":        "\u2087",
	"8":        "\u2088",
	"9":        "\u2089",
};
let specialChars = new RegExp(
	`\\\\(${Object.keys(charMap).join("|")})`
);

source_text.addEventListener("input", (event) => {
	if(event.isComposing) return;
	let e = source_text.selectionEnd;
	let amnt = 0;
	source_text.value = source_text.value.replace(
		specialChars,
		(m, p) => {
			amnt += m.length - charMap[p].length;
			return charMap[p];
		}
	);
	source_text.selectionEnd = e - amnt;
});

source_text.addEventListener("change", () => {
	localStorage.setItem("editor_content", source_text.value);
});

if (localStorage.getItem("editor_content") !== null) {
	source_text.value = localStorage.getItem("editor_content");
} else {
	source_text.value = "f(z) = 6z^2 - 2i - 1\nplot(z) = f(1 + sin(z)) / 8";
}
