function themeChange() {
	if (checkbox_theme.checked) {
		body.classList.remove("theme_light");
		body.classList.add("theme_dark");
	} else {
		body.classList.remove("theme_dark");
		body.classList.add("theme_light");
	}
	localStorage.setItem("theme", checkbox_theme.checked ? "dark" : "light");
}

checkbox_theme.addEventListener("change", themeChange);

if (localStorage.getItem("theme") !== null) {
	if (localStorage.getItem("theme") == "light") {
		checkbox_theme.checked = false;
	} else {
		checkbox_theme.checked = true;
	}
}
themeChange();
