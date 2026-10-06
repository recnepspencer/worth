//! Bounded selection of retained output families and bindings.

use std::any::TypeId;
use std::collections::BTreeMap;
use std::sync::Arc;

use super::*;

impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph) fn resolve_binding<Binding: 'static>(
        &self,
        scope: &crate::domain_computation::authorization::WorthQueryOperationScopeBinding,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
        source_partition_identity: [u8; 32],
        maximum_source_lookups: usize,
    ) -> Result<WorthQueryPriorOutputBindingResolution, ()> {
        let source = SemanticSource {
            runtime_authority: scope.runtime_authority(),
            schema: scope.binding_identity().clone(),
            scope: scope.scope(),
            output_binding: TypeId::of::<Binding>(),
        };
        if !self.by_source.contains_key(&source) {
            return (maximum_source_lookups > 0)
                .then_some(WorthQueryPriorOutputBindingResolution {
                    correspondence: None,
                    source_lookups: 1,
                })
                .ok_or(());
        }
        let mut coordinate = ProductCoordinate {
            occurrence,
            generation,
        };
        let mut source_lookups = 0_usize;
        loop {
            let (recorded, work) = self.latest_output_in_partition_budgeted(
                &source,
                coordinate,
                source_partition_identity,
                maximum_source_lookups.saturating_sub(source_lookups),
            )?;
            source_lookups = source_lookups.checked_add(work).ok_or(())?;
            if source_lookups > maximum_source_lookups {
                return Err(());
            }
            if let Some(recorded) = recorded {
                return Ok(WorthQueryPriorOutputBindingResolution {
                    correspondence: Some(Arc::clone(&recorded.correspondence)),
                    source_lookups,
                });
            }
            let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                return Ok(WorthQueryPriorOutputBindingResolution {
                    correspondence: None,
                    source_lookups,
                });
            };
            coordinate = parent;
        }
    }
}

pub(super) fn latest_output_matching(
    history: &BTreeMap<u64, RecordedGeneration>,
    maximum_generation: u64,
    mut matches: impl FnMut(&RecordedOutput) -> bool,
) -> Option<&RecordedOutput> {
    history
        .range(..=maximum_generation)
        .rev()
        .find_map(|(_, recorded)| {
            recorded
                .iter()
                .filter_map(|cell| cell.get())
                .find(|recorded| matches(recorded))
        })
}
