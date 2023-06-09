import init, * as cxgraph from "./pkg/cxgraph_web.js";
await init();

let graphView = {
	xoff: 0,
	yoff: 0,
	scale: 3,
	res_mult: 1,
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
	cxgraph.resize(width*graphView.res_mult, height*graphView.res_mult);
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
	try {
		cxgraph.load_shader(src);
		div_error_msg.hidden = true;
		redraw();
	} catch(e) {
		console.log(e);
		div_error_msg.textContent = e.toString();
		div_error_msg.hidden = false;
	}
}

window.addEventListener("resize", onResize);
canvas.addEventListener("wheel", onWheel);
canvas.addEventListener("mousedown", onMouseDown);
canvas.addEventListener("mouseup", onMouseUp);
canvas.addEventListener("mousemove", onMouseMove);
button_graph.addEventListener("click", onGraph);

let class_decor = document.getElementsByClassName("decor")
for(let e of class_decor) {
	e.addEventListener("change", () => {
		let decor = 0;
		for(let elem of class_decor) {
			if(elem.checked) {
				decor += parseInt(elem.getAttribute("data-value"));
			}
		}
		cxgraph.set_decorations(decor);
		redraw();
	});
	e.checked = false;
}

let name_color_mode = document.getElementsByName("color_mode");
for(let e of name_color_mode) {
	e.addEventListener("change", () => {
		let selected = document.querySelector("input[name=color_mode]:checked");
		cxgraph.set_coloring(parseInt(selected.getAttribute("data-value")));
		redraw();
	});
	e.checked = false;
}
name_color_mode[0].checked = true;


range_shading.addEventListener("change", () => {
	let value = parseFloat(range_shading.value);
	cxgraph.set_shading_intensity(value);
	redraw();
});

range_contour.addEventListener("change", () => {
	let value = parseFloat(range_contour.value);
	cxgraph.set_contour_intensity(value);
	redraw();
});

range_resolution.addEventListener("change", () => {
	graphView.res_mult = Math.pow(2, parseFloat(range_resolution.value));
	onResize();
});

onResize();
onGraph();

// menu
