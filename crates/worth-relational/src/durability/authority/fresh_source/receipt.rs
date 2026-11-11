//! Fixed original producer DATA, with no second operational owner graph.
use super::{refused, Stop};
use crate::branch::{
    RelationalBranchRoot, RelationalBranchRootSchemaAuthority, RelationalBranchVersion,
};
use crate::durability::medium::RelationalDurableColdSource;
use crate::runtime::{OriginalBranchSchemaInputsDenial, RelationalRuntime};
use std::sync::{Arc, Weak};
use worth_execution::ExecutionRequest;
use worth_foundational::FoundationalBranchReferenceGeneration;

#[derive(Debug)]
pub struct FreshRelationalSourceAdmission {
    runtime_instance_id: u64,
    generation: FoundationalBranchReferenceGeneration,
    truth_version: RelationalBranchVersion,
    root: Weak<RelationalBranchRoot>,
    source: Weak<dyn RelationalDurableColdSource>,
    birth: Arc<RelationalBranchRootSchemaAuthority>,
    schema: Arc<RelationalBranchRootSchemaAuthority>,
    invariant_generation: u64,
    initial_inputs_digest: [u8; 32],
}
impl FreshRelationalSourceAdmission {
    pub(super) fn capture(
        runtime: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Stop> {
        let configuration = runtime.configuration_binding();
        let installed = configuration.operation();
        let routes = runtime.history.canonical_checkpoint_gate();
        routes
            .with_checkpoint_selection(|selection| {
                if selection.selected_receipt().is_some()
                    || selection.covered_transitions().0 != 0
                    || runtime.history.catalog_len() != 0
                    || runtime.history.branch_count() != 1
                    || runtime
                        .durability
                        .medium()
                        .is_none_or(|medium| medium.has_unresolved_publication())
                {
                    return Err(refused("fresh Native producer already has durable work"));
                }
                let inputs = runtime
                    .history
                    .capture_original_branch_schema_inputs(execution)
                    .map_err(|cause| match cause {
                        OriginalBranchSchemaInputsDenial::Request(cause) => {
                            Stop::Request(cause.into())
                        }
                        OriginalBranchSchemaInputsDenial::SizeOverflow => Stop::Request(
                            crate::execution::RelationalExecutionDenialCause::MemoryOverflow,
                        ),
                        OriginalBranchSchemaInputsDenial::OriginalPrefixMismatch => {
                            refused("fresh producer input prefix differs")
                        }
                    })?;
                let birth = inputs
                    .birth
                    .as_ref()
                    .ok_or_else(|| refused("fresh producer has no original birth"))?;
                let initial_inputs_digest = inputs
                    .accepted_initial_inputs
                    .as_ref()
                    .and_then(|inputs| inputs.artifact.as_ref())
                    .map(|artifact| artifact.payload_digest())
                    .ok_or_else(|| {
                        refused("fresh producer has no durably accepted original initialization")
                    })?;
                let initial = inputs
                    .installation
                    .as_ref()
                    .map_or(birth, |installation| &installation.after);
                if !inputs.transitions().is_empty()
                    || inputs.installation.as_ref().is_some_and(|installation| {
                        installation.follows_transition.0 != 0
                            || !Arc::ptr_eq(&installation.before.schema, &birth.schema)
                    })
                {
                    return Err(refused("fresh producer has no original initial prefix"));
                }
                let cell = runtime
                    .history
                    .checkpoint_component_source(&installed.config.history.main_branch)
                    .ok_or_else(|| refused("fresh initial branch is absent"))?;
                let (generation, truth_version, root) = cell
                    .fresh_empty_root()
                    .ok_or_else(|| refused("fresh initial branch is not live Empty"))?;
                let schema = root.retained_schema_authority();
                if generation != initial.generation
                    || truth_version != initial.truth_version
                    || !Arc::ptr_eq(&schema, &initial.schema)
                    || root.id() != 0
                    || root.descriptor().is_some()
                {
                    return Err(refused("fresh initial root differs from original producer"));
                }
                let source = runtime
                    .durability
                    .cold_source()
                    .ok_or_else(|| refused("fresh source detached"))?;
                Ok(Self {
                    initial_inputs_digest,
                    runtime_instance_id: runtime.runtime_instance_id(),
                    generation,
                    truth_version,
                    root: Arc::downgrade(&root),
                    source: Arc::downgrade(&source),
                    birth: Arc::clone(&birth.schema),
                    schema,
                    invariant_generation: installed
                        .schema_contract_runtime
                        .custom_invariant_generation,
                })
            })
            .map_err(|_| refused("fresh producer publication is in flight"))?
    }
    /// Validate the SAME current original root, without reconstructing an equal root.
    pub fn validate_fresh_source_runtime(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
    ) -> Result<(), Stop> {
        crate::execution::check_request(execution).map_err(Stop::Request)?;
        if self.runtime_instance_id != runtime.runtime_instance_id() {
            return Err(refused("fresh admission belongs to another Native runtime"));
        }
        let observed = Self::capture(runtime, execution)?;
        let original = self
            .root
            .upgrade()
            .ok_or_else(|| refused("fresh initial root is no longer live"))?;
        let current = observed
            .root
            .upgrade()
            .ok_or_else(|| refused("fresh current root is unavailable"))?;
        if !Arc::ptr_eq(&original, &current)
            || self.generation != observed.generation
            || self.truth_version != observed.truth_version
            || !Arc::ptr_eq(&self.birth, &observed.birth)
            || !Arc::ptr_eq(&self.schema, &observed.schema)
            || self.source.as_ptr().cast::<()>() != observed.source.as_ptr().cast::<()>()
            || self.invariant_generation != observed.invariant_generation
            || self.initial_inputs_digest != observed.initial_inputs_digest
        {
            return Err(refused("fresh initial source has advanced or changed"));
        }
        crate::execution::check_request(execution).map_err(Stop::Request)
    }
    pub(crate) fn matches_installation(
        &self,
        generation: u64,
        digest: &[u8; 32],
        entities: usize,
        relations: usize,
    ) -> bool {
        self.invariant_generation == generation
            && &self.schema.authority_digest() == digest
            && self.schema.registry().entity_kinds.len() == entities
            && self.schema.registry().relation_kinds.len() == relations
    }
}
