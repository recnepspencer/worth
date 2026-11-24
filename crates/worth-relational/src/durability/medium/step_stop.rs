use super::{RelationalDurabilityUnresolved, RelationalDurablePublicationRefusal};

/// Why the durable step did not make a commit or transition durable.
#[derive(Debug, Clone)]
pub(crate) enum RelationalDurableStepStop {
    /// Nothing durable changed; the position or sequence is not consumed.
    Refused(RelationalDurablePublicationRefusal),
    /// The publication's durable fate is unknown until the runtime is
    /// reopened.
    Unresolved(RelationalDurabilityUnresolved),
}
