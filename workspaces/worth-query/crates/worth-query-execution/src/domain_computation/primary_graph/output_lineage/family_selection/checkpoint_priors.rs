//! Prior-only checkpoint locators for the selected live product occurrence.

use std::{
    any::TypeId,
    collections::{BTreeMap, BTreeSet},
};

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_relational::facade::identity::EntityId;
use worth_runtime_world::facade::ProductBranchIncarnation;

use super::super::{
    invalidation::InvalidationEditAdmission, ProductCoordinate, WorthQueryApplicationOutputLineage,
};

fn copy_text(value: &str) -> Result<String, ()> {
    let mut copied = String::new();
    copied.try_reserve_exact(value.len()).map_err(|_| ())?;
    copied.push_str(value);
    Ok(copied)
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct OutputFamilyRole {
    pub(in crate::domain_computation::primary_graph) family: String,
    pub(in crate::domain_computation::primary_graph) role: String,
}

/// A selected native family head, with a reusable locator only when that
/// exact performed lineage retained one before its effect.
pub(in crate::domain_computation::primary_graph) struct NativePriorCheckpointOutput {
    pub(in crate::domain_computation::primary_graph) family_role: OutputFamilyRole,
    pub(in crate::domain_computation::primary_graph) scope:
        crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    pub(in crate::domain_computation::primary_graph) binding: TypeId,
    pub(in crate::domain_computation::primary_graph) idempotency_key: [u8; 32],
    pub(in crate::domain_computation::primary_graph) partition: Option<[u8; 32]>,
    pub(in crate::domain_computation::primary_graph) entity: Option<EntityId>,
    pub(in crate::domain_computation::primary_graph) identity: Option<
        crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity,
    >,
}

impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph) fn checkpoint_prior_outputs(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        occurrence: ProductBranchIncarnation,
        generation: u64,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Vec<NativePriorCheckpointOutput>, ()> {
        let mut installed = BTreeMap::<TypeId, (&str, &str)>::new();
        for (family, bindings) in &self.output_families {
            admission.charge_external_work(1).map_err(|_| ())?;
            for (binding, role) in bindings {
                admission.charge_external_work(1).map_err(|_| ())?;
                if installed.contains_key(binding) {
                    return Err(());
                }
                admission
                    .admit_read_scratch(
                        u64::try_from(std::mem::size_of::<(TypeId, (&str, &str))>())
                            .map_err(|_| ())?,
                    )
                    .map_err(|_| ())?;
                installed.insert(*binding, (family, role));
            }
        }

        let mut publications = BTreeMap::new();
        for source in self.by_source.keys() {
            admission.charge_external_work(1).map_err(|_| ())?;
            if source.runtime_authority != runtime_authority || &source.schema != schema {
                continue;
            }
            let Some((family, role)) = installed.get(&source.output_binding).copied() else {
                continue;
            };
            let group = (family, role, source.scope);
            if !publications.contains_key(&group) {
                let bytes = std::mem::size_of::<(&str, &str, super::PublicationHeads<'_>)>();
                admission
                    .admit_read_scratch(u64::try_from(bytes).map_err(|_| ())?)
                    .map_err(|_| ())?;
            }
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
                let remaining = admission.remaining_work();
                let (partition_heads, work) = self
                    .family_partition_heads_budgeted(source, coordinate, remaining)
                    .map_err(|_| ())?;
                admission
                    .charge_external_work(u64::try_from(work).map_err(|_| ())?)
                    .map_err(|_| ())?;
                for (partition, publication) in partition_heads {
                    admission.charge_external_work(1).map_err(|_| ())?;
                    if seen_partitions.contains(&partition) {
                        continue;
                    }
                    let set_bytes = u64::try_from(std::mem::size_of::<Option<[u8; 32]>>() + 32)
                        .map_err(|_| ())?;
                    admission.admit_read_scratch(set_bytes).map_err(|_| ())?;
                    if !seen_partitions.insert(partition) {
                        return Err(());
                    }
                    let member_bytes = std::mem::size_of::<(
                        Option<[u8; 32]>,
                        &str,
                        EntityId,
                        std::cmp::Reverse<usize>,
                        u64,
                        std::any::TypeId,
                        ProductCoordinate,
                        usize,
                        &super::super::RecordedOutput,
                        bool,
                    )>();
                    admission
                        .admit_read_scratch(u64::try_from(member_bytes).map_err(|_| ())?)
                        .map_err(|_| ())?;
                    admission
                        .charge_external_work(
                            u64::try_from(heads.admission_work()).map_err(|_| ())?,
                        )
                        .map_err(|_| ())?;
                    heads.insert(
                        partition,
                        role,
                        source.output_binding,
                        ancestry_depth,
                        publication,
                    );
                }
                let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                    break;
                };
                coordinate = parent;
                ancestry_depth = ancestry_depth.checked_add(1).ok_or(())?;
                admission.charge_external_work(1).map_err(|_| ())?;
            }
        }

        let mut selected = Vec::new();
        for ((family, role_name, scope), publications) in publications {
            let (heads, ambiguous) = publications.finish();
            if ambiguous {
                return Err(());
            }
            for (role, recorded) in heads {
                if role != role_name {
                    return Err(());
                }
                admission.charge_external_work(1).map_err(|_| ())?;
                let binding = recorded.correspondence.binding_type().ok_or(())?;
                let entity = recorded.correspondence.publication_entity_for_role(role);
                let partition = recorded.source_partition_identity;
                let performed = recorded
                    .performed_origin
                    .as_ref()
                    .and_then(|origin| origin.get())
                    .unwrap_or(recorded);
                let identity = match performed.native_prior_checkpoint.as_ref() {
                    Some(locator) => {
                        let partition = partition.ok_or(())?;
                        let role_views = recorded.correspondence.native_witness_roles();
                        let role_count = role_views.len();
                        let role_bytes = role_views
                            .clone()
                            .try_fold(0usize, |total, (role, _, entity_name, _)| {
                                total
                                    .checked_add(role.len())?
                                    .checked_add(entity_name.len())?
                                    .checked_add(64)
                            })
                            .ok_or(())?;
                        let retained_bytes = std::mem::size_of::<
                            crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity,
                        >()
                        .checked_add(locator.producer.len())
                        .and_then(|bytes| bytes.checked_add(role_bytes))
                        .and_then(|bytes| {
                            bytes.checked_add(
                                role_count.checked_mul(std::mem::size_of::<
                                    crate::domain_computation::primary_graph::application_attempt::WorthQueryCheckpointOutputRole,
                                >() + 2 * std::mem::size_of::<String>())?,
                            )
                        })
                        .ok_or(())?;
                        admission
                            .admit_read_scratch(u64::try_from(retained_bytes).map_err(|_| ())?)
                            .map_err(|_| ())?;
                        admission
                            .charge_external_work(u64::try_from(retained_bytes).map_err(|_| ())?)
                            .map_err(|_| ())?;
                        let mut producer = String::new();
                        producer
                            .try_reserve_exact(locator.producer.len())
                            .map_err(|_| ())?;
                        producer.push_str(&locator.producer);
                        let mut roles = Vec::new();
                        roles.try_reserve_exact(role_count).map_err(|_| ())?;
                        for (role, posture, entity_name, entity) in role_views {
                            let mut copied_role = String::new();
                            copied_role.try_reserve_exact(role.len()).map_err(|_| ())?;
                            copied_role.push_str(role);
                            let mut copied_entity_name = String::new();
                            copied_entity_name
                                .try_reserve_exact(entity_name.len())
                                .map_err(|_| ())?;
                            copied_entity_name.push_str(entity_name);
                            roles.push(
                                crate::domain_computation::primary_graph::application_attempt::WorthQueryCheckpointOutputRole {
                                    role: copied_role,
                                    posture,
                                    entity_name: copied_entity_name,
                                    entity,
                                },
                            );
                        }
                        Some(
                            crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity {
                                producer,
                                posture: crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointPosture::Performed,
                                source: locator.source,
                                scope,
                                source_partition: partition,
                                producer_dependency: recorded.producer_dependency_identity,
                                idempotency_key: recorded.idempotency_key_identity,
                                resources: recorded.resources(),
                                roles,
                                producer_facts: None,
                                producer_fact_wire_version: 0,
                            },
                        )
                    }
                    None => None,
                };
                let family_role_bytes = family
                    .len()
                    .checked_add(role_name.len())
                    .and_then(|bytes| bytes.checked_add(std::mem::size_of::<OutputFamilyRole>()))
                    .ok_or(())?;
                admission
                    .admit_read_scratch(u64::try_from(family_role_bytes).map_err(|_| ())?)
                    .map_err(|_| ())?;
                admission
                    .charge_external_work(u64::try_from(family_role_bytes).map_err(|_| ())?)
                    .map_err(|_| ())?;
                let family = copy_text(family)?;
                let role_name = copy_text(role_name)?;
                admission
                    .admit_read_scratch(
                        u64::try_from(std::mem::size_of::<NativePriorCheckpointOutput>())
                            .map_err(|_| ())?,
                    )
                    .map_err(|_| ())?;
                admission.charge_external_work(1).map_err(|_| ())?;
                selected.try_reserve(1).map_err(|_| ())?;
                selected.push(NativePriorCheckpointOutput {
                    family_role: OutputFamilyRole {
                        family,
                        role: role_name,
                    },
                    scope,
                    binding,
                    idempotency_key: recorded.idempotency_key_identity,
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
