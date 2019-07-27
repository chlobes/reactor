use crate::vertex::*;
use crate::prelude::*;

mod collision;
use collision::*;

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const WHITE: [f32; 4] = [1.0; 4];
const DARK_GREY: [f32; 4] = [0.5, 0.5, 0.5, 1.0];
const GREY: [f32; 4] = [0.75, 0.75, 0.75, 1.0];
const LIGHT_GREY: [f32; 4] = [0.81, 0.81, 0.81, 1.0];
const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const DULL_RED: [f32; 4] = [0.59, 0.25, 0.25, 1.0];
const DARK_GREEN: [f32; 4] = [0.05, 0.24, 0.06, 1.0];
const DARK_GREEN_TRANSPARENT: [f32; 4] = [DARK_GREEN[0], DARK_GREEN[1], DARK_GREEN[2], 0.75];
const YELLOW: [f32; 4] = [0.93, 0.87, 0.47, 1.0];
const BLUE: [f32; 4] = [0.0, 0.5, 1.0, 1.0];
const GREEN: [f32; 4] = [0.33, 0.8, 0.2, 1.0];
const GLASS: [f32; 4] = [0.79, 0.85, 0.86, 1.0];

fn blend(a: [f32; 4], b: [f32; 4], ratio: f32) -> [f32; 4] {
	[
		(1.0 - ratio) * a[0] + ratio * b[0],
		(1.0 - ratio) * a[1] + ratio * b[1],
		(1.0 - ratio) * a[2] + ratio * b[2],
		(1.0 - ratio) * a[3] + ratio * b[3],
	]
}

const DT: f32 = 0.02;
const FUEL_NEUTRON_RATIO: f32 = 20.0;
const FUEL_WASTE_RATIO: f32 = 0.97;
const REACTION_RATE: f32 = 1.0 / FUEL_NEUTRON_RATIO;
const NEUTRON_HEAT_RATIO: f32 = 5.0;
const HEAT_DISSIPATION_RATE: f32 = NEUTRON_HEAT_RATIO * 0.8;
const WATER_PRESSURE: f32 = 3.5;
const WATER_REFILL_RATE: f32 = NEUTRON_HEAT_RATIO * 0.02;
const WATER_TRANSFER_RATE: f32 = WATER_REFILL_RATE * 100.0;
const REACTOR_MASS: f32 = 10.0;
const EVAPORATION_RATE: f32 = WATER_REFILL_RATE * 0.8;
const REFINERY_FUEL_FLOW_RATE: f32 = 0.1;
const REFINERY_NEUTRON_FLOW_RATE: f32 = REFINERY_NEUTRON_DECAY_RATE * REFINERY_NEUTRON_CAP;
const REFINERY_WASTE_FLOW_RATE: f32 = 1.0;

pub struct Reactor {
	fuel: f32,
	neutrons: f32,
	heat: f32,
	water: f32,
	waste: f32,
	fuel_valve: [bool; 3],
	vent: bool,
	water_tank: WaterTank,
	refinery: Refinery,
}

impl Reactor {
	pub fn new() -> Self {
		Self {
			fuel: 0.9,
			neutrons: 0.0,
			heat: 0.0,
			water: 0.0,
			waste: 1.0,
			fuel_valve: [true, false, false],
			vent: false,
			water_tank: WaterTank::new(),
			refinery: Refinery::new(),
		}
	}
	
