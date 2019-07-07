import { Reactor } from "reactor";

const BLACK = "#000000";
const WHITE = "#FFFFFF";
const DARK_GREEN = "#0C3D10";
const DARK_GREY = "#828282";
const GREY = "#C0C0C0";
const LIGHT_GREY = "#E0E0E0";
const DULL_RED = "#954040";
const RED = "#FF0000";

function blend(a, b, ratio) {
	a = [parseInt(a[1] + a[2], 16), parseInt(a[3] + a[4], 16), parseInt(a[5] + a[6], 16)];
	b = [parseInt(b[1] + b[2], 16), parseInt(b[3] + b[4], 16), parseInt(b[5] + b[6], 16)];
	var c = [ 
		(1 - ratio) * a[0] + ratio * b[0], 
		(1 - ratio) * a[1] + ratio * b[1], 
		(1 - ratio) * a[2] + ratio * b[2]
	];
	return '#' + int_to_hex(c[0]) + int_to_hex(c[1]) + int_to_hex(c[2]); 
}

function int_to_hex(num) {
	var hex = Math.round(num).toString(16);
	if (hex.length == 1)
		hex = '0' + hex;
	return hex;
}

const reactor = Reactor.new();
const reactor_canvas = document.getElementById("reactor-canvas");
reactor_canvas.width = 300;
reactor_canvas.height = 160;
const reactor_ctx = reactor_canvas.getContext('2d');

const fuel_valve = document.getElementById("fuel-valve");
fuel_valve.addEventListener("click", event => {
	reactor.toggle_fuel_valve();
});

const ignite_button = document.getElementById("ignite");
ignite_button.addEventListener("click", event => {
	reactor.ignite();
});

const vent_button = document.getElementById("vent");
var venting = false;
vent_button.addEventListener("click", event => {
	venting = !venting;
	if (venting) {
		vent_button.textContent = "☑ ";
	} else {
		vent_button.textContent = "☐";
	}
	reactor.vent(venting);
});

const drawReactor = () => {
	var heat = Math.min(Math.log(1.1 * reactor.heat()+1.0), 2.0);
	reactor_ctx.fillStyle = heat >= 1.0
		? blend(GREY, RED, 0.7 * (heat - 1.0))
		: blend(DARK_GREY, GREY, heat);
	reactor_ctx.fillRect(0, 0, reactor_canvas.width, reactor_canvas.height);
	reactor_ctx.fillStyle = BLACK;
	reactor_ctx.fillRect(5, 5, 20, 150);
	reactor_ctx.fillRect(30, 35, 20, 120);
	//reactor_ctx.fillRect(55, 35, 20, 120);
	reactor_ctx.fillStyle = DULL_RED;
	//reactor_ctx.fillRect(5, 5, 20, 30);
	reactor_ctx.fillRect(30, 5, 20, 30);
	//reactor_ctx.fillRect(55, 5, 20, 30);
	reactor_ctx.fillStyle = LIGHT_GREY;
	reactor_ctx.fillRect(5, 155, 20, -60*reactor.fuel());
	reactor_ctx.fillRect(30, 155, 20, -60*Math.min(reactor.neutrons(), 2.0));
	if (reactor.neutrons() > 2) {
		reactor_ctx.fillStyle = RED;
		reactor_ctx.fillRect(30, 35, 20, -30*Math.min(reactor.neutrons() - 2.0, 1.0));
	}
	reactor_ctx.strokeStyle = DARK_GREEN;
	reactor_ctx.beginPath();
	reactor_ctx.moveTo(5, 95);
	reactor_ctx.lineTo(50, 95);
	reactor_ctx.stroke();
};

const renderLoop = () => {
	var delta = 1/60;
	reactor.tick(delta);
	fuel_valve.textContent = reactor.fuel_valve() ? "💧" : "○";
	drawReactor();
	
	requestAnimationFrame(renderLoop);
};

requestAnimationFrame(renderLoop);
