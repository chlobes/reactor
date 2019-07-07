mod utils;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct Reactor {
	fuel: f32,
	neutrons: f32,
	heat: f32,
	fuel_valve: bool
}

#[wasm_bindgen]
impl Reactor {
	pub fn new() -> Self {
		utils::set_panic_hook();
		Self {
			fuel: 1.4,
			neutrons: 0.0,
			heat: 1.0,
			fuel_valve: true,
		}
	}
	
	pub fn tick(&mut self, delta: f32) {
		if self.heat >= 2.0 || self.neutrons >= 3.0 || self.fuel >= 3.0 {
			self.fuel_valve = false;
		}
		let reaction_speed = self.neutrons * self.fuel * self.neutrons.min(self.fuel).powf(0.02);
		self.fuel += 0.05 * delta * (if self.fuel_valve { 1.0 } else { 0.1 } - reaction_speed);
		let heat_exchange_rate = self.neutrons;
		self.neutrons += delta * (2.0 * reaction_speed - 2.0 * heat_exchange_rate);
		self.heat += 0.02 * delta * (heat_exchange_rate - (0.5 + self.heat));
		self.heat = self.heat.max(0.0);
	}
	
	pub fn ignite(&mut self) {
		if self.neutrons < 0.001 && self.fuel > 1.25 {
			self.neutrons = 0.99;
			self.fuel -= 0.5;
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
}