	pub fn tick(&mut self) {
		if self.heat >= 3.0 || self.neutrons >= 3.0 {
			self.fuel_valve[1] = true;
		} else if self.heat < 1.2 {
			self.fuel_valve[1] = false;
		}
		if self.heat >= 3.0 {
			self.water_tank.unlocked = true;
		}
		if self.waste >= 5.0 {
			self.refinery.unlocked = true;
		}
		let reaction_speed = (REACTION_RATE * self.fuel * self.neutrons.powf(1.02) - 0.001).max(0.0).min(self.fuel / DT);
		let heat_exchange_rate = FUEL_NEUTRON_RATIO * REACTION_RATE * self.neutrons;
		self.fuel += DT * (REACTION_RATE * (if self.fuel_valve() { 1.0 } else { 0.0 } + if self.vent { -5.0 } else { 0.0 }) - reaction_speed);
		self.waste += DT * FUEL_WASTE_RATIO * reaction_speed;
		self.neutrons += DT * (FUEL_NEUTRON_RATIO * reaction_speed - heat_exchange_rate);
		self.heat += DT * (NEUTRON_HEAT_RATIO * heat_exchange_rate - HEAT_DISSIPATION_RATE * if self.heat > 4.0 { self.heat / 4.0 } else { 1.0 }) / (REACTOR_MASS + self.water);
		self.fuel = self.fuel.max(0.0).min(if self.water_tank.unlocked { 3.0 } else { 1.1 });
		self.waste = self.waste.max(0.0).min(10.0);
		if !self.fuel_valve_unlocked() {
			if self.fuel == 0.0 {
				self.fuel_valve[2] = true;
			}
		}
		if self.neutrons >= 3.0 {
			self.heat += NEUTRON_HEAT_RATIO * (self.neutrons - 3.0);
			self.neutrons = 3.0;
		}
		if self.water_tank.unlocked {
			let amount = DT * WATER_TRANSFER_RATE * (self.water / 2.5).powf(WATER_PRESSURE); //to water tank
			self.water -= amount;
			self.water_tank.add_water(amount, self.heat);
			let amount = DT * self.water_tank.valve.val * WATER_TRANSFER_RATE * (self.water_tank.water / 25.0).powf(WATER_PRESSURE); //from water tank
			self.heat = ((REACTOR_MASS + self.water) * self.heat + amount * self.water_tank.heat) / (REACTOR_MASS + self.water + amount);
			self.water += amount;
			self.water_tank.water -= amount;
			self.water_tank.tick();
		}
		if self.refinery.unlocked {
			let val = (self.refinery.fuel_valve.val - 0.5) * 2.0;
			let f = if val > 0.0 { //refinery to reactor
				(REFINERY_FUEL_FLOW_RATE * val.powi(2) * self.fuel * (REFINERY_FUEL_CAP - self.refinery.fuel) / REFINERY_FUEL_CAP)
					.min(self.fuel.min(REFINERY_FUEL_CAP - self.refinery.fuel) / DT)
			} else { //reactor to refinery
				(REFINERY_FUEL_FLOW_RATE * val.abs().powi(2) * -1.0 * self.refinery.fuel * (3.0 - self.fuel) / 3.0)
					.min(self.refinery.fuel.min(3.0 - self.fuel) / DT)
			};
			self.fuel -= DT * f;
			self.refinery.fuel += DT * f;
			let n = (REFINERY_NEUTRON_FLOW_RATE * self.refinery.neutron_valve.val * self.neutrons)
				.min(self.neutrons.min(REFINERY_NEUTRON_CAP - self.refinery.neutrons) / DT);
			if n < 0.0 {
				log!("n: {}, neutrons: {}, refinery.neutrons: {}",n,self.neutrons,self.refinery.neutrons);
			}
			self.neutrons -= DT * n;
			self.refinery.neutrons += DT * n;
			let w = (REFINERY_WASTE_FLOW_RATE * self.refinery.waste_valve.val * self.waste / 10.0)
				.min(self.waste.min(REFINERY_WASTE_CAP - self.refinery.waste) / DT);
			self.waste -= DT * w;
			self.refinery.waste += DT * w;
			self.refinery.tick();
		}
		{
			if self.fuel < 0.0 {
				log!("fuel{}",self.fuel);
			}
			if self.neutrons < 0.0 {
				log!("neutrons{}",self.neutrons);
			}
			if self.waste < 0.0 {
				log!("waste{}",self.waste);
			}
			if self.refinery.fuel < 0.0 {
				log!("refinery.fuel{}",self.refinery.fuel);
			}
			if self.refinery.neutrons < 0.0 {
				log!("refinery.neutrons{}",self.refinery.neutrons);
			}
			if self.refinery.waste < 0.0 {
				log!("refinery.waste{}",self.refinery.waste);
			}
		}
		self.heat = self.heat.max(0.0);
	}
	
