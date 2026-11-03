//! Prior-only checkpoint locators for the selected live product occurrence.

use crate::equality_type_id::EqualityTypeId as TypeId;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_relational::facade::identity::EntityId;
use worth_runtime_world::facade::ProductBranchIncarnation;

use super::super::ProductCoordinate;
use super::super::WorthQueryApplicationOutputLineage;

mod denial;
mod materialization;
pub(in crate::domain_computation::primary_graph) use denial::CheckpointPriorSelectionDenial;
pub(in crate::domain_computation::primary_graph) use denial::CheckpointPriorStructure;
use CheckpointPriorSelectionDenial as Denial;
use CheckpointPriorStructure as Structure;

fn copy_text(value: &str) -> Result<String, Denial> {
    let mut copied = String::new();
    copied
        .try_reserve_exact(value.len())
        .map_err(|error| Denial::allocation("family or role text", error))?;
    copied.push_str(value);
    Ok(copied)
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct OutputFamilyRole {
    pub(in crate::domain_computation::primary_graph) family:
        super::super::super::output_family_identity::OutputFamilyIdentity,
    pub(in crate::domain_computation::primary_graph) role: String,
}

/// A selected native family head, with a reusable locator only when that
/// exact performed lineage retained one before its effect.
pub(in crate::domain_computation::primary_graph) struct NativePriorCheckpointOutput<'a> {
    pub(in crate::domain_computation::primary_graph) family_role: OutputFamilyRole,
    pub(in crate::domain_computation::primary_graph) scope:
        crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    pub(in crate::domain_computation::primary_graph) binding: TypeId,
    pub(in crate::domain_computation::primary_graph) idempotency_key: [u8; 32],
    producing_implementation:
        Option<worth_relational::facade::mvcc::RequiredOutputProducingImplementation>,
    pub(in crate::domain_computation::primary_graph) partition: Option<[u8; 32]>,
    pub(in crate::domain_computation::primary_graph) entity: Option<EntityId>,
    pub(in crate::domain_computation::primary_graph) identity:
        Option<NativePriorCheckpointIdentity<'a>>,
}

pub(in crate::domain_computation::primary_graph) struct NativePriorCheckpointIdentity<'a> {
    pub(in crate::domain_computation::primary_graph) producer: &'a str,
    pub(in crate::domain_computation::primary_graph) source: [u8; 32],
    recorded: &'a super::super::RecordedOutput,
    recovered: Option<&'a crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity>,
}

impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph) fn checkpoint_prior_outputs(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        occurrence: ProductBranchIncarnation,
        generation: u64,
    ) -> Result<Vec<NativePriorCheckpointOutput<'_>>, Denial> {
        let mut installed = HashMap::<TypeId, (&str, &str)>::new();
        for (family, bindings) in &self.output_families {
            for (binding, role) in bindings {
                if installed.contains_key(binding) {
                    return Err(Denial::Structural(Structure::DuplicateInstalledBinding));
                }

                installed
                    .try_reserve(1)
                    .map_err(|error| Denial::allocation("installed binding scratch", error))?;
                installed.insert(*binding, (family.as_str(), role.as_str()));
            }
        }

        let mut publications = BTreeMap::new();
        for source in self.by_source.keys() {
            if source.runtime_authority != runtime_authority || &source.schema != schema {
                continue;
            }
            let Some((family, role)) = installed
                .get(&self.binding_type(&source.output_binding))
                .copied()
            else {
                continue;
            };
            let group = (family, role, source.scope);

            let heads = publications
                .entry(group)
                .or_insert_with(super::PublicationHeads::default);

            let mut coordinate = ProductCoordinate {
                occurrence,
                generation,
            };
            let mut ancestry_depth = 0usize;
            let mut seen_partitions = BTreeSet::new();
            loop {
                let partition_heads = self
                    .family_partition_heads(execution, source, coordinate)
                    .map_err(|stop| Denial::admission("partition head traversal", stop))?;

                for (partition, publication) in partition_heads {
                    if seen_partitions.contains(&partition) {
                        continue;
                    }

                    if !seen_partitions.insert(partition) {
                        return Err(Denial::Structural(Structure::DuplicatePartition));
                    }

                    heads.insert(
                        partition,
                        role,
                        self.binding_type(&source.output_binding),
                        ancestry_depth,
                        publication,
                    );
                }
                let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                    break;
                };
                coordinate = parent;
                ancestry_depth = ancestry_depth
                    .checked_add(1)
                    .ok_or(Denial::arithmetic("occurrence ancestry depth"))?;
            }
        }

        let mut selected = Vec::new();
        for ((family, role_name, scope), publications) in publications {
            let (heads, ambiguous) = publications.finish();
            if ambiguous {
                return Err(Denial::Structural(Structure::AmbiguousPublicationHeads));
            }
            for (role, recorded) in heads {
                if role != role_name {
                    return Err(Denial::Structural(Structure::UnexpectedOutputRole));
                }

                let binding = recorded
                    .correspondence
                    .binding_type()
                    .ok_or(Denial::Structural(Structure::MissingOperationBinding))?;
                let entity = recorded.correspondence.publication_entity_for_role(role);
                let partition = recorded.source_partition_identity;
                let performed = recorded
                    .performed_origin
                    .as_ref()
                    .and_then(|origin| origin.get())
                    .unwrap_or(recorded);
                let identity = match performed.native_prior_checkpoint.as_ref() {
                    Some(locator) => {
                        partition.ok_or(Denial::Structural(Structure::MissingSourcePartition))?;
                        Some(NativePriorCheckpointIdentity {
                            producer: locator.producer(),
                            source: locator.source(),
                            recorded,
                            recovered: locator.recovered(),
                        })
                    }
                    None => None,
                };

                let family = copy_text(family)?;
                let role_name = copy_text(role_name)?;

                selected
                    .try_reserve(1)
                    .map_err(|error| Denial::allocation("selected head rows", error))?;
                selected.push(NativePriorCheckpointOutput {
                    family_role: OutputFamilyRole {
                        family: family.into(),
                        role: role_name,
                    },
                    scope,
                    binding,
                    idempotency_key: recorded.idempotency_key_identity,
                    producing_implementation: recorded.producing_implementation,
                    partition,
                    entity,
                    identity,
                });
            }
        }
        selected.sort_by(|left, right| {
            left.family_role
                .cmp(&right.family_role)
                .then_with(|| left.scope.cmp(&right.scope))
                .then_with(|| left.partition.cmp(&right.partition))
                .then_with(|| left.entity.cmp(&right.entity))
        });
        Ok(selected)
    }
}
