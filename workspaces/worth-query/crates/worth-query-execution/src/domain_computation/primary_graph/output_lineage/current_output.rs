use std::any::TypeId;

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

use super::{
    ProductCoordinate, SemanticSource, WorthQueryApplicationOutputLineage,
    WorthQueryOutputSourcePosture,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;

impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph) fn source_facts_for_receipt(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        receipt: &WorthQueryApplicationCommitReceipt,
        maximum_work: usize,
    ) -> Result<
        Option<(
            std::sync::Arc<[super::super::application_attempt::WorthQueryApplicationObservedFact]>,
            usize,
        )>,
        (),
    > {
        let Some(output_binding) = receipt.output_correspondence().binding_type() else {
            return Ok(None);
        };
        let Some(source_identity) = receipt.idempotency_binding().source_identity() else {
            return Ok(None);
        };
        let source = SemanticSource {
            runtime_authority,
            schema: schema.clone(),
            scope: receipt.principal_scope().scope(),
            output_binding,
        };
        let Some(versions) = self.by_source.get(&source) else {
            return Ok(None);
        };
        let mut coordinate = ProductCoordinate {
            occurrence: observation.lifecycle_incarnation(),
            generation: observation.reference_generation().get(),
        };
        let mut work = 0_usize;
        loop {
            work = work.checked_add(1).ok_or(())?;
            if work > maximum_work {
                return Err(());
            }
            if let Some(recorded) = versions
                .get(&coordinate.occurrence)
                .and_then(|history| history.range(..=coordinate.generation).next_back())
                .map(|(_, recorded)| recorded)
            {
                return Ok((recorded.source_identity == Some(source_identity)
                    && std::ptr::eq(
                        recorded.correspondence.as_ref(),
                        receipt.output_correspondence(),
                    ))
                .then(|| (std::sync::Arc::clone(&recorded.observed_source_facts), work)));
            }
            let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                return Ok(None);
            };
            coordinate = parent;
        }
    }

    pub(in crate::domain_computation::primary_graph) fn current_output_matches_receipt(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> bool {
        self.source_facts_for_receipt(runtime_authority, schema, observation, receipt, usize::MAX)
            .is_ok_and(|facts| facts.is_some())
    }

    pub(in crate::domain_computation::primary_graph) fn source_posture_for_any_output_binding(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
        output_bindings: &[TypeId],
        current_source_identity: [u8; 32],
    ) -> WorthQueryOutputSourcePosture {
        let mut retained_output = false;
        for output_binding in output_bindings {
            let source = SemanticSource {
                runtime_authority,
                schema: schema.clone(),
                scope,
                output_binding: *output_binding,
            };
            let Some(versions) = self.by_source.get(&source) else {
                continue;
            };
            let mut coordinate = ProductCoordinate {
                occurrence,
                generation,
            };
            loop {
                if versions.get(&coordinate.occurrence).is_some_and(|history| {
                    history
                        .range(..=coordinate.generation)
                        .next_back()
                        .is_some()
                }) {
                    let recorded = versions
                        .get(&coordinate.occurrence)
                        .and_then(|history| history.range(..=coordinate.generation).next_back())
                        .map(|(_, recorded)| recorded)
                        .expect("retained output was just found");
                    if recorded.source_identity == Some(current_source_identity) {
                        return WorthQueryOutputSourcePosture::Exact(*output_binding);
                    }
                    retained_output = true;
                }
                let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                    break;
                };
                coordinate = parent;
            }
        }
        if retained_output {
            WorthQueryOutputSourcePosture::Drifted
        } else {
            WorthQueryOutputSourcePosture::Absent
        }
    }
}
