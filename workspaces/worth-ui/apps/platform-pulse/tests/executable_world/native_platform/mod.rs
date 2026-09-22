mod contract;
#[cfg(target_os = "windows")]
mod windows;

pub(crate) use contract::NativePlatformPosture;
#[cfg(target_os = "windows")]
pub(crate) use contract::{NativePlatformContract, NativePlatformFailure};
#[cfg(target_os = "windows")]
pub(crate) use windows::{
    WindowsCaptureStream, WindowsNativePlatform, WindowsPreparedWheelInput,
    WindowsProcessBoundNativeClientArea,
};

pub(crate) fn current_platform_posture() -> NativePlatformPosture {
    #[cfg(target_os = "windows")]
    {
        NativePlatformPosture::CertifiedExecutable
    }
    #[cfg(not(target_os = "windows"))]
    {
        NativePlatformPosture::CompileOnly
    }
}
