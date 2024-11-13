use crate::{
    config,
    version,
};

use gl::types::GLint;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum ValidationMode {
    None,
    Basic,
    Advanced,
    Dynamic
}

impl ValidationMode {
    /// Currently, Rust's traits don't support const functions
    pub const fn equals(self, rhs: ValidationMode) -> bool { self as u8 == rhs as u8 }
}

/// Returns the validation mode based on platform and build configuration
pub const fn validation_mode() -> ValidationMode {
    if config::is_release_mode() {
        ValidationMode::None
    } else if config::is_windows() {
        ValidationMode::Advanced
    } else if config::is_mac() {
        ValidationMode::Basic
    } else {
        ValidationMode::Dynamic
    }
}

const fn should_validate() -> bool { !validation_mode().equals(ValidationMode::None) }

// ------------------------------------------------------------------------------------------

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
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

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
enum DebugSource {
    API = gl::DEBUG_SOURCE_API,
    WindowSystem = gl::DEBUG_SOURCE_WINDOW_SYSTEM,
    ShaderCompiler = gl::DEBUG_SOURCE_SHADER_COMPILER,
    ThirdParty = gl::DEBUG_SOURCE_THIRD_PARTY,
    Application = gl::DEBUG_SOURCE_APPLICATION,
    Other = gl::DEBUG_SOURCE_OTHER,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
enum DebugType {
    Error = gl::DEBUG_TYPE_ERROR,
    DeprecatedBehaviour = gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR,
    UndefinedBehaviour = gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR,
    Portability = gl::DEBUG_TYPE_PORTABILITY,
    Performance = gl::DEBUG_TYPE_PERFORMANCE,
    Marker = gl::DEBUG_TYPE_MARKER,
    PushGroup = gl::DEBUG_TYPE_PUSH_GROUP,
    PopGroup = gl::DEBUG_TYPE_POP_GROUP,
    Other = gl::DEBUG_TYPE_OTHER,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
enum DebugSeverity {
    High = gl::DEBUG_SEVERITY_HIGH,
    Medium = gl::DEBUG_SEVERITY_MEDIUM,
    Low = gl::DEBUG_SEVERITY_LOW,
    Notification = gl::DEBUG_SEVERITY_NOTIFICATION,
}

struct DebugInfo {
    severity: DebugSeverity,
    message: String,
}

fn max_message_length() -> usize {
    static mut MAX_DEBUG_MESSAGE_LENGTH: Option<GLint> = None;
    // This branch allows us to only retrieve the value if we haven't already
    if let None = unsafe{ MAX_DEBUG_MESSAGE_LENGTH } {
        let mut length = 0;
        unsafe{ gl::GetIntegerv(gl::MAX_DEBUG_MESSAGE_LENGTH, &mut length); }
        unsafe{ MAX_DEBUG_MESSAGE_LENGTH = Some(length) };
    }

    if let Some(length) = unsafe{ MAX_DEBUG_MESSAGE_LENGTH } {
        length as usize
    } else {
        unreachable!()
    }
}

#[must_use]
fn get_next_message() -> Option<DebugInfo> {
    let mut message: Vec<u8> = Vec::with_capacity(max_message_length());
    let mut source = 0;
    let mut debug_type = 0;
    let mut severity = 0;
    let mut id = 0;
    let mut length = 0;

    let message_count = unsafe{
        gl::GetDebugMessageLog(
            1,
            message.capacity() as _,
            &mut source,
            &mut debug_type,
            &mut id,
            &mut severity,
            &mut length,
            message.as_mut_ptr() as _
        )
    };

    if length > 0 {
        unsafe{ message.set_len(length as usize); }
    }

    if message_count > 0 {
        let message = unsafe{ String::from_utf8_unchecked(message) };

        // I'll add an explanation for these 'std::mem::transmute's into the notes document
        //
        // But, for now they are just 'reinterpret_cast's
        let source: DebugSource = unsafe{ std::mem::transmute(source) };
        let debug_type: DebugType = unsafe{ std::mem::transmute(debug_type) };
        let severity: DebugSeverity = unsafe{ std::mem::transmute(severity) };

        Some(DebugInfo {
            severity,
            message: format!(
                "Source: {:?}; Type: {:?}; Severity: {:?}\n{}",
                source,
                debug_type,
                severity,
                message
            ),
        })
    } else {
        None
    }
}

// ------------------------------------------------------------------------------------------

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

fn check_for_advanced_errors() {
    let mut message = String::new();
    while let Some(debug_info) = get_next_message() {
        if let DebugSeverity::Notification = debug_info.severity {
            eprintln!("{}", debug_info.message);
        } else {
            message.push_str(&debug_info.message);
            message.push('\n');
        }
    }

    if !message.is_empty() {
        panic!("{}", message);
    }
}

// ------------------------------------------------------------------------------------------

fn check_for_errors() {
    let validation_mode = const { validation_mode() };
    match validation_mode {
        ValidationMode::Basic => check_for_basic_errors(),
        ValidationMode::Advanced => check_for_advanced_errors(),
        ValidationMode::Dynamic => {
            let version = version::get_opengl_version();
            if version.supports_debug_message_log() {
                check_for_advanced_errors();
            } else {
                check_for_basic_errors();
            }
        },
        _ => {},
    }
}

#[inline]
pub fn gl_function<F: FnMut()>(mut f: F) {
    f();

    // 'should_validate' is a compile time check
    if const { should_validate() } {
        check_for_errors();
    }
}