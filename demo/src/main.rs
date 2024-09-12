use glfw::{self, Context};
use gl;

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
    gl::load_with(|symbol_name| window.get_proc_address(symbol_name));

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
        };

        window.swap_buffers(); // 'glfwSwapBuffers'
        glfw.poll_events(); // 'glfwPollEvents'
    }
}
