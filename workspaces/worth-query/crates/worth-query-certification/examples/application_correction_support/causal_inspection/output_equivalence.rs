//! Actual parity equivalence is explained only through fresh source/query diagnostics.
pub mod calculation;
mod demand;
mod journey;
pub mod program;
pub mod readiness;
pub use journey::parity_source_and_output_evaluation;

mod lifecycle;
pub use lifecycle::evaluation_rechecks_each_subject_and_releases_request_funding;

mod retry;
pub use retry::completed_comparison_survives_late_cancellation_and_checkpoint_retry;
