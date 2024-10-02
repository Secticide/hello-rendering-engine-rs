use glfw::{ Context, Glfw, GlfwReceiver, OpenGlProfileHint, PWindow, WindowEvent, WindowHint };

pub struct WindowManager(Glfw);

impl WindowManager {
    pub fn new() -> Self {
        WindowManager(glfw::init(glfw::fail_on_errors).unwrap())
    }

    pub fn create_window(&mut self, width: u32, height: u32, title: &str) -> Option<(PWindow, GlfwReceiver<(f64, WindowEvent)>)> {
        self.0.window_hint(WindowHint::ContextVersionMajor(3));
        self.0.window_hint(WindowHint::ContextVersionMinor(3));
        self.0.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
        self.0.window_hint(WindowHint::OpenGlForwardCompat(true));

        let (mut window, receiver) = self.0.create_window(width, height, title, glfw::WindowMode::Windowed)?;
        window.make_current(); // glfwMakeContextCurrent

        // Load OpenGL functions
        gl::load_with(|symbol_name| window.get_proc_address(symbol_name));

        Some((window, receiver))
    }

    pub fn poll_events(&mut self) {
        self.0.poll_events();
    }
}