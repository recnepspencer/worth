//! Validation and sealing of an aggregate projection plan.

use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationRelationRef, ApplicationSchema,
    ApplicationSignedAggregateValueBinding, DeclaredApplicationFieldValue, WritePosture,
};
use worth_relational::facade::identity::{EntityId, KindId, VersionId};

use super::super::{
    WorthQueryApplicationInvariantProjectionReader, WorthQueryInvariantEntityIdentity,
};
use super::{denial, WorthQueryInvariantAggregateDenial, WorthQueryInvariantAggregateDenialKind};
use crate::domain_computation::primary_graph::aggregate_projection::WorthQueryIncomingSumKey;

/// A layout-checked, runtime-local aggregate request.
///
/// Fields are private to this transition owner. Downstream phases may inspect
/// the sealed meaning through accessors but cannot assemble a substitute plan.
#[derive(Debug)]
pub(super) struct ValidatedAggregatePlan {
    key: WorthQueryIncomingSumKey,
    target: EntityId,
    version: VersionId,
    relation_member: String,
    field_member: String,
}

impl ValidatedAggregatePlan {
    pub(super) fn validate<
        Schema,
        Relation,
        From,
        To,
        Aspect,
        Field,
        Value,
        Write,
        Equality,
        Unit,
    >(
        reader: &WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        field: ApplicationFieldRef<Schema, From, Aspect, Field, Value, Write, Equality, Unit>,
        target: &WorthQueryInvariantEntityIdentity<Schema, To>,
    ) -> Result<Self, WorthQueryInvariantAggregateDenial>
    where
        Schema: ApplicationSchema,
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationSignedAggregateValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        let relation_layout = installed_relation(reader, relation.name())?;
        let source_kind = installed_source_kind(reader, field.entity(), field.field())?;
        let field_locator =
            installed_field_locator(reader, field.entity(), field.aspect(), field.field())?;
        if relation_layout.from != source_kind
            || relation_layout.to != target.kind
            || !reader.identity_is_local(target, relation.to())
        {
            return Err(denial(
                WorthQueryInvariantAggregateDenialKind::ForeignIdentity,
                relation.name(),
            ));
        }
        Ok(Self {
            key: WorthQueryIncomingSumKey {
                relation_kind: relation_layout.kind,
                source_kind,
                target_kind: relation_layout.to,
                field: field_locator,
            },
            target: target.entity_id,
            version: reader.snapshot.version_id(),
            relation_member: relation.name().to_owned(),
            field_member: field.field().to_owned(),
        })
    }

    pub(super) const fn key(&self) -> &WorthQueryIncomingSumKey {
        &self.key
    }

    pub(super) const fn target(&self) -> EntityId {
        self.target
    }

    pub(super) const fn version(&self) -> VersionId {
        self.version
    }

    pub(super) const fn source_kind(&self) -> KindId {
        self.key.source_kind
    }

    pub(super) fn relation_member(&self) -> &str {
        &self.relation_member
    }

    pub(super) fn field_member(&self) -> &str {
        &self.field_member
    }
}

fn installed_relation<Schema>(
    reader: &WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
    relation: &str,
) -> Result<
    crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryRelationLayout,
    WorthQueryInvariantAggregateDenial,
> {
    reader.layout.relation(relation).cloned().ok_or_else(|| {
        denial(
            WorthQueryInvariantAggregateDenialKind::RelationNotInstalled,
            relation,
        )
    })
}

fn installed_source_kind<Schema>(
    reader: &WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
    entity: &str,
    field: &str,
) -> Result<KindId, WorthQueryInvariantAggregateDenial> {
    reader.layout.entity_kind(entity).ok_or_else(|| {
        denial(
            WorthQueryInvariantAggregateDenialKind::FieldNotInstalled,
            field,
        )
    })
}

fn installed_field_locator<Schema>(
    reader: &WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
    entity: &str,
    aspect: &str,
    field: &str,
) -> Result<worth_foundational::facade::AspectFieldLocator, WorthQueryInvariantAggregateDenial> {
    reader
        .layout
        .field_locator(entity, aspect, field)
        .cloned()
        .ok_or_else(|| {
            denial(
                WorthQueryInvariantAggregateDenialKind::FieldNotInstalled,
                field,
            )
        })
}
