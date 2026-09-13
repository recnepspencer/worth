use std::marker::PhantomData;

use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;

/// Borrowed, application-only view used while lowering one invariant factory.
pub struct WorthQueryApplicationInvariantSchemaResolver<'layout, Schema> {
    pub(super) _schema: PhantomData<fn() -> Schema>,
    pub(super) layout: &'layout super::super::schema_layout::WorthQueryPrimaryGraphLayout,
    pub(super) binding_identity: ApplicationSchemaBindingIdentity,
}

impl<Schema> WorthQueryApplicationInvariantSchemaResolver<'_, Schema> {
    /// Resolves one declared entity kind for typed invariant access.
    pub fn typed_entity<Entity>(
        &self,
        entity: worth_query_declaration::facade::application_schema::ApplicationEntityRef<
            Schema,
            Entity,
        >,
    ) -> Option<
        super::super::application_invariant::WorthQueryApplicationInvariantEntityBinding<
            Schema,
            Entity,
        >,
    > {
        self.layout.entity_kind(entity.name()).map(|entity_kind| {
            super::super::application_invariant::WorthQueryApplicationInvariantEntityBinding {
                binding_identity: self.binding_identity.clone(),
                entity_kind,
                _marker: PhantomData,
            }
        })
    }

    pub fn typed_relation<Relation, From, To>(
        &self,
        relation: worth_query_declaration::facade::application_schema::ApplicationRelationRef<
            Schema,
            Relation,
            From,
            To,
        >,
    ) -> Option<
        super::super::application_invariant::WorthQueryApplicationInvariantRelationBinding<
            Schema,
            Relation,
            From,
            To,
        >,
    > {
        let layout = self.layout.relation(relation.name())?;
        if self.layout.entity_kind(relation.from()) != Some(layout.from)
            || self.layout.entity_kind(relation.to()) != Some(layout.to)
        {
            return None;
        }
        Some(
            super::super::application_invariant::WorthQueryApplicationInvariantRelationBinding {
                binding_identity: self.binding_identity.clone(),
                relation_kind: layout.kind,
                from_kind: layout.from,
                to_kind: layout.to,
                _marker: PhantomData,
            },
        )
    }

    pub fn typed_field<
        Entity,
        Aspect,
        Field,
        Value,
        Write,
        Equality,
        Unit: worth_query_declaration::facade::application_schema::ApplicationFieldUnit,
    >(
        &self,
        field: worth_query_declaration::facade::application_schema::ApplicationFieldRef<
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            Equality,
            Unit,
        >,
    ) -> Option<super::super::application_invariant::WorthQueryApplicationInvariantFieldBinding<Schema, Entity, Value>>
    where
        Field: worth_query_declaration::facade::application_schema::DeclaredApplicationFieldValue<Value = Value>,
        Field::Binding: worth_query_declaration::facade::application_schema::ApplicationReadableScalarValueBinding,
    {
        self.layout
            .field(field.entity(), field.aspect(), field.field())
            .map(|layout| super::super::application_invariant::WorthQueryApplicationInvariantFieldBinding {
                binding_identity: self.binding_identity.clone(),
                entity_kind: layout.entity_kind,
                locator: layout.locator.clone(),
                presence: Field::PRESENCE,
                decode: |value| <Field::Binding as worth_query_declaration::facade::application_schema::ApplicationReadableScalarValueBinding>::decode(value)
                    .map_err(|error| format!("{error:?}")),
                _marker: PhantomData,
            })
    }
}
