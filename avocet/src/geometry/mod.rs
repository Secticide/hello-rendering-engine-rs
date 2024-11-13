use crate::{
    graphics::opengl::{VAOResource, VBOResource},
    validation::gl_function,
};

#[allow(dead_code)]
pub struct Triangle {
    vertex_array_object: VAOResource,
    vertex_buffer_object: VBOResource,
}

impl Triangle {
    const VERTICES: [f32; 9] = [
        -0.5, -0.5, 0.0, // left  
        0.5, -0.5, 0.0, // right 
        0.0,  0.5, 0.0  // top   
    ];

    pub fn new() -> Self {
        let vertex_array_object = VAOResource::new();
        gl_function(|| unsafe{ gl::BindVertexArray(vertex_array_object.handle().index()) });

        let vertex_buffer_object = VBOResource::new();
        gl_function(|| unsafe{ gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer_object.handle().index()) });
        gl_function(|| unsafe{ gl::BufferData(gl::ARRAY_BUFFER, std::mem::size_of_val(&Triangle::VERTICES) as _, Triangle::VERTICES.as_ptr() as _, gl::STATIC_DRAW) });

        gl_function(|| unsafe{ gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (std::mem::size_of::<f32>() * 3) as _, std::ptr::null()) });
        gl_function(|| unsafe{ gl::EnableVertexAttribArray(0) });

        Self { vertex_array_object, vertex_buffer_object }
    }

    pub fn draw(&self) {
        gl_function(|| unsafe{ gl::BindVertexArray(self.vertex_array_object.handle().index()) });
        gl_function(|| unsafe{ gl::DrawArrays(gl::TRIANGLES, 0, 3) });
    }
}