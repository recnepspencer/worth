//! The Host's one thread-creation site.
//!
//! Category: host background dispatch. Two sources use it, each starting at
//! most one thread per owner and stopping it when the owner goes away: an
//! owned session's call worker (`call_worker`) and a Home's commit notifier
//! (`commit_watch`). Computation never runs here; it stays with Execution.

/// Starts a named thread running `body`. A refusal by the OS is returned, never
/// raised, so each caller fails closed with its own typed outcome.
#[expect(
    clippy::disallowed_methods,
    reason = "host background dispatch: one owned-session call worker and one Home commit notifier, named and owner-scoped"
)]
pub(crate) fn spawn_named(name: &str, body: impl FnOnce() + Send + 'static) -> std::io::Result<()> {
    std::thread::Builder::new()
        .name(name.to_owned())
        .spawn(body)
        .map(drop)
}
