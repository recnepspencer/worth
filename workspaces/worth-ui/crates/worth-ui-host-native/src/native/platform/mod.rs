//! Selects the platform port for the compiled target; implements nothing.
//!
//! Each sibling owns one operating system's pointer-position and motion
//! posture mechanism. Windows samples the message position at button time;
//! Linux caches the client-relative `CursorMoved` position; every other target
//! falls through to the unqualified sibling.

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub(crate) use windows::{
    install_pointer_input, observe_reduced_motion_posture, UiNativePointerInputPort,
};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub(crate) use linux::{
    install_pointer_input, observe_reduced_motion_posture, UiNativePointerInputPort,
};

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
mod unqualified;
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub(crate) use unqualified::{observe_reduced_motion_posture, UiNativePointerInputPort};
