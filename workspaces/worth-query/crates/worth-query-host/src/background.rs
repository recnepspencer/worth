//! Host background dispatch: Home work and commit notifications that run off
//! the caller's thread, so a single-threaded UI never blocks on a commit and
//! never polls to learn that one landed. No async runtime is involved.
//!
//! Threads start only at [`thread_site::spawn_named`]. Each thread holds no
//! Home runtime of its own: a call borrows its owned session's hold for the
//! length of the call, and a notifier holds only its Home's commit pulse, so
//! neither ever keeps a Home from closing.
pub(crate) mod call_worker;
pub(crate) mod commit_watch;
pub(crate) mod pending;
mod thread_site;
