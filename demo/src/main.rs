use std::path::PathBuf;

use glfw::{self, Context};
use gl::{self, types::*};

use avocet::graphics::opengl::ShaderProgram;

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
    let mut window_manager = WindowManager::new();
    let (mut window, _) = window_manager.create_window(800, 600, "Hello GLFW")
        .expect("Failed to create GLFW window");

    // Build and compile shaders
    let vertex_path = get_shader_path("vert_identity.hlsl");
    let fragment_path = get_shader_path("frag_monochrome.hlsl");
    let shader_program = ShaderProgram::new(vertex_path, fragment_path).unwrap();

    // Set up vertex data and configure vertex attributes
    let vertices: [f32; 9] = [
        -0.5, -0.5, 0.0, // left  
        0.5, -0.5, 0.0, // right 
        0.0,  0.5, 0.0  // top   
    ];

    // Setup Vertex Buffer and Array objects
    let (vbo, vao) = unsafe {
        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        
        let mut vbo = 0;
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            size_of_val(&vertices) as GLsizeiptr,
            vertices.as_ptr() as *const _,
            gl::STATIC_DRAW
        );

        (vbo, vao)
    };

    unsafe {
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (size_of::<f32>() * 3) as GLsizei, std::ptr::null());
        gl::EnableVertexAttribArray(0);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    };

    // The core program loop
    while !window.should_close() {
        unsafe{
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            shader_program.bind();
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        };

        window.swap_buffers(); // 'glfwSwapBuffers'
        window_manager.poll_events(); // 'glfwPollEvents'
    }

    // Clean up after ourselves
    unsafe {
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteBuffers(1, &vbo);
    };
}