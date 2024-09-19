use std::ffi::CStr;
use gl::types::*;

// ------------------------------------------------------------------------------------------

// Since the previous iteration - I've removed the 'new' associated function
// And promoted this internal type to 'pub(crate)' which means it is public only to this crate (anything in 'avocet/src')
#[derive(PartialEq, Eq)]
pub struct ResourceHandle(pub(crate) GLuint);

impl ResourceHandle {
    #[must_use] pub fn index(&self) -> GLuint { self.0 }
}

// ------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum ShaderStage {
    Vertex = gl::VERTEX_SHADER,
    Fragment = gl::FRAGMENT_SHADER
}

// '#[derive(PartialEq, Eq)]' is a way of getting to compiler to implement expected traits, this allows us to compare ShaderResources
#[derive(PartialEq, Eq)]
pub struct ShaderResource(ResourceHandle);

impl ShaderResource {
    pub fn new(stage: ShaderStage) -> Self {
        let handle = unsafe { gl::CreateShader(stage as GLenum) };
        Self(ResourceHandle(handle))
    }

    #[must_use] pub fn handle(&self) -> &ResourceHandle { &self.0 }
}

impl Drop for ShaderResource {
    fn drop(&mut self) {
        unsafe{ gl::DeleteShader(self.0.index()) }
    }
}

// ------------------------------------------------------------------------------------------

#[derive(PartialEq, Eq)]
pub struct ShaderCompiler(ShaderResource);

impl ShaderCompiler {
    pub fn new(stage: ShaderStage, source: &CStr) -> Option<Self> {
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

    #[must_use] pub fn resource(&self) -> &ShaderResource { &self.0 }
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

pub unsafe fn check_linking_success(program: GLuint) -> Result<(), String> {
    check_success(program, gl::LINK_STATUS, gl::GetProgramiv, gl::GetProgramInfoLog)?;
    Ok(())
}