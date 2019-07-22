use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlProgram,WebGl2RenderingContext,HtmlImageElement,WebGlShader,MouseEvent,HtmlCanvasElement};
use std::rc::Rc;
use std::cell::{RefCell,Cell};
use self::WebGl2RenderingContext as GL;

macro_rules! log {
	( $( $t:tt )* ) => {
		web_sys::console::log_1(&format!( $( $t )* ).into());
	}
}

mod reactor;
use reactor::Reactor;
mod vertex;
use vertex::Vertex;
mod prelude;
use prelude::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
	#[cfg(feature = "console_error_panic_hook")]
	console_error_panic_hook::set_once();
	
	log!("atan2(1,1): {}",1f32.atan2(1f32));
	log!("atan2(1,-1): {}",1f32.atan2(-1f32));
	log!("atan2(-1,1): {}",(-1f32).atan2(1f32));
	log!("atan2(-1,-1): {}",(-1f32).atan2(-1f32));
	
	let document = window().document().unwrap();
	let canvas = document.get_element_by_id("canvas").unwrap();
	let canvas = Rc::new(canvas.dyn_into::<web_sys::HtmlCanvasElement>()?);
	let context = Rc::new(canvas.get_context("webgl2")?.expect("browser does not support webgl").dyn_into::<GL>()?);
	
	let vert_shader = compile_shader(
		&context,
		GL::VERTEX_SHADER,
		include_str!("vs.vs"),
	)?;
	let frag_shader = compile_shader(
		&context,
		GL::FRAGMENT_SHADER,
		include_str!("fs.fs"),
	)?;
	let program = link_program(&context, &vert_shader, &frag_shader)?;
	context.use_program(Some(&program));
	
	let buffer = context.create_buffer().ok_or("failed to create buffer")?;
	context.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));
	
	let stride = std::mem::size_of::<Vertex>();
	
	context.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, stride as i32, 0);
	context.vertex_attrib_pointer_with_i32(1, 4, GL::FLOAT, false, stride as i32, 12);
	context.vertex_attrib_pointer_with_i32(2, 2, GL::FLOAT, false, stride as i32, 12+16);
	context.vertex_attrib_pointer_with_i32(3, 1, GL::FLOAT, false, stride as i32, 12+16+8);
	context.enable_vertex_attrib_array(0); context.enable_vertex_attrib_array(1); context.enable_vertex_attrib_array(2); context.enable_vertex_attrib_array(3);
	
	let image = Rc::new(RefCell::new(HtmlImageElement::new().unwrap()));
	let image2 = image.clone();
	
	let context2 = context.clone();
	
	let onload = Closure::wrap(Box::new(move|| {
		let texture = context2.create_texture().expect("failed to create texture");
		context2.active_texture(GL::TEXTURE0);
		context2.bind_texture(GL::TEXTURE_2D, Some(&texture));
		context2.pixel_storei(GL::UNPACK_FLIP_Y_WEBGL, 1);
		context2.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32);
		context2.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32);
		context2.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
		context2.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);
		context2.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_LOD, 0);
		context2.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAX_LOD, 0);
		context2.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAX_LEVEL, 0);
		
		context2.tex_image_2d_with_u32_and_u32_and_html_image_element(
			GL::TEXTURE_2D,
			0,
			GL::RGBA as i32,
			GL::RGBA,
			GL::UNSIGNED_BYTE,
			&image2.borrow(),
		).expect("");
	}) as Box<dyn Fn()>);
	
	image.borrow().set_onload(Some(onload.as_ref().unchecked_ref()));
	image.borrow().set_src("textures.png");
	
	onload.forget();
	
	
	let offset = Rc::new(Cell::new(vec2(0.0, 0.0)));
	let offset_location = context.get_uniform_location(&program, "offset");
	context.uniform2f(offset_location.as_ref(), offset.get().x, offset.get().y);
	let aspect_ratio_location = context.get_uniform_location(&program, "aspect_ratio");
	
	let context2 = context.clone();
	let canvas2 = canvas.clone();
	let body = document.body().expect("website had no body");
	let aspect_ratio_location2 = aspect_ratio_location.clone();
	let onresize = Closure::wrap(Box::new(move|| {
		let (w, h) = (body.client_width(), body.client_height());
		context2.uniform1f(aspect_ratio_location2.as_ref(), h as f32 / w as f32);
		context2.viewport(0, 0, w, h);
		canvas2.set_attribute("width",&w.to_string()).expect("failed to set canvas width");
		canvas2.set_attribute("height",&h.to_string()).expect("failed to set canvas height");
	}) as Box<dyn Fn()>);
	window().add_event_listener_with_callback("resize",onresize.as_ref().unchecked_ref()).expect("failed to add resize listener");
	onresize.forget();
	
	let body = document.body().expect("no body present on document");
	let (w, h) = (body.client_width(), body.client_height());
	canvas.set_attribute("width",&w.to_string()).expect("failed to set canvas width");
	canvas.set_attribute("height",&h.to_string()).expect("failed to set canvas height");
	context.uniform1f(aspect_ratio_location.as_ref(), h as f32 / w as f32);
	context.viewport(0, 0, w, h);
	
	context.clear_color(0.0, 0.0, 0.0, 1.0);
	context.enable(GL::DEPTH_TEST);
	context.depth_func(GL::GEQUAL);
	context.enable(GL::BLEND);
	context.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);
	context.pixel_storei(GL::UNPACK_PREMULTIPLY_ALPHA_WEBGL, 1);
	
	let reactor = Rc::new(RefCell::new(Reactor::new()));
	
	let reactor2 = reactor.clone();
	let canvas2 = canvas.clone();
	let offset2 = offset.clone();
	let onclick = Closure::wrap(Box::new(move|e: MouseEvent| {
		let m = screen_coords(e.client_x(), e.client_y(), &canvas2);
		reactor2.borrow_mut().click(m - offset2.get());
	}) as Box<dyn Fn(MouseEvent)>);
	canvas.set_onclick(Some(onclick.as_ref().unchecked_ref()));
	onclick.forget();
	
	let a = Rc::new(Cell::new(None));
	let b = a.clone();
	let c = a.clone();
	let canvas2 = canvas.clone();
	let onmousedown = Closure::wrap(Box::new(move|e: MouseEvent| if e.button() == 0 { a.set(Some(screen_coords(e.client_x(), e.client_y(), &canvas2))) })
		as Box<dyn Fn(MouseEvent)>);
	canvas.set_onmousedown(Some(onmousedown.as_ref().unchecked_ref()));
	onmousedown.forget();
	
	let onmouseup = Closure::wrap(Box::new(move|e: MouseEvent| if e.button() == 0 { b.set(None) })
		as Box<dyn Fn(MouseEvent)>);
	canvas.set_onmouseup(Some(onmouseup.as_ref().unchecked_ref()));
	onmouseup.forget();
	
	let canvas2 = canvas.clone();
	let offset2 = offset.clone();
	let reactor2 = reactor.clone();
	let context2 = context.clone();
	let onmove = Closure::wrap(Box::new(move|e: MouseEvent| {
		if let Some(initial_pos) = c.get() {
			let pos = screen_coords(e.client_x(), e.client_y(), &canvas2);
			let delta = vec2(e.movement_x(), -e.movement_y()).f32() * 2.0 / canvas2.client_height() as f32;
			if !reactor2.borrow_mut().drag(initial_pos - offset2.get(), pos - offset2.get(), delta) {
				offset2.set(offset2.get() + delta);
				context2.uniform2f(offset_location.as_ref(), offset2.get().x, offset2.get().y);
			}
		}
	}) as Box<dyn Fn(MouseEvent)>);
	canvas.set_onmousemove(Some(onmove.as_ref().unchecked_ref()));
	onmove.forget();
	
	let f = Rc::new(RefCell::new(None));
	let g = f.clone();
	
	// Note that `Float32Array::view` is somewhat dangerous (hence the
	// `unsafe`!). This is creating a raw view into our module's
	// `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
	// (aka do a memory allocation in Rust) it'll cause the buffer to change,
	// causing the `Float32Array` to be invalid.
	//
	// As a result, after `Float32Array::view` we have to be very careful not to
	// do any memory allocations before it's dropped.
	
	*g.borrow_mut() = Some(Closure::wrap(Box::new(move|| {
		context.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);
		context.clear_depth(-1.1);
		let mut reactor = reactor.borrow_mut();
		reactor.tick();
		let mut verts = reactor.vertices();
		let len = verts.len();
		
		unsafe {
			let ptr = verts.as_mut_ptr() as *mut f32; //note that transmuting like this is safe because both Vertex and Vec2/Vec3 are repr(C) so they are just a number of floats anyway
			let len = len * stride / 4;
			let cap = verts.capacity() * stride;
			std::mem::forget(verts);
			let verts = Vec::from_raw_parts(ptr, len, cap);
			let verts = js_sys::Float32Array::view(&verts);
			
			context.buffer_data_with_array_buffer_view(
				GL::ARRAY_BUFFER,
				&verts,
				GL::STATIC_DRAW,
			);
		}
		
		context.draw_arrays(GL::TRIANGLES, 0, len as i32);
		
		request_animation_frame(f.borrow().as_ref().unwrap());
	}) as Box<dyn FnMut()>));
	
	request_animation_frame(g.borrow().as_ref().unwrap());
	
	Ok(())
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
	window()
		.request_animation_frame(f.as_ref().unchecked_ref())
		.expect("should register `requestAnimationFrame` OK");
}


