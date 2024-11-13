#[derive(Debug, Clone, Copy)]
pub struct OpenGLVersion {
    pub major: usize,
    pub minor: usize,
}

impl OpenGLVersion {
    pub fn supports_debug_message_log(&self) -> bool {
        self.major > 3 && self.minor >= 3
    }

    /// Returns the latest possible OpenGL version: 4.6
    pub fn latest() -> Self { Self { major: 4, minor: 6 } }
}

pub fn get_opengl_version() -> OpenGLVersion {
    static mut OPENGL_VERSION: Option<OpenGLVersion> = None;
    if let None = unsafe { OPENGL_VERSION } {
        let version_string = get_opengl_version_string();
        let verion_bytes = version_string.as_bytes();
        unsafe{
            OPENGL_VERSION = Some(OpenGLVersion{
                major: (verion_bytes[0] - 48) as usize,
                minor: (verion_bytes[2] - 48) as usize
            });
        }
    }

    if let Some(version) = unsafe{ OPENGL_VERSION } {
        version
    } else {
        unreachable!()
    }
}

pub fn get_opengl_vendor_string() -> String { get_opengl_string(OpenGLStringId::Vendor) }
pub fn get_opengl_renderer_string() -> String { get_opengl_string(OpenGLStringId::Renderer) }
pub fn get_opengl_version_string() -> String { get_opengl_string(OpenGLStringId::Version) }

#[repr(u32)]
enum OpenGLStringId {
    Vendor = gl::VENDOR,
    Renderer = gl::RENDERER,
    Version = gl::VERSION,
}

fn get_opengl_string(id: OpenGLStringId) -> String {
    let cstr = unsafe{ 
        std::ffi::CStr::from_ptr(gl::GetString(id as gl::types::GLuint) as _)
    };

    cstr.to_string_lossy().into_owned()
}