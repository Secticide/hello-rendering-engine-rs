use crate::{
    graphics::ResourceHandle,
    validation::gl_function,
};

use std::{
    io::{Result, Error, ErrorKind},
    path::Path,
};

use gl::types::*;

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
            gl_function(|| gl::ShaderSource(shader, 1, &(source.as_ptr() as *const GLchar), &length));
            gl_function(|| gl::CompileShader(shader));
        };

        if let Err(error) = check_build_success(&resource) {
            eprint!("{}", error);
            Err(Error::new(ErrorKind::InvalidData, format!("Failed to build resource ({:?} shader).", stage)))
        } else {
            Ok(Self(resource))
        }
    }

    #[must_use] fn resource(&self) -> &ShaderResource { &self.0 }
}

// ------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq)]
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
            unsafe{ gl_function(|| gl::LinkProgram(program_index)); }
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
        unsafe{ gl_function(|| gl::UseProgram(self.0.handle().index())); }
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

        unsafe{ gl_function(|| gl::AttachShader(program, shader)) };

        Self{ program, shader }
    }
}

impl Drop for ShaderAttacher {
    fn drop(&mut self) {
        unsafe{ gl_function(|| gl::DetachShader(self.program, self.shader)) };
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
        #[derive(Debug, PartialEq, Eq)]
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
            let mut id = 0;
            unsafe{ gl_function(|| id = gl::CreateShader(stage as GLenum)) };
            Self(ResourceHandle(id))
        }

        fn drop(handle: &ResourceHandle) {
            unsafe{ gl_function(|| gl::DeleteShader(handle.index())) };
        }
    }
}

impl BuiltResource for ShaderResource {
    const NAME: &'static str = "shader";
    const BUILD_STAGE: &'static str = "compilation";
    const STATUS_FLAG: GLenum = gl::COMPILE_STATUS;

    #[inline(always)] fn get_parameter_fn(&self) -> GetStatusFn { gl::GetShaderiv }
    #[inline(always)] fn get_info_log_fn(&self) -> GetInfoFn { gl::GetShaderInfoLog }

}

shader_resource!{
    struct ShaderProgramResource(ResourceHandle) {
        fn new() -> Self {
            let mut id = 0;
            unsafe{ gl_function(|| id = gl::CreateProgram()) };
            Self(ResourceHandle(id))
        }

        fn drop(handle: &ResourceHandle) {
            unsafe{ gl_function(|| gl::DeleteProgram(handle.index())) };
        }
    }
}

impl BuiltResource for ShaderProgramResource {
    const NAME: &'static str = "program";
    const BUILD_STAGE: &'static str = "linking";
    const STATUS_FLAG: GLenum = gl::LINK_STATUS;

    #[inline(always)] fn get_parameter_fn(&self) -> GetStatusFn { gl::GetProgramiv }
    #[inline(always)] fn get_info_log_fn(&self) -> GetInfoFn { gl::GetProgramInfoLog }

}

// ------------------------------------------------------------------------------------------

type GetStatusFn = unsafe fn(GLuint, GLenum, *mut GLint);
type GetInfoFn = unsafe fn(GLuint, GLsizei, *mut GLsizei, *mut GLchar);

trait BuiltResource: AsRef<ResourceHandle> {
    const NAME: &'static str;
    const BUILD_STAGE: &'static str;
    const STATUS_FLAG: GLenum;

    fn get_parameter_fn(&self) -> GetStatusFn;
    fn get_info_log_fn(&self) -> GetInfoFn;
}

fn get_parameter_value<T: BuiltResource>(resource: &T, parameter_id: GLenum) -> GLint {
    let mut param = 0;
    unsafe{
        gl_function(|| resource.get_parameter_fn()(resource.as_ref().index(), parameter_id, &mut param));
    }
    param
}

fn get_info_log<T: BuiltResource>(resource: &T) -> String {
    let length = get_parameter_value(resource, gl::INFO_LOG_LENGTH) as usize;

    let mut buffer: Vec<u8> = Vec::with_capacity(length);
    let result = unsafe{
        gl_function(|| resource.get_info_log_fn()(resource.as_ref().index(), length as GLsizei, std::ptr::null_mut(), buffer.as_mut_ptr() as *mut _));
        buffer.set_len(length);
        String::from_utf8_unchecked(buffer)
    };

    result
}

fn check_build_success<T: BuiltResource>(resource: &T) -> std::result::Result<(), String> {
    if get_parameter_value(resource, T::STATUS_FLAG) == gl::FALSE as GLint {
        Err(format!("Error {} {} failed:\n{}", T::NAME, T::BUILD_STAGE, get_info_log(resource)))
    } else {
        Ok(())
    }
}