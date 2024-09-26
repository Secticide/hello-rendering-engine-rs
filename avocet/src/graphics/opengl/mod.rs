use std::ffi::CStr;
use gl::types::*;

// ------------------------------------------------------------------------------------------

#[derive(PartialEq, Eq)]
pub struct ResourceHandle(pub(crate) GLuint);

impl ResourceHandle {
    #[must_use] pub fn index(&self) -> GLuint { self.0 }
}

// ------------------------------------------------------------------------------------------

#[derive(PartialEq, Eq)]
pub struct ShaderProgram(ShaderProgramResource);

impl ShaderProgram {
    pub fn new(vertex_source: &CStr, fragment_source: &CStr) -> Option<Self> {
        let vertex_shader = ShaderCompiler::new(ShaderStage::Vertex, vertex_source)?;
        let fragment_shader = ShaderCompiler::new(ShaderStage::Fragment, fragment_source)?;

        let program = Self(ShaderProgramResource::new());
        let program_index = program.resource().handle().index();

        {
            let _vertex_attacher = ShaderAttacher::new(&program, &vertex_shader);
            let _fragment_attacher = ShaderAttacher::new(&program, &fragment_shader);
            unsafe{ gl::LinkProgram(program_index); }
        }        

        if let Err(message) = unsafe{ check_linking_success(program_index) } {
            eprintln!("Failed to link shader program: {}", message);
            None
        } else {
            Some(program)
        }
    }

    #[must_use] fn resource(&self) -> &ShaderProgramResource { return &self.0; }

    pub fn bind(&self) {
        unsafe{ gl::UseProgram(self.0.handle().index()); }
    }
}

// ------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
enum ShaderStage {
    Vertex = gl::VERTEX_SHADER,
    Fragment = gl::FRAGMENT_SHADER
}

#[derive(PartialEq, Eq)]
struct ShaderCompiler(ShaderResource);

impl ShaderCompiler {
    fn new(stage: ShaderStage, source: &CStr) -> Option<Self> {
        let resource = ShaderResource::new(stage);
        let shader = resource.handle().index();
        
        unsafe{
            gl::ShaderSource(shader, 1, &source.as_ptr(), std::ptr::null());
            gl::CompileShader(shader);
        };

        if let Err(message) = unsafe{ check_compilation_success(shader) } {
            eprintln!("Failed to compile {:?} shader: {}", stage, message);
            None
        } else {
            Some(Self(resource))
        }
    }

    #[must_use] fn resource(&self) -> &ShaderResource { &self.0 }
}

struct ShaderAttacher {
    program: GLuint,
    shader: GLuint
}

impl ShaderAttacher {
    fn new(program: &ShaderProgram, shader: &ShaderCompiler) -> Self {
        let program = program.resource().handle().index();
        let shader = shader.resource().handle().index();

        unsafe{ gl::AttachShader(program, shader) };

        Self{ program, shader }
    }
}

impl Drop for ShaderAttacher {
    fn drop(&mut self) {
        unsafe{ gl::DetachShader(self.program, self.shader); }
    }
}

// ------------------------------------------------------------------------------------------

macro_rules! shader_resource {
    (
        $struct_vis:vis struct $name:ident (ResourceHandle) {
            $new_vis:vis fn new($($argn:ident: $argt:ty),*) -> Self { $($new_body:tt)* }
            fn drop($handle:ident: &ResourceHandle) { $($drop_body:tt)* }
        }

    ) => {
        #[derive(PartialEq, Eq)]
        $struct_vis struct $name (ResourceHandle);

        impl $name {
            $new_vis fn new($($argn: $argt),*) -> Self {
                $($new_body)*
            }

            #[must_use] $struct_vis fn handle(&self) -> &ResourceHandle { &self.0 }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                let $handle: &ResourceHandle = &self.0;
                $($drop_body)*
            }
        }
    };
}

shader_resource!{
    struct ShaderResource(ResourceHandle) {
        fn new(stage: ShaderStage) -> Self {
            Self(ResourceHandle(unsafe{ gl::CreateShader(stage as GLenum) }))
        }

        fn drop(handle: &ResourceHandle) {
            unsafe{ gl::DeleteShader(handle.index()) };
        }
    }
}

shader_resource!{
    struct ShaderProgramResource(ResourceHandle) {
        fn new() -> Self {
            Self(ResourceHandle(unsafe{ gl::CreateProgram() }))
        }

        fn drop(handle: &ResourceHandle) {
            unsafe{ gl::DeleteProgram(handle.index()); }
        }
    }
}

// ------------------------------------------------------------------------------------------

type GetStatusFn = unsafe fn(GLuint, GLenum, *mut GLint);
type GetInfoFn = unsafe fn(GLuint, GLsizei, *mut GLsizei, *mut GLchar);

unsafe fn check_status(shader: GLuint, status: GLenum, get_status: GetStatusFn) -> bool {
    let mut success: GLint = 0;
    get_status(shader, status, &mut success);
    success as u8 == gl::TRUE
}

unsafe fn check_success(shader: GLuint, status: GLenum, get_status: GetStatusFn, get_info: GetInfoFn) -> Result<(), String> {
    if !check_status(shader, status, get_status) {
        let mut length: GLint = 0;
        get_status(shader, gl::INFO_LOG_LENGTH, &mut length);
        let length = length as usize; // Shadowing variable name to another type

        let mut buffer: Vec<u8> = Vec::with_capacity(length);
        get_info(shader, length as GLsizei, std::ptr::null_mut(), buffer.as_mut_ptr() as *mut _);
        buffer.set_len(length - 2); // Unsafe function to set the length since we know how many characters have been written

        Err(String::from_utf8_unchecked(buffer))
    } else {
        Ok(())
    }
}

unsafe fn check_compilation_success(shader: GLuint) -> Result<(), String> {
    check_success(shader, gl::COMPILE_STATUS, gl::GetShaderiv, gl::GetShaderInfoLog)?;
    Ok(())
}

unsafe fn check_linking_success(program: GLuint) -> Result<(), String> {
    check_success(program, gl::LINK_STATUS, gl::GetProgramiv, gl::GetProgramInfoLog)?;
    Ok(())
}