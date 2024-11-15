# Notes & Explanations

This document contains descriptions and expanded explanations to a number of Rust specific code design decisions. The repository readme references this document where additional detail is deemed necessary or interesting.

## Lecture 7: Variadic Generics

Variadic template arguments has been a C++ feature since C++11:
```cpp
struct shader_resource {
    template<typename ...Args>
    static resource_handle create(Args&&... args) { ... }
};
```
Unfortunately Rust's generics doesn't _currently_ support variadic arguments. There are however a number of options we have to emulate similar functionality. The two best options available are as follows:
1. Associated types with traits
2. Rust Macro expansion

Trait defined associated types offers a simple way for a trait implementor to define a type which can then be used as the parameter for the _new_ function:
```rs
// A trait is very similar to a C++20 concept
pub trait ResourceLifecycle: Sized {
    // This line means that the type implementing the trait must specify a description type
    type Description;

    // The following two lines mean the the type implementing the trait must specify
    // associated functions for create and destroy - notice how the create function
    // is taking the description type as a parameter
    fn create(desc: Self::Description) -> ResourceHandle;
    fn destroy(handle: &ResourceHandle);
}

// Here is our 'GenericShaderResource' which handles all of the general logic
// The T is constrained in that it must implement the 'ResourceLifecycle' trait
#[derive(PartialEq, Eq)]
pub struct GenericShaderResource<T: ResourceLifecycle>(ResourceHandle, std::marker::PhantomData<T>);

impl<T: ResourceLifecycle> GenericShaderResource<T> {
    // Notice how similar to C++ we can pull out the arguments meaning that this new function
    // is also required to take the description type associated with the 'ResourceLifecycle'
    // trait
    pub fn new(desc: T::Description) -> Self {
        Self(T::create(desc), std::marker::PhantomData)
    }

    #[must_use] pub fn handle(&self) -> &ResourceHandle { &self.0 }
}

impl<T: ResourceLifecycle> Drop for GenericShaderResource<T> {
    fn drop(&mut self) {
        T::destroy(&self.0);
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum ShaderStage {
    Vertex = gl::VERTEX_SHADER,
    Fragment = gl::FRAGMENT_SHADER
}

// This is now just a "tag" type that implements the trait - it doesn't contain data
#[derive(PartialEq, Eq)]
pub struct ShaderResourceLifecycle;

// This impl block is used to specifically implement 'ResourceLifecycle'
impl ResourceLifecycle for ShaderResourceLifecycle {
    // Note that we're saying our description type is 'ShaderStage'
    // The main issue with this setup is that if we wanted the create function
    // to take multiple arguments, we would have to do one of the following:
    // - Create a new type packing the required objects together (preferred)
    // - Specify our 'Description' type as an anonymous tuple - "type Description = (u32, u32, u32);"
    //
    // The second option would mean that we would need to call the create function like so:
    // "TupleResource::create((1, 2, 3));"
    // Notice the double braces.
    type Description = ShaderStage;

    fn create(stage: ShaderStage) -> ResourceHandle {
        ResourceHandle(unsafe { gl::CreateShader(stage as GLenum) })
    }

    fn destroy(handle: &ResourceHandle) {
        unsafe{ gl::DeleteShader(handle.index()) }
    }
}

// Here is how we would define the type for external use
pub type ShaderResource = GenericShaderResource<ShaderResourceLifecycle>;
```

The other core option is to use Rust's macro system. Most people who have used C or C++'s preprocessor _macro_ system are likely to wince or be generally dismissive when macros are suggested as a solution to a problem. However, Rust's macros aim to solve a number of issues present with C and C++'s preprocessor.

After careful consideration, and with the fact that this project is intended for educational purposes; I elected to move forward with the macro approach:
```rs
// 'macro_rules!' is how we define a macro
macro_rules! shader_resource {
    // This initial portion is a way to define 'patterns' that are matched against
    // for this example though, this is super simple
    (
        // This pattern means the following:
        // 1. We capture visibility so 'pub', 'pub(crate)' or nothing
        // 2. We expect the exact characters "struct"
        // 3. We capture a name 'ident' (identifier) to use as the name
        // 4. We expect the exact characters "(ResourceHandle) {" - this design makes
        //    it nicer to look at in the usage area
        // 5. Another visibility for the new function
        // 6. Exact characters "fn new("
        // 7. Now we capture a variadic number of arguments - as 'argn: argt'
        //    (or `argument_name: argument_type`) separated by commas
        // 8. More exact characters ") -> Self {"
        // 9. The body of the new function, as a variable number of type-trees
        //
        // The rest is just versions of the above
        $struct_vis:vis struct $name:ident (ResourceHandle) {
            $new_vis:vis fn new($($argn:ident: $argt:ty),*) -> Self { $($new_body:tt)* }
            fn drop($handle:ident: &ResourceHandle) { $($drop_body:tt)* }
        }

    ) => {
        // The above "match" is then expanded below with similar rules
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

// Here are the two usage examples
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
```
The macro version does require knowing "another language" to write them, but once written they are very powerful at generating code for you!