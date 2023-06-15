"use strict"

import init, * as cxgraph from "./pkg/cxgraph_web.js";
await init();

let graphView = {
	xoff: 0,
	yoff: 0,
	scale: 3,
	res_mult: 1,
	varNames: [],
};

let graphPoints = [];
let graphSliders = [];

let mouseX = 0.0;
let mouseY = 0.0;
let mousePressed = false;

function remap(x, lo1, hi1, lo2, hi2) {
	return lo2 + (hi2 - lo2) * (x - lo1) / (hi1 - lo1);
}

function cxToScreen(cx) {
	let [sc, sca] = [graphView.scale, graphView.scale * (window.innerWidth / window.innerHeight)];
	return {
		x: remap(cx.re - graphView.xoff, -sca, sca, 0, window.innerWidth),
		y: remap(cx.im + graphView.yoff, -sc,  sc,  0, window.innerHeight),
	}
}


function screenToCx(screen) {
	let [sc, sca] = [graphView.scale, graphView.scale * (window.innerWidth / window.innerHeight)];
	return {
		re:  graphView.xoff + remap(screen.x, 0, window.innerWidth,  -sca, sca),
		im: -graphView.yoff + remap(screen.y, 0, window.innerHeight, -sc,   sc),
	}
}

//
// Canvas
//

function redraw() {
	cxgraph.redraw();
}

function tryRedraw() {
	if(checkbox_autoredraw.checked) {
		redraw();
	}
}

function calcBounds() {
	let width = window.innerWidth;
	let height = window.innerHeight;
	let aspect = width / height;
	return {
		x_min: graphView.xoff - graphView.scale * aspect,
		y_min: graphView.yoff - graphView.scale,
		x_max: graphView.xoff + graphView.scale * aspect,
		y_max: graphView.yoff + graphView.scale,
	};
}

function onViewChange() {
	let dim = { w: window.innerWidth, h: innerHeight };
	let bounds = calcBounds();
	cxgraph.set_bounds(bounds.x_min, bounds.y_min, bounds.x_max, bounds.y_max);
	let origin = cxToScreen({ re: 0, im: 0 });
	let one = cxToScreen({ re: 1, im: 0 });

	if(svg_axis_x.visibility != "hidden") {
		svg_axis_x.setAttribute("x1", 0)
		svg_axis_x.setAttribute("x2", dim.w);
		svg_axis_x.setAttribute("y1", origin.y);
		svg_axis_x.setAttribute("y2", origin.y);
	}
	if(svg_axis_y.visibility != "hidden") {
		svg_axis_y.setAttribute("x1", origin.x);
		svg_axis_y.setAttribute("x2", origin.x);
		svg_axis_y.setAttribute("y1", 0);
		svg_axis_y.setAttribute("y2", dim.h);
	}
	if(svg_unitcircle.visibility != "hidden") {
		svg_unitcircle.setAttribute("cx", origin.x);
		svg_unitcircle.setAttribute("cy", origin.y);
		svg_unitcircle.setAttribute("r", one.x - origin.x);
	}

	for(let point of graphPoints) {
		point.onViewChange();
	}

	tryRedraw();
}

function onResize() {
	let width = window.innerWidth;
	let height = window.innerHeight;
	cxgraph.resize(width*graphView.res_mult, height*graphView.res_mult);
	canvas.style.width = "100vw";
	canvas.style.height = "100vh";
	onViewChange();
}

function onWheel(e) {
	graphView.scale *= Math.exp(e.deltaY * 0.0007);
	onViewChange();
}

function onMouseDown(e) {
	mousePressed = true;
	mouseX = e.offsetX;
	mouseY = e.offsetY;
}

function onMouseUp(e) {
	mousePressed = false;
	for(let point of graphPoints) {
		point.mousePressed = false;
	}
}

function onMouseMove(e) {
	if(mousePressed) {
		let dX = e.offsetX - mouseX;
		let dY = e.offsetY - mouseY;
		mouseX = e.offsetX;
		mouseY = e.offsetY;
		graphView.xoff -= 2.0 * graphView.scale * dX / window.innerHeight;
		graphView.yoff += 2.0 * graphView.scale * dY / window.innerHeight;
		onViewChange();
	} else {
		for(let point of graphPoints) {
			point.onMouseMove(e);
		}
	}
}

