use glfw::{ Context, Glfw, GlfwReceiver, OpenGlProfileHint, PWindow, WindowEvent, WindowHint };
use avocet::{ version, validation::ValidationMode };

pub struct WindowManager{
    glfw: Glfw,
    version: version::OpenGLVersion,
}

impl WindowManager {
    pub fn new() -> Self {
        let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
        let version = find_opengl_version(&mut glfw);

        Self { glfw, version }
    }

    pub fn create_window(&mut self, width: u32, height: u32, title: &str) -> Option<(PWindow, GlfwReceiver<(f64, WindowEvent)>)> {
        self.glfw.window_hint(WindowHint::ContextVersion(self.version.major as _, self.version.minor as _));
        self.glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
        self.glfw.window_hint(WindowHint::OpenGlForwardCompat(true));

        let validation_mode = avocet::validation::validation_mode();
        if  validation_mode == ValidationMode::Advanced ||
            (validation_mode == ValidationMode::Dynamic && self.version.supports_debug_message_log()) {
            self.glfw.window_hint(WindowHint::OpenGlDebugContext(true));
        }

        let (mut window, receiver) = self.glfw.create_window(width, height, title, glfw::WindowMode::Windowed)?;
        window.make_current(); // glfwMakeContextCurrent

        // Load OpenGL functions
        gl::load_with(|symbol_name| window.get_proc_address(symbol_name));

        WindowManager::initialise_debug();

        Some((window, receiver))
    }

    pub fn poll_events(&mut self) {
        self.glfw.poll_events();
    }

    fn initialise_debug() {
        let mut flags = 0;
        unsafe{ gl::GetIntegerv(gl::CONTEXT_FLAGS, &mut flags); }
        if (flags as u32 & gl::CONTEXT_FLAG_DEBUG_BIT) != 0 {
            unsafe{ gl::Enable(gl::DEBUG_OUTPUT); }
            unsafe{ gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS); }
            unsafe{ gl::DebugMessageControl(gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE, 0, std::ptr::null(), gl::TRUE); }
        }
    }
}

fn find_opengl_version(glfw: &mut Glfw) -> version::OpenGLVersion {
    // When looking into how GLFW works - when requesting a specific context version
    // it will lock-in on the requested version. Ideally we want the highest version
    // supported by the platform. This is done by not supplying a context version hint.

    // We create a hidden window to create a context and retrieve the OpenGL version
    glfw.window_hint(WindowHint::Visible(false));

    let (mut window, _) = glfw.create_window(1024, 768, "", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW Window");
    window.make_current();

    gl::load_with(|symbol_name| window.get_proc_address(symbol_name));

    glfw.default_window_hints();
    version::get_opengl_version()
}