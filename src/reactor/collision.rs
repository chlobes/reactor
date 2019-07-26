use crate::prelude::*;
use crate::vertex::{Vertex,Tex,transformed_quad};

pub fn circle(pos: Vec2<f32>, center: Vec2<f32>, radius: f32) -> bool {
	let pos = pos - center;
	pos.x * pos.x + pos.y * pos.y <= radius * radius
}

pub fn rect(pos: Vec2<f32>, rect: Vec2<f32>, size: Vec2<f32>) -> bool {
	pos.x >= rect.x && pos.x <= rect.x + size.x &&
	pos.y >= rect.y && pos.y <= rect.y + size.y
}

pub fn angle(center: Vec2<f32>, a: Vec2<f32>, b: Vec2<f32>) -> f32 {
	let a = a - center;
	let b = b - center;
	//let cos_angle = dot(a, b) / (a.magnitude() * b.magnitude());
	//cos_angle.acos()
	(a.y).atan2(a.x) - (b.y).atan2(b.x)
}

#[derive(Debug,Copy,Clone)]
pub struct Dial {
	pub pos: Vec2<f32>,
	pub size: Vec2<f32>,
	pub val: f32,
	pub range: Option<(f32, f32)>,
	pub tex: Tex,
	pub z_index: f32,
	pub background: Option<[Vertex; 6]>,
}

impl Dial {
	pub fn render(&self, v: &mut Vec<Vertex>) {
		self.background.as_ref().map(|b| v.extend_from_slice(b));
		transformed_quad(v, self.pos.extend(self.z_index), self.size, self.tex, self.trans());
	}
	
	pub fn trans(&self) -> Mat2<f32> {
		let v = -self.val * 2.0 * PI;
		Mat2::<f32>::rotate(self.range.map(|(min, max)| (min + v) * (max - min)).unwrap_or(v))
	}
	
	pub fn drag(&mut self, initial_pos: Vec2<f32>, pos: Vec2<f32>, delta: Vec2<f32>) -> bool {
		let center = self.pos + self.size / 2.0;
		if circle(initial_pos, center, self.size.x * 2.5 / 6.0)  && distance(initial_pos, center) >= 0.001 {
			let mut theta = angle(center, pos - delta, pos);
			if theta.abs() > 1.0 {
				theta = 0.0;
			}
			if let Some((min, max)) = self.range {
				self.val = (self.val + theta / (2.0 * PI) / (max - min)).min(1.0).max(0.0);
			} else {
				self.val = modulus(self.val + theta / (2.0 * PI), 1.0);
			}
			true
		} else {
			false
		}
	}
}
