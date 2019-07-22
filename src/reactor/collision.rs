use crate::prelude::*;

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
