use std::ffi::CStr;

use glfw::{self, Context};
use gl::{self, types::*};

// Using this syntax we can choose symbols to import
use avocet::graphics::opengl::{
    ShaderStage,
    ShaderCompiler,
    check_linking_success,
};

// This is essentially a C++ 'static const char*', '&CStr' is a C-string-view and the 'static is referring to the lifetime
static VERTEX_SHADER_SOURCE: &'static CStr = c"
#version 330 core
layout (location = 0) in vec3 aPos;
void main() {
    gl_Position = vec4(aPos.xyz, 1.0);
}";

// Multiline literals!
static FRAGMENT_SHADER_SOURCE: &'static CStr = c"
#version 330 core
out vec4 FragColor;
void main() {
    FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
}";

fn main() {
    // Initialise GLFW
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

    // Create the window and event handler
    let (mut window, _) = glfw.create_window(800, 600, "Hello GLFW", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.set_key_polling(true); // 'glfwSetKeyCallback'
    window.make_current(); // 'glfwMakeContextCurrent'

    // 'gl' is a OpenGL loader (similar to GLAD)
    gl::load_with(|symbol_name| window.get_proc_address(symbol_name));

    // Build and compile shaders
    let vertex_shader = ShaderCompiler::new(ShaderStage::Vertex, VERTEX_SHADER_SOURCE).unwrap();
    let fragment_shader  = ShaderCompiler::new(ShaderStage::Fragment, FRAGMENT_SHADER_SOURCE).unwrap();

    // Link shaders
    let shader_program = unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vertex_shader.resource().handle().index());
        gl::AttachShader(program, fragment_shader.resource().handle().index());
        gl::LinkProgram(program);
        check_linking_success(program).unwrap();
        program
    };

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

            gl::UseProgram(shader_program);
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        };

        window.swap_buffers(); // 'glfwSwapBuffers'
        glfw.poll_events(); // 'glfwPollEvents'
    }

    // Clean up after ourselves
    unsafe {
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteProgram(shader_program);
    };
}