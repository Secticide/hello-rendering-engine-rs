use std::ffi::CStr;

use glfw::{self, Context};
use gl::{self, types::*};

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

// Rust types are idiomatically named in Pascal case
// - Structs in Rust only contain data, not functions
// - Structs with braces contain anonymous types, and can be tuples, the fields are accessed with their number e.g. 'self.0'
// - Rust is move-by-default, and moves are destructive. This means that we do not need to worry about move constructors etc.
// - 'pub' here is the 'public' access modifier, it is public since we will eventually want it to be used from outside our library
pub struct ResourceHandle(GLuint);

// 'impl' blocks contain the associated functions and methods for a type
impl ResourceHandle {
    // Rust doesn't have constructors
    // - An associated 'new' function is provided as a convention (ResourceHandle::new(0))
    pub fn new(index: GLuint) -> Self {
        // This is the only single way to initialise a braced struct
        // Return statements here are not required, statements without semi-colons act as the return
        Self(index)
    }

    // This is what I call a 'dot method', a method that can be accessed via 'handle.index()'
    // - The 'self' here is an explicit 'this' pointer
    // - The '&' here is a reference, but since everything is immutable in Rust by default - this is an immutable reference
    #[must_use] pub fn index(&self) -> GLuint { self.0 }
}

// 'Default' is a trait (similar to an interface or C++20 concept)
// - Here we are implementing the default trait for our ResourceHandle
impl Default for ResourceHandle {
    fn default() -> Self {
        Self(0)
    }
}

// Similarly to the 'ResourceHandle' type, this implicitly cannot be constructed by code outside this module due to the braced type not being public
pub struct ShaderResource(ResourceHandle);

impl ShaderResource {
    // Capitalised 'Self' refers to the type name, it is shorthand
    pub fn new(shader_stage: GLenum) -> Self {
        let handle = unsafe { gl::CreateShader(shader_stage) };
        Self(ResourceHandle(handle))
    }

    // The '#[must_use]' here is an attribute - similar to '[[nodiscard]]' in C++17
    #[must_use] pub fn handle(&self) -> &ResourceHandle { &self.0 }
}

// In Rust, we don't explictly have destructors. The 'Drop' trait provides a way to create RAII style types,
// giving types which obtain a resource at initialisation time a change to clean up 
impl Drop for ShaderResource {
    fn drop(&mut self) {
        unsafe{ gl::DeleteShader(self.0.index()) }
    }
}

pub struct ShaderCompiler(ShaderResource);

impl ShaderCompiler {
    pub fn new(shader_stage: GLenum, source: &CStr) -> Result<Self, String> {
        let resource = ShaderResource::new(shader_stage);
        let shader = resource.handle().index();
        
        unsafe{
            gl::ShaderSource(shader, 1, &source.as_ptr(), std::ptr::null());
            gl::CompileShader(shader);

            check_compilation_success(shader)?;
        };

        Ok(Self(resource))
    }

    #[must_use] pub fn resource(&self) -> &ShaderResource { &self.0 }
}

fn main() {
    // Initialise GLFW
    // - calls 'glfwInit()' internally
    // - Rust doesn't have exceptions, the 'Result<T, E>' type is used to handle errors (similar to 'std::expected' in C++23)
    // - The GLFW 'crate' (library) we're using has a Rust-like wrapper to make it easier to use
    // - Calling 'Result::<T>::unwrap' on an error will abort the program, 'unwrap' is generally used to get a program up and running quickly
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

    // Create the window and event handler (currently discarding the event handler using the '_' underscore character)
    // - Calls 'glfwCreateWindow' internally
    // - This function returns a Rust 'Option' which is synonymous with C++17's 'std::optional'
    // - The 'Option::<T>::expect' function here will abort the program on failure to create the window
    let (mut window, _) = glfw.create_window(800, 600, "Hello GLFW", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.set_key_polling(true); // 'glfwSetKeyCallback'
    window.make_current(); // 'glfwMakeContextCurrent'

    // 'gl' is a OpenGL loader (similar to GLAD)
    // The syntax for a closure / lambda is here too "|var| { ... }"
    gl::load_with(|symbol_name| window.get_proc_address(symbol_name));

    // Build and compile shaders
    let vertex_shader = ShaderCompiler::new(gl::VERTEX_SHADER, VERTEX_SHADER_SOURCE).unwrap();
    let fragment_shader  = ShaderCompiler::new(gl::FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE).unwrap();

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

    // Similarly to functions - any statement without a semi-colon can return
    // - Here we are returning a tuple / pair and using "structured bindings" (at least that what you'd call it in C++) to bind them to the variables
    // - You can use tuples like this to return from functions too
    // - This is a decent pattern if you want to setup a variable but keep it const after (note that vao and vbo inside the unsafe block are mutable)
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
    // - Calls 'glfwWindowShouldClose' internally
    // - 'while' loops, 'if' statements and 'for' loops all do not require brackets for the condition
    while !window.should_close() {

        // Our first usage of 'unsafe'
        // - This is requires since the 'gl::ClearColor' function is an unsafe 'C' call
        // - It is unsafe since it does not satisfy the memory safety rules Rust enforces
        unsafe{
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Draw our first triangle
            gl::UseProgram(shader_program);
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        };

        window.swap_buffers(); // 'glfwSwapBuffers'
        glfw.poll_events(); // 'glfwPollEvents'
    }

    unsafe {
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteProgram(shader_program);
    };
}

// These are like 'using' or 'typedef' statements in C++
// - They are defining the types of function pointers
type GetStatusFn = unsafe fn(GLuint, GLenum, *mut GLint);
type GetInfoFn = unsafe fn(GLuint, GLsizei, *mut GLsizei, *mut GLchar);

unsafe fn check_status(shader: GLuint, status: GLenum, get_status: GetStatusFn) -> bool {
    let mut success: GLint = 0;
    get_status(shader, status, &mut success);
    success as u8 == gl::TRUE
}

unsafe fn check_success(shader: GLuint, status: GLenum, get_status: GetStatusFn, get_info: GetInfoFn) -> Result<(), String> {
    if !check_status(shader, status, get_status) {
        let mut buffer: [GLchar; 512] = [0; 512];
        let mut length: GLsizei = 0;
        get_info(shader, buffer.len() as GLsizei, &mut length, buffer.as_mut_ptr());
        Err(CStr::from_ptr(buffer.as_ptr()).to_str().unwrap().to_owned())
    } else {
        Ok(())
    }
}

// For these checking functions we're returning a 'Result<(), String>', the '()' is effectively Rusts "void" type - we're simply using
// the 'Ok' side of the result to signify success
// - Note: at the end of the 'check_status' call; we're using a '?' which is effectively a 'try-return'
//         this means if 'check_status' returns 'Ok' continue as normal, but if 'Err(String)' is returned:
//         return that error from this function. The error types must be the same.
unsafe fn check_compilation_success(shader: GLuint) -> Result<(), String> {
    check_success(shader, gl::COMPILE_STATUS, gl::GetShaderiv, gl::GetShaderInfoLog)?;
    Ok(())
}

unsafe fn check_linking_success(shader: GLuint) -> Result<(), String> {
    check_success(shader, gl::LINK_STATUS, gl::GetProgramiv, gl::GetProgramInfoLog)?;
    Ok(())
}