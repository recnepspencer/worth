//! Process-local transport custody over authentic native Query source owners.
mod contract;
mod custody;
mod diagnostic;
mod protocol;
mod registration;
pub use registration::mount_query_source;
mod transport;
mod view;
mod worker;
use crate::WorthServerProductApplicationAdapterRegistration;
pub use contract::*;
pub use diagnostic::{WorthServerSourceDiagnostic, WorthServerSourceDiagnosticCategory};
pub use protocol::{WorthServerSourceCommand, WorthServerSourceReply};
use std::num::NonZeroUsize;
pub use transport::WorthServerSourceContinuation;
pub use view::{
    CommitPosture as WorthServerSourceCommitPosture, Conclusion as WorthServerSourceConclusion,
    KeyedResolution as WorthServerSourceKeyedResolution, SourcePosture as WorthServerSourcePosture,
    View as WorthServerSourceView,
};

#[derive(Clone, Copy)]
pub struct WorthServerSourceCustodyLimits {
    pub live_operations: NonZeroUsize,
    pub queued_commands: NonZeroUsize,
}
#[must_use = "retain the continuation before installing the accepting registration"]
pub struct WorthServerSourceCustodyMount {
    pub registration: WorthServerProductApplicationAdapterRegistration,
    pub continuation: WorthServerSourceContinuation,
}
