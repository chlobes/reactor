mod utils;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const FUEL_NEUTRON_RATIO: f32 = 40.0;
const REACTION_RATE: f32 = 0.05;
const NEUTRON_HEAT_RATIO: f32 = 0.04;
const HEAT_DISSIPATION_RATE: f32 = NEUTRON_HEAT_RATIO * 0.164;

#[wasm_bindgen]
pub struct Reactor {
	fuel: f32,
	neutrons: f32,
	heat: f32,
	fuel_valve: bool,
	vent: bool,
}

#[wasm_bindgen]
impl Reactor {
	pub fn new() -> Self {
		utils::set_panic_hook();
		Self {
			fuel: 1.01,
			neutrons: 1.0,
			heat: 0.0,
			fuel_valve: true,
			vent: false,
		}
	}
	
	pub fn tick(&mut self, delta: f32) {
		if self.heat >= 3.0 || self.neutrons >= 3.0 || self.fuel >= 2.5 {
			self.fuel_valve = false;
		}
		let reaction_speed = (REACTION_RATE * self.fuel * self.neutrons - 1e-4).max(0.0);
		let heat_exchange_rate = FUEL_NEUTRON_RATIO * REACTION_RATE * self.neutrons;
		self.fuel += delta * (if self.fuel_valve { 0.05 } else { -0.005 } + if self.vent { -0.25 } else { 0.0 } - reaction_speed);
		self.neutrons += delta * (FUEL_NEUTRON_RATIO * reaction_speed - heat_exchange_rate);
		self.heat += delta * (NEUTRON_HEAT_RATIO * heat_exchange_rate - HEAT_DISSIPATION_RATE * (3.0 + self.heat));
		self.fuel = self.fuel.max(0.0);
		if self.neutrons >= 3.0 {
			self.heat += NEUTRON_HEAT_RATIO * (self.neutrons - 3.0);
			self.neutrons = 3.0;
		}
		self.heat = self.heat.max(0.0);
	}
	
	pub fn ignite(&mut self) {
		if self.neutrons < 0.001 && self.fuel >= 1.0 + FUEL_NEUTRON_RATIO.recip() {
			self.neutrons = 0.99;
			self.fuel -= FUEL_NEUTRON_RATIO.recip();
		}
	}
	
	pub fn render(&self) -> bool {
		true
	}
	
	pub fn fuel(&self) -> f32 {
		self.fuel
	}
	
	pub fn neutrons(&self) -> f32 {
		self.neutrons
	}
	
	pub fn heat(&self) -> f32 {
		self.heat
	}
	
	pub fn fuel_valve(&self) -> bool {
		self.fuel_valve
	}
	
	pub fn toggle_fuel_valve(&mut self) {
		self.fuel_valve = !self.fuel_valve;
	}
	
	pub fn vent(&mut self, v: bool) {
		self.vent = v;
	}
}