	pub fn vertices(&self) -> Vec<Vertex> {
		let mut result = Vec::new();
		let v = &mut result;
		
		let h = (0.67 * self.heat).min(2.0);
		let panel_color = if h >= 1.0 { blend(LIGHT_GREY, RED, 0.8 * (h - 1.0)) } else { blend(GREY, LIGHT_GREY, h) };
		let panel_size = self.panel_size();
		let panel_pos = -panel_size / 2.0;
		let bar_size = self.bar_size();
		let bar_spacing = self.bar_spacing();
		let bar_gap = self.bar_gap();
		quad(v, panel_pos.extend(1.0), panel_size, Color(panel_color));
		quad(v, vec3(panel_pos.x + bar_gap + bar_spacing, -bar_size.y / 2.0, 2.0), bar_size, Color(BLACK));
		quad(v, vec3(panel_pos.x + bar_gap + bar_spacing * 2.0, -bar_size.y / 2.0, 2.0), bar_size, Color(BLACK));
		quad(v, vec3(panel_pos.x + bar_gap + bar_spacing * 2.0, -bar_size.y / 2.0, 2.0), bar_size, Color(BLACK));
		quad(v, vec3(panel_pos.x + bar_gap + bar_spacing * 2.0, bar_size.y / 6.0, 3.0), vec2(bar_size.x, bar_size.y / 3.0), Color(DULL_RED));
		quad(v, vec3(panel_pos.x + bar_gap + bar_spacing, -bar_size.y / 2.0, 4.0), vec2(bar_size.x, bar_size.y * self.fuel / 3.0), Color(YELLOW));
		quad(v, vec3(panel_pos.x + bar_gap + bar_spacing * 2.0, -bar_size.y / 2.0, 4.0), vec2(bar_size.x, bar_size.y * self.neutrons.min(2.0) / 3.0), Color(WHITE));
		quad(v, vec3(panel_pos.x + bar_gap + bar_spacing * 2.0, bar_size.y / 6.0, 4.0), vec2(bar_size.x, bar_size.y * ((self.neutrons - 2.0) / 3.0).max(0.0)), Color(RED));
		quad(v, vec3(panel_pos.x + bar_gap + bar_spacing, -bar_size.y / 6.0 - 0.0015, 5.0), vec2(bar_size.x + 0.05, 0.003), Color(DARK_GREEN_TRANSPARENT));
		quad(v, vec3(-0.03, bar_size.y / 2.0 - 0.06, 2.0), vec2(0.06, 0.06), Color(DARK_GREY));
		quad(v, vec3(-0.03 + 0.005, bar_size.y / 2.0 - 0.06 + 0.005, 3.0), vec2(0.05, 0.05), Texture(if self.heat < 2.0 { 0 } else { 4 }));
		quad(v, vec3(-0.03, bar_size.y / 2.0 - 0.13, 2.0), vec2(0.06, 0.06), Color(DARK_GREY));
		quad(v, vec3(-0.03 + 0.005, bar_size.y / 2.0 - 0.13 + 0.005, 3.0), vec2(0.05, 0.05), Texture(if self.vent { 2 } else { 1 }));
		if self.fuel_valve_unlocked() {
			let p = self.fuel_valve_pos();
			quad(v, p.extend(2.0), vec2(0.06, 0.06), Color(DARK_GREY));
			let n = if self.fuel_valve_blocked() { 4 } else if self.fuel_valve() { 6 } else { 5 };
			quad(v, (p + vec2(0.005, 0.005)).extend(2.0), vec2(0.05, 0.05), Texture(n));
		}
		if self.water_tank.unlocked {
			quad(v, vec3(-0.29, -bar_size.y / 2.0, 2.0), bar_size, Color(BLACK));
			quad(v, vec3(-0.29, -bar_size.y / 2.0, 4.0), vec2(bar_size.x, bar_size.y / 2.5 * self.water), Color(BLUE));
			self.water_tank.render(v);
		}
		if self.refinery.unlocked {
			quad(v, vec3(panel_pos.x + bar_gap + bar_spacing * 3.0, -bar_size.y / 2.0, 2.0), bar_size, Color(BLACK));
			quad(v, vec3(panel_pos.x + bar_gap + bar_spacing * 3.0, -bar_size.y / 2.0, 4.0), vec2(bar_size.x, bar_size.y * self.waste / 10.0), Color(GREEN));
			self.refinery.render(v);
		}
		
		result
	}
	
