#[allow(dead_code)]
#[repr(u32)]
#[derive(Debug, PartialEq, Eq)]
enum ErrorCode {
    None = gl::NO_ERROR,
    InvalidEnum = gl::INVALID_ENUM,
    InvalidValue = gl::INVALID_VALUE,
    InvalidOperation = gl::INVALID_OPERATION,
    InvalidFramebufferOperation = gl::INVALID_FRAMEBUFFER_OPERATION,
    StackOverflow = gl::STACK_OVERFLOW,
    StackUnderflow = gl::STACK_UNDERFLOW,
    OutOfMemory = gl::OUT_OF_MEMORY,
}

fn check_for_basic_errors() {
    let mut message = String::new();
    loop {
        let error_code: ErrorCode = unsafe{ std::mem::transmute(gl::GetError()) };
        if error_code == ErrorCode::None {
            break;
        }

        message.push_str(&format!("{:?}\n", error_code));
    }    

    if !message.is_empty() {
        panic!("{}", message);
    }
}

#[repr(u8)]
pub enum BuildMode {
    Debug,
    Release
}

impl BuildMode {
    // Currently, Rust's traits don't support const functions
    pub const fn equals(self, rhs: BuildMode) -> bool { self as u8 == rhs as u8 }
}

#[cfg(debug_assertions)]
const BUILD_MODE: BuildMode = BuildMode::Debug;

#[cfg(not(debug_assertions))]
const BUILD_MODE: BuildMode = BuildMode::Release;

pub const fn build_mode() -> BuildMode { BUILD_MODE }
pub const fn is_debug_mode() -> bool { BUILD_MODE.equals(BuildMode::Debug) }
pub const fn is_release_mode() -> bool { BUILD_MODE.equals(BuildMode::Release) }

pub fn gl_function<F: FnMut()>(mut f: F) {
    f();

    if const { is_debug_mode() } {
        check_for_basic_errors();
    }
}