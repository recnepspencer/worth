/// The windowing system a qualified profile drives winit through.
///
/// Manifest field: `windowing_system`. winit chooses its Linux backend at run
/// time from `WAYLAND_DISPLAY` and `DISPLAY`, while the profile is chosen at
/// compile time. This axis is what lets the event loop force the backend the
/// profile was qualified against instead of accepting whichever the session
/// offers: left uncoupled, an X11-profile build landing on a Wayland session
/// would configure `Opaque` — which Wayland also offers — and certify a
/// Wayland surface under a record that says X11.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeWindowingSystem {
    Win32,
    Wayland,
    X11,
}

impl UiNativeWindowingSystem {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Win32 => "win32",
            Self::Wayland => "wayland",
            Self::X11 => "x11",
        }
    }

    /// Const equality, so the selected profile can be bound to the build flag
    /// in a compile-time assertion (`PartialEq` is not callable there).
    /// Compares discriminants rather than enumerating pairs, so a new variant
    /// cannot be left out and silently compare unequal to itself.
    pub(crate) const fn same_as(self, other: Self) -> bool {
        self as u8 == other as u8
    }
}
