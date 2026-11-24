//! Typed seed rows lower to Native creates and, for persistently named kinds,
//! to the exact birth plan Relational requires of every such creation.
//!
//! A seed's application key is its authored occurrence: stable across
//! rebuilds, unique per kind, and never an allocation or a bare operation.
use worth_query_execution_naming::facade::{
    ExecutionNameBirthPlanBuilder, ExecutionPendingEntityBirth, ExecutionPendingRelationBirth,
};
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::lineage::{CommitNamingPlan, PersistentNameSubjectKind};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, CreatedEntityRef, CreatedRelationRef, EntityReference,
    EntitySpec, MutationIntent, RelationSpec, WorkerIntentBatch,
};

use super::super::schema_layout::WorthQueryPrimaryGraphLayout;
use super::super::typed_bootstrap::{
    WorthQueryTypedEntityBootstrapRow, WorthQueryTypedRelationBootstrapRow,
};
use super::super::{
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
};

/// Appends every typed seed row to `batch` and returns the naming plan the
/// commit must attach, `None` when no row names its record.
///
/// Complexity: O(S log S) for S seed rows.
pub(in crate::domain_computation::primary_graph) fn append_typed_rows(
    mut batch: WorkerIntentBatch,
    entity_rows: Vec<WorthQueryTypedEntityBootstrapRow>,
    relation_rows: Vec<WorthQueryTypedRelationBootstrapRow>,
    layout: &WorthQueryPrimaryGraphLayout,
) -> Result<(WorkerIntentBatch, Option<CommitNamingPlan>), WorthQueryPrimaryGraphInstallationDenial>
{
    let plan = seed_naming_plan(layout, &entity_rows, &relation_rows)?;
    for row in entity_rows {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: row.kind,
            client_key: ClientKey::raw(row.key),
            fields: AspectFieldPatch::from(row.fields),
        })));
    }
    for row in relation_rows {
        batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: row.kind,
                client_key: ClientKey::raw(row.key),
                source: created_entity(row.from_kind, row.from_key),
                target: created_entity(row.to_kind, row.to_key),
                fields: AspectFieldPatch::default(),
            },
        )));
    }
    Ok((batch, plan))
}

fn created_entity(
    kind_id: worth_relational::facade::identity::KindId,
    key: String,
) -> EntityReference {
    EntityReference::Created(CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id,
        client_key: ClientKey::raw(key),
    })
}

fn occurrence(key: &str) -> String {
    format!("worth.bootstrap-seed/{key}")
}

fn seed_naming_plan(
    layout: &WorthQueryPrimaryGraphLayout,
    entity_rows: &[WorthQueryTypedEntityBootstrapRow],
    relation_rows: &[WorthQueryTypedRelationBootstrapRow],
) -> Result<Option<CommitNamingPlan>, WorthQueryPrimaryGraphInstallationDenial> {
    let Some(names) = layout.persistent_names() else {
        return Ok(None);
    };
    let mut builder = ExecutionNameBirthPlanBuilder::new(names.catalog(), std::iter::empty());
    let mut births = 0_usize;
    for row in entity_rows {
        let Some(name) = &row.name else { continue };
        let scheme = names
            .scheme_for_kind(PersistentNameSubjectKind::Entity, row.kind)
            .ok_or_else(|| denial("seed names a kind without a persistent-name scheme"))?;
        let mut birth = ExecutionPendingEntityBirth::new(
            CreatedEntityRef {
                partition_id: PartitionId::main(),
                kind_id: row.kind,
                client_key: ClientKey::raw(row.key.clone()),
            },
            scheme,
            occurrence(&row.key),
            name.role.clone(),
        );
        birth.canonical_discriminator = name.discriminator.clone();
        birth.embedded_name_digests = name.references.clone();
        builder
            .add_entity_birth(birth)
            .map_err(|cause| denial(format!("seed persistent name refused: {cause:?}")))?;
        births += 1;
    }
    for row in relation_rows {
        let Some(name) = &row.name else { continue };
        let scheme = names
            .scheme_for_kind(PersistentNameSubjectKind::Relation, row.kind)
            .ok_or_else(|| denial("seed names a relation without a persistent-name scheme"))?;
        let mut birth = ExecutionPendingRelationBirth::new(
            CreatedRelationRef {
                partition_id: PartitionId::main(),
                kind_id: row.kind,
                client_key: ClientKey::raw(row.key.clone()),
                source: created_entity(row.from_kind, row.from_key.clone()),
                target: created_entity(row.to_kind, row.to_key.clone()),
            },
            scheme,
            occurrence(&row.key),
            name.role.clone(),
        );
        birth.canonical_discriminator = name.discriminator.clone();
        birth.embedded_name_digests = name.references.clone();
        builder
            .add_relation_birth(birth)
            .map_err(|cause| denial(format!("seed persistent name refused: {cause:?}")))?;
        births += 1;
    }
    Ok((births > 0).then(|| builder.build().into_relational_plan()))
}

fn denial(subject: impl Into<String>) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::InvalidSchemaMember,
        subject,
    )
}
