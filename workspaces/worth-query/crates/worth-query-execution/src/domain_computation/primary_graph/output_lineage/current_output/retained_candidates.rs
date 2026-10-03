//! Retained candidate selection preserves its spent legacy Work on every exit.

use super::*;

impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph) fn retained_output_candidates(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
        output_bindings: &[TypeId],
        source_partition_identity: [u8; 32],
        maximum_work: usize,
    ) -> Result<(Vec<WorthQueryRetainedOutputCandidate>, usize), ()> {
        let mut remaining = maximum_work;
        let candidates = self.retained_output_candidates_with_remaining(
            runtime_authority,
            schema,
            scope,
            occurrence,
            generation,
            output_bindings,
            source_partition_identity,
            &mut remaining,
        )?;
        Ok((candidates, maximum_work - remaining))
    }

    pub(in crate::domain_computation::primary_graph) fn retained_output_candidates_with_remaining(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
        output_bindings: &[TypeId],
        source_partition_identity: [u8; 32],
        remaining: &mut usize,
    ) -> Result<Vec<WorthQueryRetainedOutputCandidate>, ()> {
        let mut candidates = Vec::new();
        for output_binding in output_bindings {
            let source = SemanticSource {
                runtime_authority,
                schema: schema.clone(),
                scope,
                output_binding: *output_binding,
            };
            if !self.by_source.contains_key(&source) {
                *remaining = remaining.checked_sub(1).ok_or(())?;
                continue;
            }
            let mut coordinate = ProductCoordinate {
                occurrence,
                generation,
            };
            loop {
                let lookup = self.latest_output_in_partition_budgeted(
                    &source,
                    coordinate,
                    source_partition_identity,
                    *remaining,
                );
                let (recorded, work) = match lookup {
                    Ok(found) => found,
                    Err(()) => {
                        // This owner returns only budget exhaustion. Its bounded
                        // legacy scan spent the available visits before stopping.
                        *remaining = 0;
                        return Err(());
                    }
                };
                *remaining = remaining.checked_sub(work).ok_or(())?;
                if let Some(recorded) = recorded {
                    candidates.push(WorthQueryRetainedOutputCandidate {
                        binding: *output_binding,
                        correspondence: Arc::clone(&recorded.correspondence),
                        source_identity: recorded.source_identity,
                        observed_source_facts: recorded.observed_source_facts(),
                        resources: recorded.resources(),
                        idempotency_key_identity: recorded.idempotency_key_identity,
                    });
                    break;
                }
                let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                    break;
                };
                coordinate = parent;
            }
        }
        Ok(candidates)
    }
}