	pub fn click(&mut self, m: Vec2<f32>) {
		let panel_size = self.panel_size();
		let bar_size = self.bar_size();
		if rect(m, -panel_size / 2.0, panel_size) {
			if circle(m, vec2(0.0, bar_size.y / 2.0 - 0.06 + 0.005 + 0.05 / 2.0), 0.05 / 2.0) {
				self.ignite();
			} else if circle(m, vec2(0.0, bar_size.y / 2.0 - 0.13 + 0.005 + 0.05 / 2.0), 0.05 / 2.0) {
				self.toggle_vent();
			}
		} else if rect(m, self.fuel_valve_pos() + vec2(0.005, 0.005), self.fuel_valve_size() - vec2(0.01, 0.01)) {
			self.toggle_fuel_valve();
		}
		if self.water_tank.unlocked { self.water_tank.click(m) };
		if self.refinery.unlocked { self.refinery.click(m) };
	}
	
	pub fn drag(&mut self, initial_pos: Vec2<f32>, pos: Vec2<f32>, delta: Vec2<f32>) -> bool {
		if self.water_tank.unlocked {
			if self.water_tank.drag(initial_pos, pos, delta) {
				return true;
			}
		}
		if self.refinery.unlocked {
			if self.refinery.drag(initial_pos, pos, delta) {
				return true;
			}
		}
		false
	}
	
	fn ignite(&mut self) {
		if self.heat < 2.0 && self.neutrons < 1.0 && self.fuel >= FUEL_NEUTRON_RATIO.recip() * 2.0 {
			self.neutrons += 1.0;
			self.fuel -= FUEL_NEUTRON_RATIO.recip() * 2.0;
		}
	}
	
	fn panel_size(&self) -> Vec2<f32> {
		vec2(0.6, 0.3)
	}
	
	fn bar_size(&self) -> Vec2<f32> {
		vec2(0.037, self.panel_size().y - 2.0 * self.bar_gap())
	}
	
	fn bar_spacing(&self) -> f32 {
		0.05
	}
	
	fn bar_gap(&self) -> f32 {
		self.panel_size().y / 30.0
	}
	
	fn fuel_valve_pos(&self) -> Vec2<f32> {
		-self.panel_size()+vec2(-0.15, 0.05)
	}
	
	fn fuel_valve_size(&self) -> Vec2<f32> {
		vec2(0.06, 0.06)
	}
	
	fn fuel_valve(&self) -> bool {
		self.fuel_valve[0] && !self.fuel_valve[1]
	}
	
	fn fuel_valve_blocked(&self) -> bool {
		self.fuel_valve[1]
	}
	
	fn toggle_fuel_valve(&mut self) {
		self.fuel_valve[0] = !self.fuel_valve[0];
	}
	
	fn fuel_valve_unlocked(&self) -> bool {
		self.fuel_valve[2]
	}
	
	fn toggle_vent(&mut self) {
		self.vent = !self.vent;
	}
}

struct WaterTank {
	unlocked: bool,
	water: f32,
	heat: f32,
	valve: Dial,
}

impl WaterTank {
	fn new() -> Self {
		let valve_pos = vec2(-0.4, 0.27);
		let valve_size = 0.1;
		Self {
			unlocked: false,
			water: 20.0,
			heat: 0.0,
			valve: Dial {
				pos: valve_pos + valve_size / 12.0,
				size: valve_size * 5.0 / 6.0,
				val: 0.0,
				range: Some((0.0, 0.5)),
				tex: Texture(3),
				z_index: 3.0,
				background: Some(make_quad(valve_pos.extend(2.0), vec2(valve_size,valve_size), Color(DARK_GREY), Mat2::ident())),
			},
		}
	}
	
	fn add_water(&mut self, amount: f32, heat: f32) {
		self.heat = ((self.water + 1.0) * self.heat + amount * heat) / (self.water + 1.0 + amount);
		self.water += amount;
	}
	
	fn tick(&mut self) {
		let amount = (DT * WATER_REFILL_RATE).min(25.0 - self.water);
		self.add_water(amount, 0.0);
		let evaporation_amount = (EVAPORATION_RATE * DT * self.water.sqrt() * self.heat * (3.1 - self.heat).recip().abs()).min(self.water).max(0.0);
		let heat_reduction = (3.0 - self.heat).max(0.1); //water boils at heat 3, if tank temp > 3 then water boiling off will be self.heat + 0.1 so heat reduction = 0.1
		self.water -= evaporation_amount;
		self.heat -= (evaporation_amount * heat_reduction) / (self.water + 1.0);
		//self.heat -= DT * (HEAT_DISSIPATION_RATE * 0.1 * self.heat) / (1.0 + self.water); //heat dissipation, disabled because it unbalanced reactor heat dissipation/evaporation requirement
		self.heat = self.heat.max(0.0);
	}
	
