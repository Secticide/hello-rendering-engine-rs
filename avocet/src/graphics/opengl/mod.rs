use std::{
    io::{Result, Error, ErrorKind},
    path::Path,
};
use gl::types::*;

// ------------------------------------------------------------------------------------------

#[derive(PartialEq, Eq)]
pub struct ResourceHandle(pub(crate) GLuint);

impl ResourceHandle {
    #[must_use] pub fn index(&self) -> GLuint { self.0 }
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
    fn new(stage: ShaderStage, path: &Path) -> Result<Self> {
        let source = std::fs::read_to_string(path)?;
        let resource = ShaderResource::new(stage);
        let shader = resource.handle().index();
        
        unsafe{
            let length = source.len() as GLint;
            gl::ShaderSource(shader, 1, &(source.as_ptr() as *const GLchar), &length);
            gl::CompileShader(shader);
        };

        if let Err(error) = check_build_success(&resource) {
            eprint!("{}", error);
            Err(Error::new(ErrorKind::InvalidData, "Failed to build resource."))
        } else {
            Ok(Self(resource))
        }
    }

    #[must_use] fn resource(&self) -> &ShaderResource { &self.0 }
}

// ------------------------------------------------------------------------------------------

#[derive(PartialEq, Eq)]
pub struct ShaderProgram(ShaderProgramResource);

impl ShaderProgram {
    pub fn new<P: AsRef<Path>>(vertex_path: P, fragment_path: P) -> Result<Self> {
        let vertex_shader = ShaderCompiler::new(ShaderStage::Vertex, vertex_path.as_ref())?;
        let fragment_shader = ShaderCompiler::new(ShaderStage::Fragment, fragment_path.as_ref())?;

        let program = Self(ShaderProgramResource::new());
        let program_index = program.resource().handle().index();

        {
            let _vertex_attacher = ShaderAttacher::new(&program, &vertex_shader);
            let _fragment_attacher = ShaderAttacher::new(&program, &fragment_shader);
            unsafe{ gl::LinkProgram(program_index); }
        }

        if let Err(error) = check_build_success(program.resource()) {
            eprint!("{}", error);
            Err(Error::new(ErrorKind::InvalidData, "Failed to build resource."))
        } else {
            Ok(program)
        }
    }

    #[must_use] fn resource(&self) -> &ShaderProgramResource { return &self.0; }

    pub fn bind(&self) {
        unsafe{ gl::UseProgram(self.0.handle().index()); }
    }
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

        impl AsRef<ResourceHandle> for $name {
            fn as_ref(&self) -> &ResourceHandle {
                &self.0
            }
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

impl BuiltResource for ShaderResource {
    #[inline(always)] fn build_stage() -> &'static str { "compilation" }
    #[inline(always)] fn status_flag() -> GLenum { gl::COMPILE_STATUS }

    #[inline(always)] fn get_parameter_fn(&self) -> GetStatusFn { gl::GetShaderiv }
    #[inline(always)] fn get_info_log_fn(&self) -> GetInfoFn { gl::GetShaderInfoLog }

    #[inline(always)] fn name() -> &'static str { "shader" }
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

impl BuiltResource for ShaderProgramResource {
    #[inline(always)] fn build_stage() -> &'static str { "linking" }
    #[inline(always)] fn status_flag() -> GLenum { gl::LINK_STATUS }

    #[inline(always)] fn get_parameter_fn(&self) -> GetStatusFn { gl::GetProgramiv }
    #[inline(always)] fn get_info_log_fn(&self) -> GetInfoFn { gl::GetProgramInfoLog }

    #[inline(always)] fn name() -> &'static str { "program" }
}

// ------------------------------------------------------------------------------------------

type GetStatusFn = unsafe fn(GLuint, GLenum, *mut GLint);
type GetInfoFn = unsafe fn(GLuint, GLsizei, *mut GLsizei, *mut GLchar);

trait BuiltResource: AsRef<ResourceHandle> {
    fn build_stage() -> &'static str;
    fn status_flag() -> GLenum;
    fn get_parameter_fn(&self) -> GetStatusFn;
    fn get_info_log_fn(&self) -> GetInfoFn;
    fn name() -> &'static str;
}

fn get_parameter_value<T: BuiltResource>(resource: &T, parameter_id: GLenum) -> GLint {
    let mut param = 0;
    unsafe{
        resource.get_parameter_fn()(resource.as_ref().index(), parameter_id, &mut param);
    }
    param
}

fn get_info_log<T: BuiltResource>(resource: &T) -> String {
    let length = get_parameter_value(resource, gl::INFO_LOG_LENGTH) as usize;

    let mut buffer: Vec<u8> = Vec::with_capacity(length);
    let result = unsafe{
        resource.get_info_log_fn()(resource.as_ref().index(), length as GLsizei, std::ptr::null_mut(), buffer.as_mut_ptr() as *mut _);
        buffer.set_len(length);
        String::from_utf8_unchecked(buffer)
    };

    result
}

fn check_build_success<T: BuiltResource>(resource: &T) -> std::result::Result<(), String> {
    if get_parameter_value(resource, T::status_flag()) == gl::FALSE as GLint {
        Err(format!("Error {} {} failed:\n{}", T::name(), T::build_stage(), get_info_log(resource)))
    } else {
        Ok(())
    }
}