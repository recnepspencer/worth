use std::collections::BTreeMap;
use std::marker::PhantomData;

use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_installation::facade::{
    ApplicationEntityRef, ApplicationFieldRef, ApplicationFieldUnit, ApplicationRelationRef,
    ApplicationScalarValueBinding, ApplicationSchema, DeclaredApplicationFieldValue,
    EqualityPosture, WritePosture,
};
use worth_relational::facade::identity::KindId;

use super::{
    WorthQueryApplicationEntityKey, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
};

pub struct WorthQueryApplicationEntitySeed<Schema, Entity> {
    entity: &'static str,
    key: WorthQueryApplicationEntityKey<Schema, Entity>,
    fields: Vec<
        Result<
            (&'static str, &'static str, AspectValue),
            worth_query_installation::facade::ApplicationValueEncodeDenial,
        >,
    >,
}

impl<Schema, Entity> WorthQueryApplicationEntitySeed<Schema, Entity> {
    pub fn new(
        entity: ApplicationEntityRef<Schema, Entity>,
        key: WorthQueryApplicationEntityKey<Schema, Entity>,
    ) -> Self {
        Self {
            entity: entity.name(),
            key,
            fields: Vec::new(),
        }
    }

    pub fn field<Aspect, Field, Value, Write, Equality, Unit>(
        mut self,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        value: Value,
    ) -> Self
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationScalarValueBinding<Value = Value>,
        Write: WritePosture,
        Equality: EqualityPosture,
        Unit: ApplicationFieldUnit,
    {
        self.fields.push(
            Field::Binding::encode(&value).map(|value| (field.aspect(), field.field(), value)),
        );
        self
    }
}

pub struct WorthQueryApplicationRelationSeed<Schema, Relation, From, To> {
    relation: &'static str,
    key: String,
    from: WorthQueryApplicationEntityKey<Schema, From>,
    to: WorthQueryApplicationEntityKey<Schema, To>,
    _marker: PhantomData<fn() -> Relation>,
}

impl<Schema, Relation, From, To> WorthQueryApplicationRelationSeed<Schema, Relation, From, To> {
    pub fn new(
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        key: impl Into<String>,
        from: WorthQueryApplicationEntityKey<Schema, From>,
        to: WorthQueryApplicationEntityKey<Schema, To>,
    ) -> Self {
        Self {
            relation: relation.name(),
            key: key.into(),
            from,
            to,
            _marker: PhantomData,
        }
    }
}

pub(super) struct WorthQueryTypedEntityBootstrapRow {
    pub(super) kind: KindId,
    pub(super) key: String,
    pub(super) fields: BTreeMap<AspectFieldLocator, AspectValue>,
}

pub(super) struct WorthQueryTypedRelationBootstrapRow {
    pub(super) kind: KindId,
    pub(super) key: String,
    pub(super) from_kind: KindId,
    pub(super) from_key: String,
    pub(super) to_kind: KindId,
    pub(super) to_key: String,
}

impl<Schema> WorthQueryPrimaryGraphBootstrap<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn bind_entity<Entity>(
        &mut self,
        seed: WorthQueryApplicationEntitySeed<Schema, Entity>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        let kind = self
            .graph
            .layout
            .entity_kind(seed.entity)
            .ok_or_else(|| invalid_seed(seed.entity))?;
        let key = seed.key.into_string();
        if !self.entity_keys.insert((kind, key.clone())) {
            return Err(invalid_seed("duplicate application entity key"));
        }
        let fields = seed
            .fields
            .into_iter()
            .map(|encoded| {
                let (aspect, field, value) = encoded
                    .map_err(|_| invalid_seed("typed bootstrap field value failed validation"))?;
                self.graph
                    .layout
                    .field_locator(seed.entity, aspect, field)
                    .cloned()
                    .map(|locator| (locator, value))
                    .ok_or_else(|| invalid_seed(field))
            })
            .collect::<Result<_, _>>()?;
        self.entity_rows
            .push(WorthQueryTypedEntityBootstrapRow { kind, key, fields });
        Ok(())
    }

    pub fn bind_relation<Relation, From, To>(
        &mut self,
        seed: WorthQueryApplicationRelationSeed<Schema, Relation, From, To>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        let layout = self
            .graph
            .layout
            .relation(seed.relation)
            .cloned()
            .ok_or_else(|| invalid_seed(seed.relation))?;
        let from_key = seed.from.into_string();
        let to_key = seed.to.into_string();
        if !self.entity_keys.contains(&(layout.from, from_key.clone()))
            || !self.entity_keys.contains(&(layout.to, to_key.clone()))
        {
            return Err(invalid_seed("typed relation endpoint is not bound"));
        }
        let relation_identity = (layout.kind, seed.key.clone());
        if !self.relation_keys.insert(relation_identity) {
            return Err(invalid_seed("duplicate application relation key"));
        }
        self.relation_rows
            .push(WorthQueryTypedRelationBootstrapRow {
                kind: layout.kind,
                key: seed.key,
                from_kind: layout.from,
                from_key,
                to_kind: layout.to,
                to_key,
            });
        Ok(())
    }
}

fn invalid_seed(subject: impl Into<String>) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::InvalidSchemaMember,
        subject,
    )
}
