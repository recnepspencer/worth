//! What happens to the product process if this test process dies without
//! running `Drop`. The answer is observed at launch, not declared per target:
//! the same Linux binary is contained under `xvfb-run` and leaked on a
//! desktop display, and the evidence must say which.
#[cfg(target_os = "linux")]
use std::ffi::OsStr;
#[cfg(target_os = "linux")]
use std::path::Path;

/// Each variant exists only on the targets where it can be the truth.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeProcessContainment {
    /// A kill-on-close job object: the kernel terminates the product when the
    /// last handle to the job closes, however this process ends.
    #[cfg(target_os = "windows")]
    JobObject,
    /// The product's X display is private to this lane's `xvfb-run`, whose
    /// EXIT trap tears the server down when the lane's command exits; the
    /// product then exits on losing its display (Xlib's I/O-error exit, which
    /// is containment evidence, not a typed product stop). Kill shapes,
    /// measured 2026-09-21: the lane process SIGKILLed -> Xvfb and product
    /// gone within a second; `xvfb-run` itself SIGKILLed -> its trap never
    /// runs, Xvfb and the product survive reparented and the `xvfb-run.*`
    /// directory leaks. So this posture holds exactly while `xvfb-run`
    /// outlives the lane; the `native-platform` lane owns that condition
    /// (design D14) and polls for the product's disappearance with a timeout
    /// rather than trusting a recorded latency.
    #[cfg(target_os = "linux")]
    DisplayLifetime,
    /// Only `Drop`'s two-second termination guards the product; a lane killed
    /// outright leaks it. The `native-platform` lane refuses this posture.
    #[cfg(not(target_os = "windows"))]
    DropTerminationOnly,
}

impl NativeProcessContainment {
    pub(crate) fn observe() -> Self {
        #[cfg(target_os = "windows")]
        {
            Self::JobObject
        }
        #[cfg(target_os = "linux")]
        {
            if display_is_lane_private(std::env::var_os("XAUTHORITY").as_deref()) {
                Self::DisplayLifetime
            } else {
                Self::DropTerminationOnly
            }
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            Self::DropTerminationOnly
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            #[cfg(target_os = "windows")]
            Self::JobObject => "job_object",
            #[cfg(target_os = "linux")]
            Self::DisplayLifetime => "display_lifetime",
            #[cfg(not(target_os = "windows"))]
            Self::DropTerminationOnly => "drop_termination_only",
        }
    }
}

/// Debian's `xvfb-run` writes the cookie for the server it owns into a
/// `mktemp -d -t xvfb-run.XXXXXX` directory and removes both at exit. A
/// cookie anywhere else belongs to a display that outlives this lane.
#[cfg(target_os = "linux")]
fn display_is_lane_private(xauthority: Option<&OsStr>) -> bool {
    const XVFB_RUN_MARKER: &str = "xvfb-run.";
    xauthority.is_some_and(|path| {
        Path::new(path)
            .parent()
            .and_then(Path::file_name)
            .and_then(OsStr::to_str)
            .is_some_and(|directory| directory.starts_with(XVFB_RUN_MARKER))
    })
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use std::ffi::OsStr;

    #[test]
    fn only_a_cookie_inside_an_xvfb_run_directory_is_lane_private() {
        assert!(super::display_is_lane_private(Some(OsStr::new(
            "/tmp/xvfb-run.Ab12Cd/Xauthority"
        ))));
        assert!(!super::display_is_lane_private(Some(OsStr::new(
            "/run/user/1000/.mutter-Xwaylandauth.JWYHV3"
        ))));
        assert!(!super::display_is_lane_private(Some(OsStr::new(
            "/home/runner/.Xauthority"
        ))));
        assert!(!super::display_is_lane_private(Some(OsStr::new(
            "/tmp/xvfb-run.Ab12Cd"
        ))));
        assert!(!super::display_is_lane_private(None));
    }
}
