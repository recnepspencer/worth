//! Genuine, deliberately swallowed decision polls in the native consumer.
use super::*;

pub(super) fn poll<Schema, Binding>(reader: &DecisionReader<'_, '_, '_, Schema, Binding>)
where
    Schema: TopologySchemaBinding,
    Binding: worth_query_decl::facade::application_operation::ApplicationMutationBinding<Schema>,
{
    #[cfg(test)]
    observation::before_poll();
    let first = reader.checkpoint();
    #[cfg(test)]
    observation::record(first);
    // Polling again must retain the first stop, even if another cause arrives.
    let second = reader.checkpoint();
    #[cfg(test)]
    observation::record(second);
    let _ = (first, second);
}

#[cfg(test)]
mod observation;
#[cfg(test)]
pub(crate) use observation::{candidate_started, PollObservation};
