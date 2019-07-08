import { Reactor, WaterTank } from "reactor";
import {$, jQuery } from "jquery";
//import roundSlider from "./roundslider.js";

const BLACK = "#000000";
const WHITE = "#FFFFFF";
const DARK_GREEN = "#0C3D10";
const DARK_GREY = "#828282";
const GREY = "#C0C0C0";
const LIGHT_GREY = "#D0D0D0";
const GLASS = "#C9DBDC";
const DULL_RED = "#954040";
const RED = "#FF0000";
const BLUE = "#0080FF";

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

const water_tank = WaterTank.new();
const water_tank_canvas = document.getElementById("water-tank-canvas");
water_tank_canvas.width = 300;
water_tank_canvas.height = 155;
const water_tank_ctx = water_tank_canvas.getContext('2d');
var water_tank_unlocked = false;

/*const water_tank_slider = roundSlider.CreateRoundSlider({
	radius: 80,
	circleShape: "quarter-top-right",
	sliderType: "min-range",
	showTooltip: false,
	value: 100
});*/ //TODO: figure out how javascript syntax works
const water_tank_slider	= document.getElementById("water-valve-slider");

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

const drawWaterTank = () => {
	if (water_tank.render()) {
		if (!water_tank_unlocked) {
			water_tank_unlocked = true;
			water_tank_slider.oninput = function() {
				water_tank.set_valve(this.value / 1000.0);
			};
			water_tank_slider.style.visibility = "visible";
			water_tank_slider.value = 10;
		}
		var heat = Math.min(water_tank.heat(), 2.0);
		water_tank_ctx.fillStyle = heat >= 1.0
			? blend(LIGHT_GREY, RED, 0.8 * (heat - 1.0))
			: blend(GREY, LIGHT_GREY, heat);
		water_tank_ctx.fillRect(0, 0, 300, 155);
		water_tank_ctx.fillStyle = blend(GLASS, RED, 0.6 * Math.max(heat - 1.0, 0.0));
		water_tank_ctx.fillRect(5, 0, 290, 150);
		water_tank_ctx.fillStyle = BLUE;
		water_tank_ctx.fillRect(5, 150, 290, -6*water_tank.water());
	}
};

const drawReactor = () => {
	if (reactor.render()) {
		var heat = Math.min(Math.log(1.1 * reactor.heat()+1.0), 2.0);
		reactor_ctx.fillStyle = heat >= 1.0
			? blend(GREY, RED, 0.7 * (heat - 1.0))
			: blend(DARK_GREY, GREY, heat);
		reactor_ctx.fillRect(0, 0, reactor_canvas.width, reactor_canvas.height);
		reactor_ctx.fillStyle = BLACK;
		reactor_ctx.fillRect(30, 5, 20, 150);
		reactor_ctx.fillRect(55, 35, 20, 120);
		reactor_ctx.fillStyle = DULL_RED;
		reactor_ctx.fillRect(55, 5, 20, 30);
		reactor_ctx.fillStyle = LIGHT_GREY;
		reactor_ctx.fillRect(30, 155, 20, -60*reactor.fuel());
		reactor_ctx.fillRect(55, 155, 20, -60*Math.min(reactor.neutrons(), 2.0));
		if (reactor.neutrons() > 2) {
			reactor_ctx.fillStyle = RED;
			reactor_ctx.fillRect(55, 35, 20, -30*Math.min(reactor.neutrons() - 2.0, 1.0));
		}
		reactor_ctx.strokeStyle = DARK_GREEN;
		reactor_ctx.beginPath();
		reactor_ctx.moveTo(30, 95);
		reactor_ctx.lineTo(75, 95);
		reactor_ctx.stroke();
		if (water_tank.render()) {
			reactor_ctx.fillStyle = BLACK;
			reactor_ctx.fillRect(5, 5, 20, 150);
			reactor_ctx.fillRect(5, 5, 20, 30);
			reactor_ctx.fillRect(5, 5, 20, 30);
			reactor_ctx.fillStyle = BLUE;
			reactor_ctx.fillRect(5, 155, 20, -60*reactor.water());
		}
	}
};

const renderLoop = () => {
	var delta = 1/60;
	
	reactor.tick(delta, water_tank);
	fuel_valve.textContent = reactor.fuel_valve() ? "placeholder💧" : "placeholder○";
	drawReactor();
	
	water_tank.tick(delta);
	drawWaterTank();
	
	requestAnimationFrame(renderLoop);
};

requestAnimationFrame(renderLoop);
