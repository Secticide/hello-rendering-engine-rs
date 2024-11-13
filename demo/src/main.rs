use std::path::PathBuf;

use glfw::{self, Context};

use avocet::{
    graphics::opengl as agl,
    geometry::Triangle,
};

mod util;
use util::WindowManager;

fn get_shader_path(filename: &str) -> PathBuf {
    const CARGO_MANIFEST_DIR: &'static str = std::env!("CARGO_MANIFEST_DIR");
    const SHADERS_DIR_NAME: &'static str = "shaders";

    let directory_separator_count = 2;
    let mut path = PathBuf::with_capacity(
        CARGO_MANIFEST_DIR.len() +
        SHADERS_DIR_NAME.len() +
        directory_separator_count +
        filename.len());
    
    path.push(CARGO_MANIFEST_DIR);
    path.push(SHADERS_DIR_NAME);
    path.push(filename);

    path
}

fn main() {
    let mut window_manager = match WindowManager::new() {
        Ok(wm) => wm,
        Err(init_error) => {
            eprintln!("Failed to initialise: {:?}", init_error);
            return;
        },
    };

    let (mut window, _) = window_manager.create_window(800, 600, "Hello Rendering Engine")
        .expect("Failed to create GLFW window");

    println!(
        "Vendor: {}\nRenderer: {}\nVersion: {}",
        avocet::version::get_opengl_vendor_string(),
        avocet::version::get_opengl_renderer_string(),
        avocet::version::get_opengl_version_string(),
    );

    // Build and compile shaders
    let vertex_path = get_shader_path("vert_identity.glsl");
    let fragment_path = get_shader_path("frag_monochrome.glsl");
    let shader_program = agl::ShaderProgram::new(vertex_path, fragment_path).unwrap();
    let triangle = Triangle::new();

    // The core program loop
    while !window.should_close() {
        unsafe{
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            shader_program.bind();
            triangle.draw();
        };

        window.swap_buffers(); // 'glfwSwapBuffers'
        window_manager.poll_events(); // 'glfwPollEvents'
    }
}