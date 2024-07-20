use std::{
	ffi::CString,
	mem, ptr,
	time::{self, Instant, SystemTime},
};

use gl::types::{GLboolean, GLenum, GLfloat, GLint, GLsizeiptr, GLuint};
use glfw::{fail_on_errors, Action, Context, Key, WindowEvent, WindowHint, WindowMode};

// https://github.com/brendanzab/gl-rs/blob/1cd256c7b59a0baac8af7863ca0f768bfc1a4b51/gl/examples/triangle.rs#L44-L111

fn compile_shader(src: &str, ty: GLenum) -> GLuint {
	#[allow(clippy::uninit_vec)]
	unsafe {
		let shader = gl::CreateShader(ty);

		let c_str = CString::new(src.as_bytes()).unwrap();
		gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
		gl::CompileShader(shader);

		let mut status = gl::FALSE as GLint;
		gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

		if status != gl::TRUE as GLint {
			let mut len = 0;
			gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);

			let mut buf = Vec::with_capacity(len as usize);
			buf.set_len(len as usize - 1);
			gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut _);

			panic!(
				"{}",
				String::from_utf8(buf).expect("ShaderInfoLog not valid utf8.")
			);
		}

		shader
	}
}

fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
	#[allow(clippy::uninit_vec)]
	unsafe {
		let program = gl::CreateProgram();
		gl::AttachShader(program, vs);
		gl::AttachShader(program, fs);
		gl::LinkProgram(program);

		let mut status = gl::FALSE as GLint;
		gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

		if status != (gl::TRUE as GLint) {
			let mut len: GLint = 0;
			gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);

			let mut buf = Vec::with_capacity(len as usize);
			buf.set_len(len as usize - 1);
			gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut _);

			panic!(
				"{}",
				String::from_utf8(buf).expect("ProgramInfoLog not valid utf8")
			);
		}

		program
	}
}

fn main() {
	let mut glfw = glfw::init(fail_on_errors!()).unwrap();

	glfw.window_hint(WindowHint::Resizable(false));

	let resolution = (1024, 1024);

	let (mut window, events) = glfw
		.create_window(
			resolution.0,
			resolution.1,
			"Worley Noise",
			WindowMode::Windowed,
		)
		.expect("Failed to create GLFW window.");
	window.make_current();
	window.set_key_polling(true);

	gl::load_with(|s| glfw.get_proc_address_raw(s));

	let mut vao = 0;
	let mut vbo = 0;

	let vs = compile_shader(include_str!("shaders/vertex.glsl"), gl::VERTEX_SHADER);
	let fs = compile_shader(include_str!("shaders/fragment.glsl"), gl::FRAGMENT_SHADER);
	let program = link_program(vs, fs);

	unsafe {
		gl::GenVertexArrays(1, &mut vao);
		gl::BindVertexArray(vao);

		#[rustfmt::skip]
		let vertex_buf: [GLfloat; 8] = [
			1.0, -1.0,
			-1.0, -1.0,
			1.0, 1.0,
			-1.0, 1.0
		];

		gl::GenBuffers(1, &mut vbo);
		gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
		gl::BufferData(
			gl::ARRAY_BUFFER,
			(vertex_buf.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
			&vertex_buf[0] as *const GLfloat as *const _,
			gl::STATIC_DRAW,
		);

		gl::UseProgram(program);

		let c_str = CString::new("resolution").unwrap();
		let res_location = gl::GetUniformLocation(program, c_str.as_ptr());
		gl::Uniform2f(
			res_location,
			resolution.0 as GLfloat,
			resolution.1 as GLfloat,
		);

		let c_str = CString::new("seed").unwrap();
		let seed_location = gl::GetUniformLocation(program, c_str.as_ptr());

		let seed = match SystemTime::now().duration_since(time::UNIX_EPOCH) {
			Ok(d) => (d.as_millis() % i16::MAX as u128) as f32,
			Err(e) => {
				panic!("{}", e);
			}
		};
		gl::Uniform1f(seed_location, seed);

		gl::EnableVertexAttribArray(0);
		gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE as GLboolean, 0, ptr::null());
	}

	let time_location = unsafe {
		let c_str = CString::new("time").unwrap();
		gl::GetUniformLocation(program, c_str.as_ptr())
	};

	let instant = Instant::now();
	while !window.should_close() {
		unsafe {
			gl::Uniform1f(time_location, instant.elapsed().as_secs_f32());
			gl::ClearColor(0.0, 0.0, 0.0, 1.0);
			gl::Clear(gl::COLOR_BUFFER_BIT);
			gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
		}
		window.swap_buffers();

		glfw.poll_events();
		for (_, event) in glfw::flush_messages(&events) {
			#[allow(clippy::single_match)]
			match event {
				WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
					window.set_should_close(true);
				}
				_ => (),
			}
		}
	}

	unsafe {
		gl::DeleteProgram(program);
		gl::DeleteShader(fs);
		gl::DeleteShader(vs);
		gl::DeleteBuffers(1, &vbo);
		gl::DeleteVertexArrays(1, &vao);
	}
}
