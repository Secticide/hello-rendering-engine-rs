use crate::{
    graphics::ResourceHandle,
    validation::gl_function,
};

pub trait VertexResourceLifecycle {
    fn generate<const N: usize>() -> [ResourceHandle; N];
    fn destroy(indices: &[ResourceHandle]);
}

#[derive(PartialEq, Eq)]
pub struct VertexResource<const N: usize, T: VertexResourceLifecycle>([ResourceHandle; N], std::marker::PhantomData<T>);

impl<const N: usize, T: VertexResourceLifecycle> VertexResource<N, T> {
    pub fn new() -> Self {
        Self (T::generate(), std::marker::PhantomData)
    }

    pub fn handle_at(&self, idx: usize) -> &ResourceHandle { &self.0[idx] }
}

impl<T: VertexResourceLifecycle> VertexResource<1, T> {
    pub fn handle(&self) -> &ResourceHandle { &self.0[0] }
}

impl<const N: usize, T: VertexResourceLifecycle> Drop for VertexResource<N, T> {
    fn drop(&mut self) {
        T::destroy(&self.0);
    }
}

// ------------------------------------------------------------------------------------------

pub struct VertexArrayLifecycle;

impl VertexResourceLifecycle for VertexArrayLifecycle {
    fn generate<const N: usize>() -> [ResourceHandle; N] {
        let mut result = [const { ResourceHandle(0) }; N];
        unsafe{ gl_function(|| gl::GenVertexArrays(N as _, result.as_mut_ptr() as _)) };
        result
    }

    fn destroy(handles: &[ResourceHandle]) {
        unsafe{ gl_function(|| gl::DeleteVertexArrays(handles.len() as _, handles.as_ptr() as _)) };
    }
}

pub struct VertexBufferLifecycle;

impl VertexResourceLifecycle for VertexBufferLifecycle {
    fn generate<const N: usize>() -> [ResourceHandle; N] {
        let mut result = [const { ResourceHandle(0) }; N];
        unsafe { gl_function(|| gl::GenBuffers(N as _, result.as_mut_ptr() as _)) };
        result
    }

    fn destroy(handles: &[ResourceHandle]) {
        unsafe { gl_function(|| gl::DeleteBuffers(handles.len() as _, handles.as_ptr() as _)) };
    }
}

// ------------------------------------------------------------------------------------------

pub type VAOResource = VertexResource<1, VertexArrayLifecycle>;
pub type VBOResource = VertexResource<1, VertexBufferLifecycle>;