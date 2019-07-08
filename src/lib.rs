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
	water: f32,
	fuel_valve: bool,
	vent: bool,
}

#[wasm_bindgen]
impl Reactor {
	pub fn new() -> Self {
		utils::set_panic_hook();
		Self {
			fuel: 0.7,
			neutrons: 0.0,
			heat: 0.0,
			water: 0.0,
			fuel_valve: true,
			vent: false,
		}
	}
	
	pub fn tick(&mut self, delta: f32, water_tank: &mut WaterTank) {
		if self.heat >= 3.0 || self.neutrons >= 3.0 || self.fuel >= 2.5 {
			self.fuel_valve = false;
		}
		if self.heat >= 3.0 {
			water_tank.unlocked = true;
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
		if water_tank.unlocked {
			let amount = delta * water_tank.valve * water_tank.water * (2.5 - self.water); //from water tank
			self.heat = ((10.0 + self.water) * self.heat + amount * water_tank.heat) / (10.0 + self.water + amount);
			self.water += amount;
			water_tank.water -= amount;
			let amount = delta * self.water * 0.1 * (25.0 - water_tank.water); //to water tank
			self.water -= amount;
			water_tank.add_water(amount, self.heat);
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
	
	pub fn water(&self) -> f32 {
		self.water
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

#[wasm_bindgen]
pub struct WaterTank {
	unlocked: bool,
	water: f32,
	heat: f32,
	valve: f32,
}

impl WaterTank {
	fn add_water(&mut self, amount: f32, heat: f32) {
		self.heat = ((self.water + 1.0) * self.heat + amount * heat) / (self.water + 1.0 + amount);
		self.water += amount;
	}
}

#[wasm_bindgen]
impl WaterTank {
	pub fn new() -> Self {
		Self {
			unlocked: false,
			water: 20.0,
			heat: 0.0,
			valve: 0.01,
		}
	}
	
	pub fn tick(&mut self, delta: f32) {
		if self.unlocked {
			let amount = (delta * 0.005).min(25.0 - self.water);
			self.add_water(amount, 0.0);
			self.water -= delta * 0.01 * self.water.sqrt() * (self.heat.exp() - 1.0); //evaporation
			self.heat -= delta * 0.00001 * (self.water.sqrt() + 1.0); //heat dissipation
			self.heat = self.heat.max(0.0);
		}
	}
	
	pub fn water(&self) -> f32 {
		self.water
	}
	
	pub fn heat(&self) -> f32 {
		self.heat
	}
	
	pub fn render(&self) -> bool {
		self.unlocked
	}
	
	pub fn set_valve(&mut self, v: f32) {
		self.valve = v;
	}
}
