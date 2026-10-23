//! Mechanism-specific historical material supplied after current disclosure.
use super::ApplicationCorrectionPreImage;

/// Historical material selected by the original operation's installed mechanism.
/// This read-only value supplies no current operation authority.
pub enum ApplicationCorrectionSourceMaterial<'source, Schema> {
    RecordedInverse(&'source dyn ApplicationCorrectionPreImage<Schema>),
    Compensation,
}
