//! Original producer DATA needed by service-free complete reconstruction.
use std::sync::Arc;
use worth_execution::ExecutionRequest;

use crate::config::data::MutationConfig;
use crate::history::data::canonical_commit_envelope::{
    recorded_inputs::RecordedCanonicalInputs, schema_carrier::RecordedBranchSchemaImage,
};

/// The producer's two selected schema roles before their recorded images are bound.
pub(crate) struct OriginalCanonicalInputs<Schema> {
    pub(crate) result_schema: Arc<Schema>,
    pub(crate) prestate_schema: Arc<Schema>,
    pub(crate) mutation: MutationConfig,
}

/// Both original carriers share their admitted backing for service-free reconstruction.
#[derive(Debug, Clone)]
pub(crate) struct CapturedOriginalCanonicalInputs {
    pub(crate) result_schema: RecordedBranchSchemaImage,
    pub(crate) prestate_schema: RecordedBranchSchemaImage,
    pub(crate) mutation: MutationConfig,
}

impl CapturedOriginalCanonicalInputs {
    pub(crate) fn recorded(&self) -> RecordedCanonicalInputs {
        let prestate_schema = self.prestate_schema.carrier_digest();
        let result_schema = self.result_schema.carrier_digest();
        RecordedCanonicalInputs {
            prestate_schema,
            result_schema,
            mutation: self.mutation.clone(),
            result_image: (prestate_schema != result_schema).then(|| self.result_schema.clone()),
        }
    }
}

impl super::RelationalCommitArtifact {
    pub(crate) fn set_original_inputs(&mut self, inputs: CapturedOriginalCanonicalInputs) {
        self.original_inputs = Some(inputs);
    }

    pub(crate) fn original_inputs(&self) -> Option<&CapturedOriginalCanonicalInputs> {
        self.original_inputs.as_ref()
    }

    /// Reserve the existing catalog allocation before the durable cutover.
    /// Existing input schema allocations are shared, not charged a second time.
    pub(crate) fn prepare_catalog_owner(
        mut self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, crate::execution::denial_cause::RelationalExecutionDenialCause> {
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .ok()
            .map(|(layout, _)| layout.pad_to_align());
        let bytes = layout
            .and_then(|layout| u64::try_from(layout.size()).ok())
            .ok_or(crate::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow)?;
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(crate::execution::denial_cause::RelationalExecutionDenialCause::from)?;
        execution
            .check_live()
            .map_err(crate::execution::denial_cause::RelationalExecutionDenialCause::from)?;
        self.allocation_memory = memory;
        Ok(Arc::new(self))
    }

    /// A same-subject replacement may carry original DATA without recreating it.
    pub(crate) fn preserve_original_inputs(
        &mut self,
        previous: &Self,
    ) -> Result<(), super::RelationalCommitArtifactDenial> {
        if self.identity != previous.identity
            || self.parentage != previous.parentage
            || self.roots != previous.roots
            || self.schema_version != previous.schema_version
            || self.canonical_payload_digest()? != previous.canonical_payload_digest()?
        {
            return Err(super::RelationalCommitArtifactDenial::RootLinkage);
        }
        if previous.original_inputs.is_none() {
            return Ok(());
        }
        let same_cell = Arc::ptr_eq(&self.envelope, &previous.envelope);
        let same_bound_source = match (
            self.envelope.source_placement(),
            previous.envelope.source_placement(),
        ) {
            (Some((entry, source)), Some((prior_entry, prior_source))) => {
                Arc::ptr_eq(&source, &prior_source) && entry.as_ref() == prior_entry.as_ref()
            }
            _ => false,
        };
        if !same_cell && !same_bound_source {
            return Err(super::RelationalCommitArtifactDenial::RootLinkage);
        }
        self.original_inputs = previous.original_inputs.clone();
        Ok(())
    }
}
