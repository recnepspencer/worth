use super::UiNativePlatformProfileIdentity;

/// The qualification verdict for the target this build was compiled for.
///
/// This is a resolved value rather than a boolean: `arch_laws` #5 permits
/// booleans to render outcomes but never to encode them, and collapsing the two
/// denials into one would lose the architecture denial on, for example,
/// `aarch64-unknown-linux-gnu`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeQualifiedTarget {
    Qualified(UiNativePlatformProfileIdentity),
    UnqualifiedOperatingSystem,
    UnqualifiedArchitecture,
}

impl UiNativeQualifiedTarget {
    /// Resolved for the compiled target. Consumers receive this already
    /// resolved so their own validation performs no lookup of its own.
    pub const RESOLVED: Self = Self::resolve(
        cfg!(any(target_os = "windows", target_os = "linux")),
        cfg!(target_arch = "x86_64"),
    );

    const fn resolve(qualified_operating_system: bool, qualified_architecture: bool) -> Self {
        if !qualified_operating_system {
            return Self::UnqualifiedOperatingSystem;
        }
        if !qualified_architecture {
            return Self::UnqualifiedArchitecture;
        }
        Self::Qualified(super::ACTIVE_PROFILE.identity)
    }
}
