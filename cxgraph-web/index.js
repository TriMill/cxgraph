menu_checkbox.addEventListener("change", () => {
	menu_inner.hidden = !menu_checkbox.checked;
});

import init, * as cxgraph from "./pkg/cxgraph_web.js";
await init();

let graphView = {
	xoff: 0,
	yoff: 0,
	scale: 3,
};

function redraw() {
	cxgraph.redraw();
}

function onViewChange() {
	let width = window.innerWidth;
	let height = window.innerHeight;
	let aspect = width / height;
	cxgraph.set_bounds(
		graphView.xoff - graphView.scale * aspect,
		graphView.yoff - graphView.scale,
		graphView.xoff + graphView.scale * aspect,
		graphView.yoff + graphView.scale
	);
	redraw();
}

function onResize() {
	let width = window.innerWidth;
	let height = window.innerHeight;
	cxgraph.resize(width, height);
	grid_canvas.width = width;
	grid_canvas.height = height;
	canvas.style.width = "100vw";
	canvas.style.height = "100vh";
	onViewChange();
}

let mouseX = 0.0;
let mouseY = 0.0;
let mousePressed = false;

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
	}
}

function onGraph() {
	let src = document.getElementById("source_text").value;
	cxgraph.load_shader(src);
	redraw();
}

window.addEventListener("resize", onResize);
canvas.addEventListener("wheel", onWheel);
canvas.addEventListener("mousedown", onMouseDown);
canvas.addEventListener("mouseup", onMouseUp);
canvas.addEventListener("mousemove", onMouseMove);
button_graph.addEventListener("click", onGraph);

onResize();
onGraph();

// menu
