#[cfg(worth_ui_certified_executable)]
mod certified_graphics;
mod contract;
#[cfg(all(target_os = "linux", worth_ui_certified_executable))]
mod linux_x11;
#[cfg(worth_ui_certified_executable)]
mod pointer_visual_settlement;
#[cfg(worth_ui_certified_executable)]
mod qualified_record;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(worth_ui_certified_executable)]
pub(crate) use certified_graphics::CertifiedGraphicsRecord;
pub(crate) use contract::NativePlatformPosture;
#[cfg(worth_ui_certified_executable)]
pub(crate) use contract::{
    CreationSurfaceSuccession, NativePlatformContract, NativePlatformFailure,
};

// The build that executes the native courtroom binds exactly one observer
// under posture-neutral names, so no scenario names the vendor it runs on.
#[cfg(all(target_os = "linux", worth_ui_certified_executable))]
pub(crate) use linux_x11::{
    LinuxX11InputEnvironmentDenial as NativeInputEnvironmentDenial,
    LinuxX11NativePlatform as CertifiedNativePlatform,
    LinuxX11ProcessBoundNativeClientArea as CertifiedProcessBoundNativeClientArea,
};
#[cfg(target_os = "windows")]
pub(crate) use windows::{
    WindowsInputEnvironmentDenial as NativeInputEnvironmentDenial,
    WindowsNativePlatform as CertifiedNativePlatform,
    WindowsProcessBoundNativeClientArea as CertifiedProcessBoundNativeClientArea,
};

/// The single appearance scale row the certified lane executes at, in
/// milli: a Windows host at 150 % and the X11 lane's server qualified to
/// the matching 144 dpi. One owner, so the courtroom's physical expectations
/// and the observer's qualification cannot drift apart.
#[cfg(worth_ui_certified_executable)]
pub(crate) const CERTIFIED_SCALE_MILLI: u32 = 1_500;

/// A logical extent at the certified scale. The product's window extents are
/// chosen so the projection is integral; a fractional pixel here would mean
/// the expectation, not the product, was rounding.
#[cfg(worth_ui_certified_executable)]
pub(crate) fn certified_physical_extent(logical: [u32; 2]) -> [u32; 2] {
    logical.map(|axis| {
        let milli = axis * CERTIFIED_SCALE_MILLI;
        assert!(
            milli.is_multiple_of(1_000),
            "{axis} logical pixels are not integral at {CERTIFIED_SCALE_MILLI} milli"
        );
        milli / 1_000
    })
}

/// The requirement id a phase's world evidence is filed under. The id names
/// the lane that produced it, so a Linux run never emits a Windows record.
#[cfg(worth_ui_certified_executable)]
pub(crate) fn world_requirement(phase: u8) -> String {
    #[cfg(target_os = "windows")]
    {
        format!("P{phase}-WINDOWS-WORLD-01")
    }
    #[cfg(target_os = "linux")]
    {
        format!("P{phase}-LINUX-X11-WORLD-01")
    }
}

pub(crate) fn current_platform_posture() -> NativePlatformPosture {
    #[cfg(worth_ui_certified_executable)]
    {
        NativePlatformPosture::CertifiedExecutable
    }
    #[cfg(all(not(worth_ui_certified_executable), worth_ui_product_executable))]
    {
        NativePlatformPosture::NotYetCertifiedExecutable
    }
    #[cfg(not(worth_ui_product_executable))]
    {
        NativePlatformPosture::CompileOnly
    }
}
