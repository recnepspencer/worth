//! Borrowed original attribution DATA; no root, admission or publication authority.
use crate::config::data::{
    AdjacencyPolicy, CascadeDeletePolicy, CrossContextPolicy, MutationConfig,
};
use crate::durability::data::DurableBranchRootSchemaImage;
use crate::durability::source_initial_inputs::recorded::RecordedCanonicalInputs;
use crate::history::data::CanonicalCommitEnvelope;
use crate::schema::data::DescriptorSemanticsVersion;
use worth_foundational::CanonicalDigestId;

/// Original producer policies and schema attribution on an immutable envelope.
///
/// Acquisition borrows the actual retained DATA without loading a body, copying
/// a schema, or granting currentness. Equality includes every original policy
/// and the entire optional introduced schema, rather than its digest alone.
#[derive(Debug, PartialEq, Eq)]
pub struct RelationalOriginalInputObservation<'a> {
    prestate_schema: CanonicalDigestId,
    result_schema: CanonicalDigestId,
    introduced_schema: Option<RelationalIntroducedSchemaObservation<'a>>,
    cascade_delete_policy: &'a CascadeDeletePolicy,
    adjacency_policy: &'a AdjacencyPolicy,
    cross_context_policy: &'a CrossContextPolicy,
}

/// A borrowed observation of the original introduced schema, with no wire door.
/// Whole-image equality compares its declarations and content as well as metadata.
#[derive(PartialEq, Eq)]
pub struct RelationalIntroducedSchemaObservation<'a>(&'a DurableBranchRootSchemaImage);

impl CanonicalCommitEnvelope {
    pub fn original_input_observation(&self) -> Option<RelationalOriginalInputObservation<'_>> {
        self.recorded_original_inputs()
            .map(RelationalOriginalInputObservation::from_recorded)
    }
}
impl<'a> RelationalOriginalInputObservation<'a> {
    fn from_recorded(inputs: &'a RecordedCanonicalInputs) -> Self {
        let RecordedCanonicalInputs {
            result_image,
            prestate_schema,
            result_schema,
            mutation,
        } = inputs;
        let MutationConfig {
            cascade_delete_policy,
            adjacency_policy,
            cross_context_policy,
        } = mutation;
        Self {
            prestate_schema: CanonicalDigestId::new(*prestate_schema),
            result_schema: CanonicalDigestId::new(*result_schema),
            introduced_schema: result_image
                .as_ref()
                .map(RelationalIntroducedSchemaObservation),
            cascade_delete_policy,
            adjacency_policy,
            cross_context_policy,
        }
    }
    pub fn prestate_schema(&self) -> CanonicalDigestId {
        self.prestate_schema
    }
    pub fn result_schema(&self) -> CanonicalDigestId {
        self.result_schema
    }
    pub fn introduced_schema(&self) -> Option<&RelationalIntroducedSchemaObservation<'a>> {
        self.introduced_schema.as_ref()
    }
    pub fn cascade_delete_policy(&self) -> CascadeDeletePolicy {
        *self.cascade_delete_policy
    }
    pub fn adjacency_policy(&self) -> &AdjacencyPolicy {
        self.adjacency_policy
    }
    pub fn cross_context_policy(&self) -> CrossContextPolicy {
        *self.cross_context_policy
    }
}
impl RelationalIntroducedSchemaObservation<'_> {
    pub fn carrier_digest(&self) -> CanonicalDigestId {
        CanonicalDigestId::new(self.0.carrier_digest())
    }
    pub fn schema_root(&self) -> CanonicalDigestId {
        CanonicalDigestId::new(self.0.schema_root())
    }
    pub fn descriptor_semantics_version(&self) -> DescriptorSemanticsVersion {
        self.0.descriptor_semantics_version()
    }
}
impl std::fmt::Debug for RelationalIntroducedSchemaObservation<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RelationalIntroducedSchemaObservation")
            .field("carrier_digest", &self.carrier_digest())
            .field("schema_root", &self.schema_root())
            .field(
                "descriptor_semantics_version",
                &self.descriptor_semantics_version(),
            )
            .finish()
    }
}

#[cfg(test)]
mod tests;