	fn render(&self, v: &mut Vec<Vertex>) {
		let heat = (self.heat * 0.67).min(2.0);
		let panel_color = if heat >= 1.0 { blend(LIGHT_GREY, RED, 0.8 * (heat - 1.0)) } else { blend(GREY, LIGHT_GREY, heat) };
		let panel_pos = self.panel_pos();
		let panel_size = self.panel_size();
		quad(v, panel_pos.extend(1.0), panel_size, Color(panel_color));
		quad(v, (panel_pos + 0.01).extend(1.5), panel_size - vec2(0.02, 0.01), Color(blend(GLASS, RED, 0.3 * (heat - 1.0).max(0.0))));
		quad(v, (panel_pos + 0.01).extend(2.0), vec2(panel_size.x - 0.02, (panel_size.y - 0.02) * self.water / 25.0), Color(BLUE));
		//quad(v, self.valve_pos().extend(2.0), self.valve_size(), Color(DARK_GREY));
		//transformed_quad(v, (self.valve_pos() + (self.valve_size() / 12.0)).extend(3.0), self.valve_size() * 5.0 / 6.0, Texture(3), Mat2::<f32>::rotate(-self.valve * PI));
		self.valve.render(v);
	}
	
	fn click(&mut self, _m: Vec2<f32>) {
	}
	
	fn drag(&mut self, initial_pos: Vec2<f32>, pos: Vec2<f32>, delta: Vec2<f32>) -> bool {
		/*let center = self.valve_pos() + self.valve_size() / 2.0;
		if circle(initial_pos, center, self.valve_size().x * 2.5 / 6.0)  && distance(initial_pos, center) >= 0.001 {
			let mut theta = angle(center, pos - delta, pos);
			if theta.abs() > 1.0 {
				theta = 0.0;
			}
			self.valve = (self.valve + theta / PI).min(1.0).max(0.0);
			true
		} else {
			false
		}*/
		self.valve.drag(initial_pos, pos, delta)
	}
	
	fn panel_pos(&self) -> Vec2<f32> {
		vec2(-1.0, 0.4)
	}
	
	fn panel_size(&self) -> Vec2<f32> {
		vec2(0.5, 0.3)
	}
	
	fn set_valve(&mut self, v: f32) {
		self.valve.val = v;
	}
}

const WASTE_FUEL_RATIO: f32 = 0.2;
const REFINERY_NEUTRON_DECAY_RATE: f32 = 0.03;
const REFINERY_REACTION_RATE: f32 = 0.1;
const REFINERY_FUEL_CAP: f32 = 10.0;
const REFINERY_NEUTRON_CAP: f32 = 0.2;
const REFINERY_WASTE_CAP: f32 = 20.0;

struct Refinery {
	unlocked: bool,
	fuel: f32,
	neutrons: f32,
	waste: f32,
	fuel_valve: Dial,
	neutron_valve: Dial,
	waste_valve: Dial,
}

impl Refinery {
	fn new() -> Self {
		let fuel_valve_pos = vec2(-0.55,-0.4);
		let neutron_valve_pos = vec2(-0.35,-0.4);
		let waste_valve_pos = vec2(-0.2,-0.4);
		let fuel_valve_size = 0.1;
		let neutron_valve_size = 0.1;
		let waste_valve_size = 0.1;
		Self {
			unlocked: false,
			fuel: 0.0,
			neutrons: 0.0,
			waste: 0.0,
			fuel_valve: Dial {
				pos: fuel_valve_pos + fuel_valve_size / 12.0,
				size: fuel_valve_size * 5.0 / 6.0,
				val: 0.5,
				range: Some((0.0, 0.5)),
				tex: Texture(3),
				z_index: 3.0,
				background: Some(make_quad(fuel_valve_pos.extend(2.0), vec2(fuel_valve_size,fuel_valve_size), Color(DARK_GREY), Mat2::ident())),
			},
			neutron_valve: Dial {
				pos: neutron_valve_pos + neutron_valve_size / 12.0,
				size: neutron_valve_size * 5.0 / 6.0,
				val: 0.0,
				range: Some((0.0, 0.5)),
				tex: Texture(3),
				z_index: 3.0,
				background: Some(make_quad(neutron_valve_pos.extend(2.0), vec2(neutron_valve_size,neutron_valve_size), Color(DARK_GREY), Mat2::ident())),
			},
			waste_valve: Dial {
				pos: waste_valve_pos + waste_valve_size / 12.0,
				size: waste_valve_size * 5.0 / 6.0,
				val: 0.0,
				range: Some((0.0, 0.5)),
				tex: Texture(3),
				z_index: 3.0,
				background: Some(make_quad(waste_valve_pos.extend(2.0), vec2(waste_valve_size,waste_valve_size), Color(DARK_GREY), Mat2::ident())),
			},
		}
	}
	
