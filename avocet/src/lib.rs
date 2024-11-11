pub mod graphics;
pub mod geometry;
pub mod debugging;

#[macro_export]
macro_rules! const_assert {
    ($cond:expr) => { const _: () = assert!($cond); };
}