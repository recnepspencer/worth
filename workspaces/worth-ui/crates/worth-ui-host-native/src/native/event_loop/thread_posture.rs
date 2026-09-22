#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum UiNativeEventLoopThreadPosture {
    #[default]
    MainThreadRequired,
    CertificationWorker,
}

impl UiNativeEventLoopThreadPosture {
    #[cfg(target_os = "windows")]
    pub(super) fn configure<T>(self, builder: &mut winit::event_loop::EventLoopBuilder<T>) {
        use winit::platform::windows::EventLoopBuilderExtWindows;

        match self {
            Self::MainThreadRequired => {
                builder.with_any_thread(false);
            }
            Self::CertificationWorker => {
                builder.with_any_thread(true);
            }
        }
    }

    /// Renders the posture for the Linux extension traits, which take the
    /// permission as a flag; the windowing-system owner applies it through
    /// the trait of the backend it forces.
    #[cfg(target_os = "linux")]
    pub(super) const fn any_thread(self) -> bool {
        match self {
            Self::MainThreadRequired => false,
            Self::CertificationWorker => true,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::MainThreadRequired => "main-thread-required",
            Self::CertificationWorker => "certification-worker",
        }
    }
}
