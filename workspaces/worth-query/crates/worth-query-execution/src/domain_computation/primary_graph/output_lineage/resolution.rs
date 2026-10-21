//! Bounded selection of retained output families and bindings.

use crate::equality_type_id::EqualityTypeId as TypeId;
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
    ) -> Result<WorthQueryPriorOutputBindingResolution, ()> {
        let Some(output_binding) = self.binding_identity(TypeId::of::<Binding>()) else {
            return Ok(WorthQueryPriorOutputBindingResolution {
                correspondence: None,
            });
        };
        let source = SemanticSource {
            runtime_authority: scope.runtime_authority(),
            schema: scope.binding_identity().clone(),
            scope: scope.scope(),
            output_binding,
        };
        let mut coordinate = ProductCoordinate {
            occurrence,
            generation,
        };
        loop {
            if let Some(recorded) =
                self.latest_output_in_partition(&source, coordinate, source_partition_identity)
            {
                return Ok(WorthQueryPriorOutputBindingResolution {
                    correspondence: Some(Arc::clone(&recorded.correspondence)),
                });
            }
            let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                return Ok(WorthQueryPriorOutputBindingResolution {
                    correspondence: None,
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
