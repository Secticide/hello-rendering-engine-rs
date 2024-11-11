mod shader;
mod buffers;

use gl::types::GLuint;

pub use shader::*;
pub use buffers::*;

#[repr(transparent)]
#[derive(PartialEq, Eq)]
pub struct ResourceHandle(pub(crate) GLuint);

impl ResourceHandle {
    #[must_use] pub fn index(&self) -> GLuint { self.0 }
}

// static_assert to ensure ResourceHandle and GLuint are the same size
crate::const_assert!(std::mem::size_of::<ResourceHandle>() == std::mem::size_of::<GLuint>());