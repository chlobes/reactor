pub use math_lib::vec2::*;
pub use math_lib::vec3::*;
pub use math_lib::mat2::*;
pub use array_tuple::ArrayTuple;
pub use std::f32::consts::PI;
pub fn modulus<A: std::ops::Rem<f32,Output=A>+std::ops::Add<f32,Output=A>>(a: A, b: f32) -> A { ((a % b) + b) % b }