window.addEventListener("resize", onResize);
canvas.addEventListener("wheel", onWheel);
canvas.addEventListener("mousedown", onMouseDown);
canvas.addEventListener("mouseup", onMouseUp);
canvas.addEventListener("mousemove", onMouseMove);

//
// Graph/redraw
//

function onGraph() {
	let src = source_text.value;
	try {
		cxgraph.load_shader(src, graphView.varNames);
		div_error_msg.hidden = true;
		redraw();
	} catch(e) {
		console.log(e);
		div_error_msg.textContent = e.toString().replace("\n", "\r\n");
		div_error_msg.hidden = false;
	}
}

button_graph.addEventListener("click", onGraph);
button_redraw.addEventListener("click", redraw);

let charMap = {
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
};
let specialChars = new RegExp(
	`\\\\(${Object.keys(charMap).join("|")})`
);
console.log(specialChars);

source_text.addEventListener("input", () => {
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

//
// Options
//

button_reset_view.addEventListener("click", () => {
	graphView.xoff = 0;
	graphView.yoff = 0;
	graphView.scale = 3;
	onViewChange();
})

range_shading.addEventListener("change", () => {
	let value = parseFloat(range_shading.value);
	cxgraph.set_shading_intensity(value);
	tryRedraw();
});

range_contour.addEventListener("change", () => {
	let value = parseFloat(range_contour.value);
	cxgraph.set_contour_intensity(value);
	tryRedraw();
});

range_resolution.addEventListener("change", () => {
	graphView.res_mult = Math.pow(2, parseFloat(range_resolution.value));
	onResize();
});

let classDecor = document.getElementsByClassName("decor")
for(let e of classDecor) {
	e.addEventListener("change", () => {
		let decor = 0;
		for(let elem of classDecor) {
			if(elem.checked) {
				decor += parseInt(elem.getAttribute("data-value"));
			}
		}
		cxgraph.set_decorations(decor);
		tryRedraw();
	});
	e.checked = false;
}

let nameColorMode = document.getElementsByName("color_mode");
for(let e of nameColorMode) {
	e.addEventListener("change", () => {
		let selected = document.querySelector("input[name=color_mode]:checked");
		cxgraph.set_coloring(parseInt(selected.getAttribute("data-value")));
		tryRedraw();
	});
	e.checked = false;
}
nameColorMode[0].checked = true;

overlay_axes.addEventListener("change", () => {
	let vis = overlay_axes.checked ? "visible" : "hidden";
	svg_axis_x.setAttribute("visibility", vis);
	svg_axis_y.setAttribute("visibility", vis);
});

overlay_unitcircle.addEventListener("change", () => {
	let vis = overlay_unitcircle.checked ? "visible" : "hidden";
	svg_unitcircle.setAttribute("visibility", vis);
});

//
// Variables
//

let nextVarId = 0;
let varCount = 0;

function genVarData() {
	varCount = 0;
	for(let child of div_variables.children) {
		if(child.id.startsWith("slider")) {
			let value = parseFloat(child.querySelector(".var-value").value) || 0;
			cxgraph.set_variable(varCount, value, 0);
			varCount++;
		} else if(child.id.startsWith("point")) {
			let re = parseFloat(child.querySelector(".var-value-re").value) || 0;
			let im = parseFloat(child.querySelector(".var-value-im").value) || 0;
			cxgraph.set_variable(varCount, re, im);
			varCount++;
		}
	}
}

function genVarNames() {
	graphView.varNames = [];
	varCount = 0;
	for(let child of div_variables.children) {
		if(child.id.startsWith("slider")) {
			let name = child.querySelector(".var-name").value || "";
			graphView.varNames.push(name);
			varCount++;
		} else if(child.id.startsWith("point")) {
			let name = child.querySelector(".var-name").value || "";
			graphView.varNames.push(name);
			varCount++;
		}
	}
	genVarData();
	onGraph();
}

function addSlider() {
	if(varCount >= 8) {
		return;
	}
	let newSlider = slider_template.cloneNode(true);
	let id = nextVarId++;
	newSlider.id = "slider_" + id;
	newSlider.hidden = false;
	div_variables.appendChild(newSlider);
	newSlider.querySelector(".var-name").addEventListener("change", genVarNames);
	newSlider.querySelector(".var-delete").addEventListener("click", () => {
		document.getElementById("slider_" + id).remove()
		genVarNames();
	});
	newSlider.querySelector(".var-min").addEventListener("input", (e) => {
		newSlider.querySelector(".var-slider").min = e.target.value;
		genVarData();
		tryRedraw();
	});
	newSlider.querySelector(".var-max").addEventListener("input", (e) => {
		newSlider.querySelector(".var-slider").max = e.target.value
		genVarData();
		tryRedraw();
	});
	newSlider.querySelector(".var-step").addEventListener("input", (e) => {
		newSlider.querySelector(".var-slider").step = e.target.value;
		genVarData();
		tryRedraw();
	});
	newSlider.querySelector(".var-slider").addEventListener("input", (e) => {
		newSlider.querySelector(".var-value").value = e.target.value
		genVarData();
		tryRedraw();
	});
	newSlider.querySelector(".var-value").addEventListener("input", (e) => {
		newSlider.querySelector(".var-slider").value = e.target.value
		genVarData();
		tryRedraw();
	});
	genVarNames();
}

class Point {
	constructor(id) {
		this.id = id;

		let menuPoint = point_template.cloneNode(true);
		this.menuPoint = menuPoint;
		menuPoint.id = "point_" + id;
		menuPoint.hidden = false;
		div_variables.appendChild(menuPoint);

		let svgPoint = svg_point_template.cloneNode(true);
		this.svgPoint = svgPoint;
		svgPoint.id = "svg_point_" + id;
		svgPoint.setAttribute("visibility", "");
		svgPoint.setAttribute("data-id", menuPoint.id);
		overlay_points.appendChild(svgPoint);

		this.mousePressed = false;

		menuPoint.querySelector(".var-name").addEventListener("change", genVarNames);

		menuPoint.querySelector(".var-delete").addEventListener("click", () => this.destroy());

		menuPoint.querySelector(".var-value-re").addEventListener("input", () => {
			this.onViewChange();
			genVarData();
			tryRedraw();
		});

		menuPoint.querySelector(".var-value-im").addEventListener("input", () => {
			this.onViewChange();
			genVarData();
			tryRedraw();
		});

		svgPoint.addEventListener("mousedown", (e) => {
			this.mousePressed = true;
			mouseX = e.offsetX;
			mouseY = e.offsetY;
		});

		svgPoint.addEventListener("mouseup", () => {
			this.mousePressed = false;
			mousePressed = false;
		});

		svgPoint.addEventListener("mousemove", (e) => this.onMouseMove(e));

		this.onViewChange();
		genVarNames();
	}

	onMouseMove(e) {
		if(this.mousePressed) {
			mouseX = e.offsetX;
			mouseY = e.offsetY;
			let cx = screenToCx({ x: mouseX, y: mouseY });
			this.menuPoint.querySelector(".var-value-re").value = cx.re;
			this.menuPoint.querySelector(".var-value-im").value = -cx.im;
			this.onViewChange();
			genVarData();
			redraw();
		}
	}

	onViewChange() {
		let re = parseFloat(this.menuPoint.querySelector(".var-value-re").value) || 0;
		let im = parseFloat(this.menuPoint.querySelector(".var-value-im").value) || 0;
		let screen = cxToScreen({ re: re, im: -im });
		this.svgPoint.setAttribute("transform", `translate(${screen.x} ${screen.y})`);
	}

	destroy() {
		this.menuPoint.remove();
		this.svgPoint.remove();
		graphPoints = graphPoints.filter(p => p != this);
		genVarNames();
	}
}

function addPoint() {
	if(varCount >= 8) {
		return;
	}
	graphPoints.push(new Point(nextVarId++));
}

button_slider_new.addEventListener("click", addSlider);
button_point_new.addEventListener("click", addPoint);

//
// Init
//

onResize();
onGraph();

// Debug

export function show_ast() {
	console.info(cxgraph.show_shader_ast(source_text.value));
}