	fn tick(&mut self) {
		let reaction_speed = (REFINERY_REACTION_RATE * self.neutrons * self.waste * self.fuel * (10.0 - self.fuel)).min(self.waste / DT);
		//log!("{}",self.fuel);
		self.fuel += DT * WASTE_FUEL_RATIO * reaction_speed;
		self.waste -= DT * reaction_speed;
		self.neutrons -= DT * self.neutrons * REFINERY_NEUTRON_DECAY_RATE;
	}
	
	fn render(&self, v: &mut Vec<Vertex>) {
		let panel_pos = self.panel_pos();
		let panel_size = self.panel_size();
		let (bar_size, bar_gap, bar_spacing) = (self.bar_size(), self.bar_gap(), self.bar_spacing());
		quad(v, panel_pos.extend(1.0), panel_size, Color(DULL_RED));
		quad(v, (panel_pos + bar_gap).extend(2.0), bar_size, Color(BLACK));
		quad(v, (panel_pos + bar_gap).extend(3.0), bar_size * vec2(1.0, self.fuel / REFINERY_FUEL_CAP), Color(YELLOW));
		quad(v, (panel_pos + bar_gap + vec2(bar_spacing, 0.0)).extend(2.0), bar_size, Color(BLACK));
		quad(v, (panel_pos + bar_gap + vec2(bar_spacing, 0.0)).extend(3.0), bar_size * vec2(1.0, self.neutrons / REFINERY_NEUTRON_CAP), Color(WHITE));
		quad(v, (panel_pos + bar_gap + vec2(bar_spacing, 0.0) * 4.0).extend(2.0), bar_size, Color(BLACK));
		quad(v, (panel_pos + bar_gap + vec2(bar_spacing, 0.0) * 4.0).extend(3.0), bar_size * vec2(1.0, self.waste / REFINERY_WASTE_CAP), Color(GREEN));
		self.fuel_valve.render(v);
		self.neutron_valve.render(v);
		self.waste_valve.render(v);
	}
	
	fn click(&mut self, _m: Vec2<f32>) {
		/*if rect(m, self.panel_pos(), self.panel_size()) {
			
		}*/
	}
	
	fn drag(&mut self, initial_pos: Vec2<f32>, pos: Vec2<f32>, delta: Vec2<f32>) -> bool {
		if self.fuel_valve.drag(initial_pos, pos, delta) {
			true
		} else if self.neutron_valve.drag(initial_pos, pos, delta) {
			true
		} else if self.waste_valve.drag(initial_pos, pos, delta) {
			true
		} else {
			false
		}
	}
	
	fn panel_pos(&self) -> Vec2<f32> {
		vec2(-0.6, -0.9)
	}
	
	fn panel_size(&self) -> Vec2<f32> {
		vec2(self.bar_gap() * 2.0 + self.bar_spacing() * 4.0 + self.bar_size().x, self.panel_size_y__())
	}
	
	fn panel_size_y__(&self) -> f32 { //how to not recurse
		0.4
	}
	
	fn bar_size(&self) -> Vec2<f32> {
		vec2(0.037, self.panel_size_y__() - 2.0 * self.bar_gap())
	}
	
	fn bar_spacing(&self) -> f32 {
		0.05
	}
	
	fn bar_gap(&self) -> f32 {
		self.panel_size_y__() / 40.0
	}
}
