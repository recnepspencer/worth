//! Descriptive completion fields from the actual finished bootstrap publication.
use super::super::WorthQueryPrimaryGraphPublication;
use super::{codec, denial, WorthQueryPrimaryGraphInstallationDenial as Denial};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

pub(super) struct BootstrapPublicationRecord {
    pub values: [AspectValue; 8],
    pub funding: Option<ExecutionMemoryReservation>,
}
impl BootstrapPublicationRecord {
    pub fn prepare(
        binding: &ApplicationSchemaBindingIdentity,
        publication: &WorthQueryPrimaryGraphPublication,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let count = |value| {
            u64::try_from(value)
                .map(AspectValue::UInt64)
                .map_err(|_| denial("Home bootstrap completion count overflow"))
        };
        let (binding, funding) = codec::encode(binding, execution)?;
        Ok(Self {
            values: [
                AspectValue::UInt64(2),
                AspectValue::String(InternedString::from(binding)),
                AspectValue::UInt64(publication.bootstrap_commit_id().0),
                count(publication.principal_binding_count())?,
                count(publication.identity_index_count())?,
                count(publication.application_equality_index_count())?,
                count(publication.policy_entity_count())?,
                count(publication.policy_relation_count())?,
            ],
            funding,
        })
    }
}
