#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum BuildMode {
    Debug,
    Release
}

impl BuildMode {
    /// Currently, Rust's traits don't support const functions
    pub const fn equals(self, rhs: BuildMode) -> bool { self as u8 == rhs as u8 }
}

#[cfg(debug_assertions)]
const BUILD_MODE: BuildMode = BuildMode::Debug;

#[cfg(not(debug_assertions))]
const BUILD_MODE: BuildMode = BuildMode::Release;

/// Retrieve the current build mode
pub const fn build_mode() -> BuildMode { BUILD_MODE }

/// Returns true if compiling with '#[cfg(debug_assertions)]'
pub const fn is_debug_mode() -> bool { BUILD_MODE.equals(BuildMode::Debug) }

/// Returns true if compiling in release mode '#[cfg(not(debug_assertions))]'
pub const fn is_release_mode() -> bool { BUILD_MODE.equals(BuildMode::Release) }

// ------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum TargetPlatform {
    Windows,
    Mac,
    Linux,
}

impl TargetPlatform {
    /// Currently, Rust's traits don't support const functions
    pub const fn equals(self, rhs: TargetPlatform) -> bool { self as u8 == rhs as u8 }
}

#[cfg(target_os = "windows")]
const TARGET_PLATFORM: TargetPlatform = TargetPlatform::Windows;

#[cfg(target_os = "macos")]
const TARGET_PLATFORM: TargetPlatform = TargetPlatform::Mac;

#[cfg(target_os = "linux")]
const TARGET_PLATFORM: TargetPlatform = TargetPlatform::Mac;

/// Retrieve the current target platform
pub const fn target_platform() -> TargetPlatform { TARGET_PLATFORM }

/// Returns true if the target platform is Windows
pub const fn is_windows() -> bool { TARGET_PLATFORM.equals(TargetPlatform::Windows) }

/// Returns true if the target platform is Mac
pub const fn is_mac() -> bool { TARGET_PLATFORM.equals(TargetPlatform::Mac) }

/// Returns true if the target platform is Linux
pub const fn is_linux() -> bool { TARGET_PLATFORM.equals(TargetPlatform::Linux) }