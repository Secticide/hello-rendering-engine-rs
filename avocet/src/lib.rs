pub mod graphics;
pub mod geometry;
pub mod validation;
pub mod config;
pub mod version;

#[macro_export]
macro_rules! const_assert {
    ($cond:expr) => { const _: () = assert!($cond); };
}