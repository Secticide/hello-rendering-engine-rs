use std::path::PathBuf;
use glfw::PWindow;
use avocet::graphics as ag;

use crate::util::{self, WindowManager};

fn get_test_asset_path(filename: &str) -> PathBuf {
    const CARGO_MANIFEST_DIR: &'static str = std::env!("CARGO_MANIFEST_DIR");
    const TEST_ASSETS_DIR_NAME: &'static str = "testing";

    let directory_separator_count = 2;
    let mut path = PathBuf::with_capacity(
        CARGO_MANIFEST_DIR.len() +
        TEST_ASSETS_DIR_NAME.len() +
        directory_separator_count +
        filename.len());
    
    path.push(CARGO_MANIFEST_DIR);
    path.push(TEST_ASSETS_DIR_NAME);
    path.push(filename);

    path
}

fn setup() -> (WindowManager, PWindow) {
    let mut manager = util::WindowManager::new().unwrap();
    let (window, _) = manager.create_window(util::WindowConfig::hidden()).unwrap();
    (manager, window)
}

#[test]
fn shader_program() {
    let (_managerm, _window) = setup();

    missing_vertex_shader();
    missing_fragment_shader();
    
    broken_vertex_shader();
    broken_fragment_shader();
}

fn missing_vertex_shader() {
    let vertex_path = get_test_asset_path("missing_file.glsl");
    let fragment_path = get_test_asset_path("monochrome_frag.glsl");

    let result = ag::ShaderProgram::new(vertex_path, fragment_path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);
}

fn missing_fragment_shader() {
    let vertex_path = get_test_asset_path("identity_vert.glsl");
    let fragment_path = get_test_asset_path("missing_file.glsl");

    let result = ag::ShaderProgram::new(vertex_path, fragment_path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);
}

fn broken_vertex_shader() {
    let vertex_path = get_test_asset_path("broken_identity_vert.glsl");
    let fragment_path = get_test_asset_path("monochrome_frag.glsl");

    let result = ag::ShaderProgram::new(vertex_path, fragment_path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidData);
}

fn broken_fragment_shader() {
    let vertex_path = get_test_asset_path("identity_vert.glsl");
    let fragment_path = get_test_asset_path("broken_monochrome_frag.glsl");

    let result = ag::ShaderProgram::new(vertex_path, fragment_path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidData);
}