fn window() -> web_sys::Window {
	web_sys::window().expect("no global `window` exists")
}

fn compile_shader(
	context: &GL,
	shader_type: u32,
	source: &str,
) -> Result<WebGlShader, String> {
	let shader = context
		.create_shader(shader_type)
		.ok_or_else(|| String::from("Unable to create shader object"))?;
	context.shader_source(&shader, source);
	context.compile_shader(&shader);
	
	if context.get_shader_parameter(&shader, GL::COMPILE_STATUS).as_bool().unwrap_or(false)	{
		Ok(shader)
	} else {
		Err(context.get_shader_info_log(&shader).unwrap_or_else(|| String::from("Unknown error creating shader")))
	}
}

fn link_program(
	context: &GL,
	vert_shader: &WebGlShader,
	frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
	let program = context
		.create_program()
		.ok_or_else(|| String::from("Unable to create shader object"))?;
	
	context.attach_shader(&program, vert_shader);
	context.attach_shader(&program, frag_shader);
	context.link_program(&program);
	
	if context
		.get_program_parameter(&program, GL::LINK_STATUS)
		.as_bool()
		.unwrap_or(false)
	{
		Ok(program)
	} else {
		Err(context
			.get_program_info_log(&program)
			.unwrap_or_else(|| String::from("Unknown error creating program object")))
	}
}

fn screen_coords(x: i32, y: i32, canvas: &HtmlCanvasElement) -> Vec2<f32> {
	let x = x as f32;
	let y = -y as f32;
	let (x, y) = (x / canvas.client_width() as f32 * 2.0 - 1.0, y / canvas.client_height() as f32 * 2.0 + 1.0);
	let x = x * canvas.client_width() as f32 / canvas.client_height() as f32; //multiply by aspect ratio so it will line up with aspect ratio rendered
	vec2(x,y)
}
