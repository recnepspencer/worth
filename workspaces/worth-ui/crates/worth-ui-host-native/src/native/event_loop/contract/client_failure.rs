/// Which callback the event loop had invoked when its client refused.
///
/// The loop attaches this, never the client. The loop is the only party that
/// knows which of the nine fallible callbacks is in flight, so a client
/// cannot name the wrong one: it returns a [`UiNativeEventLoopClientDenial`]
/// and nothing else. Pairing happens once, in
/// `event_loop::client_invocation`, where all nine wrappers sit side by side.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeEventLoopClientCallback {
    InstallObservationClock,
    ObservationTimeReady,
    InstallApplicationReadiness,
    ApplicationReadinessReady,
    NativeSurfaceReady,
    RedrawReady,
    PhysicalWorkProgressed,
    NativeObservationsReady,
    ExternalCloseRequested,
}

/// Why a client refused a callback.
///
/// Every variant is reachable from the qualified client today; this is a
/// record of refusals that happen, not a taxonomy of refusals that might.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeEventLoopClientDenial {
    /// The callback installs a resource the client already holds.
    AlreadyInstalled,
    /// The grant's generation does not advance the last one the client
    /// observed, so honoring it would replay settled work.
    StaleGrant,
    /// The client has no bound native surface to satisfy the callback
    /// against.
    SurfaceUnbound,
    /// The client holds a surface but could not rebind it to the scale the
    /// grant carries.
    SurfaceScaleRebindDenied,
    /// The client has no application left to launch a native surface for.
    ApplicationUnavailable,
    /// Launching the application's native surface was denied. Any cleanup
    /// the denial carried is retained by the client for its `close`.
    ApplicationLaunchDenied,
    /// The client's own progression of the granted work was denied.
    ClientProgressDenied,
    /// The application the client drives could not progress — or could not
    /// be activated against — the granted work.
    ApplicationProgressDenied,
    /// Draining the admitted native observation batches was denied.
    ObservationDrainDenied,
    /// This client does not implement the callback the loop invoked.
    Unsupported,
    /// The client refused, and its own internals did not carry a reason out.
    ///
    /// This names a gap rather than a cause. Every occurrence marks an owner
    /// below the client boundary that still refuses with `()`; a stop report
    /// carrying it says which callback refused but not why, which is the
    /// state this type exists to retire. Greppable on purpose.
    Unattributed,
}

/// A client's refusal of one callback, named on both axes.
///
/// This is what reaches a stop report. Before it existed the report carried
/// `ApplicationDriver` for every refusal, which is why the qualified client
/// grew `eprintln!` diagnostics to distinguish four of them: the type could
/// not say what the product already knew.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiNativeEventLoopClientFailure {
    callback: UiNativeEventLoopClientCallback,
    denial: UiNativeEventLoopClientDenial,
}

impl UiNativeEventLoopClientFailure {
    pub(in crate::native::event_loop) const fn refused(
        callback: UiNativeEventLoopClientCallback,
        denial: UiNativeEventLoopClientDenial,
    ) -> Self {
        Self { callback, denial }
    }

    pub const fn callback(&self) -> UiNativeEventLoopClientCallback {
        self.callback
    }

    pub const fn denial(&self) -> UiNativeEventLoopClientDenial {
        self.denial
    }
}
