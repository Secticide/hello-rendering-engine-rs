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

pub fn gl_function<F: FnMut()>(mut f: F) {
    f();
    check_for_basic_errors();
}