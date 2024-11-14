use glfw::{ Context, Glfw, GlfwReceiver, OpenGlProfileHint, PWindow, WindowEvent, WindowHint };
use avocet::{ version, validation::ValidationMode };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitError {
    InitialiseGlfw(glfw::InitError),
    RetrieveOpenGLVersion,
}

impl From<glfw::InitError> for InitError {
    fn from(value: glfw::InitError) -> Self {
        Self::InitialiseGlfw(value)
    }
}

pub struct WindowManager {
    glfw: Glfw,
    version: version::OpenGLVersion,
}

impl WindowManager {
    pub fn new() -> Result<Self, InitError> {
        let mut glfw = match glfw::init(glfw::fail_on_errors) {
            Ok(glfw) => glfw,
            Err(init_error) => return Err(init_error.into()),
        };

        if let Some(version) = find_opengl_version(&mut glfw) {
            Ok(Self { glfw, version })
        } else {
            Err(InitError::RetrieveOpenGLVersion)
        }
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

fn find_opengl_version(glfw: &mut Glfw) -> Option<version::OpenGLVersion> {
    // When looking into how GLFW works - when requesting a specific context version
    // it will lock-in on the requested version. Ideally we want the highest version
    // supported by the platform. This is done by not supplying a context version hint.

    // The above comment makes sense for Windows and Linux
    // On Mac, if now hint is provided; the driver defaults to 2.1
    // As such we specifically ask for 4.1 on Mac
    if const { avocet::config::is_mac() } {
        glfw.window_hint(WindowHint::ContextVersion(4, 1));
        glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
        glfw.window_hint(WindowHint::OpenGlForwardCompat(true));
    }

    // We create a hidden window to create a context and retrieve the OpenGL version
    glfw.window_hint(WindowHint::Visible(false));

    if let Some((mut window, _)) = glfw.create_window(1, 1, "", glfw::WindowMode::Windowed) {
        window.make_current();
        gl::load_with(|symbol_name| window.get_proc_address(symbol_name));
        glfw.default_window_hints();

        Some(version::get_opengl_version())
    } else {
        None
    }
}