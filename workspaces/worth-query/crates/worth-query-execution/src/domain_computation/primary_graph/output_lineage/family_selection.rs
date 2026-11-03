//! Candidate heads for every source partition in a producer-qualified output family.
use super::ProductCoordinate;
use super::SemanticSource;
use super::WorthQueryApplicationOutputLineage;
use super::WorthQueryCurrentOutputCandidate;
use super::WorthQueryCurrentOutputFamilyResolution;
use crate::equality_type_id::EqualityTypeId as TypeId;
use std::collections::BTreeSet;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
mod checkpoint_priors;
mod publication_heads;
pub(in crate::domain_computation::primary_graph) use checkpoint_priors::CheckpointPriorSelectionDenial;
pub(in crate::domain_computation::primary_graph) use checkpoint_priors::CheckpointPriorStructure;
pub(in crate::domain_computation::primary_graph) use checkpoint_priors::NativePriorCheckpointOutput;
pub(super) use publication_heads::PublicationHeads;
use worth_relational::facade::mvcc::CompanionPreflightStop;

impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph) fn checkpoint_family_role(
        &self,
        binding: Option<TypeId>,
    ) -> Result<Option<(&str, &str)>, CheckpointPriorSelectionDenial> {
        let Some(binding) = binding else {
            return Ok(None);
        };
        let mut found = None;
        for (family, bindings) in &self.output_families {
            for (candidate, role) in bindings {
                if *candidate != binding {
                    continue;
                }
                if found.is_some() {
                    return Err(CheckpointPriorSelectionDenial::Structural(
                        CheckpointPriorStructure::DuplicateInstalledBinding,
                    ));
                }
                found = Some((family.as_str(), role.as_str()));
            }
        }
        Ok(found)
    }

    pub(in crate::domain_computation::primary_graph) fn resolve_current_family(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        family: &str,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
    ) -> Result<WorthQueryCurrentOutputFamilyResolution, CompanionPreflightStop> {
        execution
            .consult()
            .map_err(|stop| CompanionPreflightStop::Execution(stop.into()))?;
        let Some(bindings) = self.output_families.get(family) else {
            return Ok(WorthQueryCurrentOutputFamilyResolution {
                family_installed: false,
                ambiguous_publication: false,
                candidates: Vec::new(),
            });
        };
        let mut publications = PublicationHeads::default();
        for (output_binding, output_role) in bindings {
            execution
                .consult()
                .map_err(|stop| CompanionPreflightStop::Execution(stop.into()))?;
            let source = SemanticSource {
                runtime_authority,
                schema: schema.clone(),
                scope,
                output_binding: self
                    .binding_identity(*output_binding)
                    .expect("output family binding is installed"),
            };
            if !self.by_source.contains_key(&source) {
                continue;
            }
            let mut coordinate = ProductCoordinate {
                occurrence,
                generation,
            };
            let mut seen = BTreeSet::new();
            let mut ancestry_depth = 0usize;
            loop {
                for (partition, publication) in
                    self.family_partition_heads(execution, &source, coordinate)?
                {
                    execution
                        .consult()
                        .map_err(|stop| CompanionPreflightStop::Execution(stop.into()))?;
                    if seen.insert(partition) {
                        publications.insert(
                            partition,
                            output_role,
                            *output_binding,
                            ancestry_depth,
                            publication,
                        );
                    }
                }
                let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                    break;
                };
                coordinate = parent;
                ancestry_depth = ancestry_depth
                    .checked_add(1)
                    .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
            }
        }
        let (heads, ambiguous_publication) = publications.finish();
        let candidates = heads
            .into_iter()
            .filter_map(|(role, recorded)| {
                self.implementation_is_current(recorded)
                    .then(|| WorthQueryCurrentOutputCandidate::from_recorded(recorded, role))
                    .flatten()
            })
            .collect();
        Ok(WorthQueryCurrentOutputFamilyResolution {
            family_installed: true,
            ambiguous_publication,
            candidates,
        })
    }
}
