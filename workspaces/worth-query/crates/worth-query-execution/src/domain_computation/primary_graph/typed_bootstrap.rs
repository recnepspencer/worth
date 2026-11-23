use std::collections::BTreeMap;
use std::marker::PhantomData;
use worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus;

use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_declaration::facade::application_naming::ApplicationNameDiscriminator;
mod seed_name;
pub(super) use seed_name::SeedPersistentName;
use worth_query_installation::facade::{
    ApplicationEntityRef, ApplicationRelationRef, ApplicationScalarValueBinding, ApplicationSchema,
    ApplicationValueEncodeDenial,
};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::lineage::PersistentNameSubjectKind as Subject;

use super::{
    WorthQueryApplicationEntityKey, WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind,
};

type EncodedSeedField = Result<
    (String, String, AspectValue),
    (ApplicationFieldBindingLocus, ApplicationValueEncodeDenial),
>;

/// One entity to seed into the primary graph at bootstrap: its kind, key, and
/// field values.
///
/// Build with `new` and `field`, then pass it to `bind_entity`.
pub struct WorthQueryApplicationEntitySeed<Schema, Entity> {
    entity: &'static str,
    key: WorthQueryApplicationEntityKey<Schema, Entity>,
    fields: Vec<EncodedSeedField>,
    name: Option<SeedPersistentName>,
}

impl<Schema, Entity> WorthQueryApplicationEntitySeed<Schema, Entity> {
    pub(super) fn migration_cost(&self) -> (usize, usize) {
        let bytes = std::mem::size_of::<Self>()
            .saturating_add(self.key.owned_allocation_capacity_bytes())
            .saturating_add(
                self.name
                    .as_ref()
                    .map_or(0, SeedPersistentName::retained_bytes),
            )
            .saturating_add(
                self.fields
                    .capacity()
                    .saturating_mul(std::mem::size_of::<EncodedSeedField>()),
            );
        let bytes = self.fields.iter().fold(bytes, |bytes, field| {
            bytes.saturating_add(field.as_ref().map_or_else(
                |(locus, cause)| {
                    locus
                        .entity()
                        .len()
                        .saturating_add(locus.aspect().len())
                        .saturating_add(locus.field().len())
                        .saturating_add(cause.retained_heap_bytes())
                },
                |(aspect, field, value)| {
                    aspect.capacity().saturating_add(
                        field
                            .capacity()
                            .saturating_add(value.owned_allocation_capacity_bytes()),
                    )
                },
            ))
        });
        (1_usize.saturating_add(self.fields.len()), bytes)
    }

    pub fn new(
        entity: ApplicationEntityRef<Schema, Entity>,
        key: WorthQueryApplicationEntityKey<Schema, Entity>,
    ) -> Self {
        Self {
            entity: entity.name(),
            key,
            fields: Vec::new(),
            name: None,
        }
    }

    /// Names the seeded record under its kind's declared persistent-name
    /// scheme, born under the declared `role` with the seed key as its
    /// authored occurrence. A seed of a schemed kind must declare its name.
    pub fn persistent_name(mut self, role: &str) -> Self {
        self.name = Some(SeedPersistentName::plain(role));
        self
    }

    /// As [`Self::persistent_name`], distinguished by a typed discriminator.
    pub fn persistent_name_with_discriminator<D: ApplicationNameDiscriminator>(
        mut self,
        role: &str,
        discriminator: &D,
    ) -> Self {
        self.name = Some(SeedPersistentName::discriminated(role, discriminator));
        self
    }

    pub fn record_field<
        A: worth_query_declaration::facade::application_schema::fragment::Aspect,
        T,
    >(
        mut self,
        field: worth_query_declaration::facade::application_schema::fragment::RecordField<
            Entity,
            A,
            T,
        >,
        value: T,
    ) -> Result<
        Self,
        worth_query_declaration::facade::application_schema::fragment::FieldEncodeRefusal,
    > {
        let encoded = field.encode(&value)?;
        self.fields.push(Ok((
            field.locus().aspect().to_owned(),
            field.locus().field().to_owned(),
            encoded,
        )));
        Ok(self)
    }

    pub fn field<Aspect, Field, Value>(mut self, field: Field, value: Value) -> Self
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
        Field::Binding: ApplicationScalarValueBinding<Value = Value>,
    {
        self.fields.push(
            Field::Binding::encode(&value)
                .map(|value| (field.aspect().to_owned(), field.field().to_owned(), value))
                .map_err(|cause| {
                    (
                        ApplicationFieldBindingLocus::new(
                            self.entity,
                            field.aspect(),
                            field.field(),
                        ),
                        cause,
                    )
                }),
        );
        self
    }
}

/// One relation to seed at bootstrap, keyed and linking two seeded entities by
/// their keys.
///
/// Pass it to `bind_relation`.
pub struct WorthQueryApplicationRelationSeed<Schema, Relation, From, To> {
    relation: &'static str,
    key: String,
    from: WorthQueryApplicationEntityKey<Schema, From>,
    to: WorthQueryApplicationEntityKey<Schema, To>,
    name: Option<SeedPersistentName>,
    _marker: PhantomData<fn() -> Relation>,
}

impl<Schema, Relation, From, To> WorthQueryApplicationRelationSeed<Schema, Relation, From, To> {
    pub(super) fn migration_cost(&self) -> (usize, usize) {
        (
            1,
            std::mem::size_of::<Self>()
                .saturating_add(self.key.capacity())
                .saturating_add(self.from.owned_allocation_capacity_bytes())
                .saturating_add(self.to.owned_allocation_capacity_bytes())
                .saturating_add(
                    self.name
                        .as_ref()
                        .map_or(0, SeedPersistentName::retained_bytes),
                ),
        )
    }

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
            name: None,
            _marker: PhantomData,
        }
    }

    /// Names the seeded relation under its declared persistent-name scheme,
    /// born under `role` with the seed key as its authored occurrence.
    pub fn persistent_name(mut self, role: &str) -> Self {
        self.name = Some(SeedPersistentName::plain(role));
        self
    }

    /// As [`Self::persistent_name`], distinguished by a typed discriminator.
    pub fn persistent_name_with_discriminator<D: ApplicationNameDiscriminator>(
        mut self,
        role: &str,
        discriminator: &D,
    ) -> Self {
        self.name = Some(SeedPersistentName::discriminated(role, discriminator));
        self
    }
}

pub(super) struct WorthQueryTypedEntityBootstrapRow {
    pub(super) kind: KindId,
    pub(super) key: String,
    pub(super) fields: BTreeMap<AspectFieldLocator, AspectValue>,
    pub(super) name: Option<SeedPersistentName>,
}

pub(super) struct WorthQueryTypedRelationBootstrapRow {
    pub(super) kind: KindId,
    pub(super) key: String,
    pub(super) from_kind: KindId,
    pub(super) from_key: String,
    pub(super) to_kind: KindId,
    pub(super) to_key: String,
    pub(super) name: Option<SeedPersistentName>,
}

#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct TypedBootstrapRows {
    pub(super) entity_keys: std::collections::BTreeSet<(KindId, String)>,
    pub(super) pending_entity_keys: std::collections::BTreeSet<(KindId, String)>,
    pub(super) relation_keys: std::collections::BTreeSet<(KindId, String)>,
    pub(super) unique_seed_values:
        std::collections::BTreeSet<super::bootstrap::unique_seed_values::WorthQueryUniqueSeedValue>,
    pub(super) entity_rows: Vec<WorthQueryTypedEntityBootstrapRow>,
    pub(super) relation_rows: Vec<WorthQueryTypedRelationBootstrapRow>,
}

impl TypedBootstrapRows {
    pub(super) fn bind_entity<Schema: ApplicationSchema, Entity>(
        &mut self,
        layout: &super::schema_layout::WorthQueryPrimaryGraphLayout,
        recovered: bool,
        seed: WorthQueryApplicationEntitySeed<Schema, Entity>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        let kind = layout
            .entity_kind(seed.entity)
            .ok_or_else(|| invalid_seed(seed.entity))?;
        let key = seed.key.into_string();
        if layout.is_signed_entity_kind(kind) || layout.signing_lifecycle.is_record(kind) {
            return Err(invalid_seed(
                "signed entities require native authenticated publication",
            ));
        }
        if self.entity_keys.contains(&(kind, key.clone())) {
            return Err(invalid_seed("duplicate application entity key"));
        }
        require_declared_name(layout, Subject::Entity, kind, seed.name.is_some())?;
        let mut fields = BTreeMap::new();
        for encoded in seed.fields {
            let (aspect, field, value) = encoded.map_err(|(field, cause)| {
                invalid_seed("typed bootstrap field value failed validation")
                    .with_field_encoding_denial(field, cause)
            })?;
            let locator = layout
                .field_locator(seed.entity, &aspect, &field)
                .cloned()
                .ok_or_else(|| invalid_seed(&field))?;
            let locus = ApplicationFieldBindingLocus::new(seed.entity, &aspect, &field);
            fields.insert(locator, value);
            if let Some(presence) = layout.record_presence_locator(&locus) {
                fields.insert(presence.clone(), AspectValue::Bool(true));
            }
        }
        super::bootstrap::unique_seed_values::admit_values(
            layout,
            recovered,
            &mut self.unique_seed_values,
            fields.iter().map(|(locator, value)| (kind, locator, value)),
        )?;
        self.entity_keys.insert((kind, key.clone()));
        self.pending_entity_keys.insert((kind, key.clone()));
        self.entity_rows.push(WorthQueryTypedEntityBootstrapRow {
            kind,
            key,
            fields,
            name: seed.name,
        });
        Ok(())
    }

    pub(super) fn bind_relation<Schema: ApplicationSchema, Relation, From, To>(
        &mut self,
        layout: &super::schema_layout::WorthQueryPrimaryGraphLayout,
        _recovered: bool,
        seed: WorthQueryApplicationRelationSeed<Schema, Relation, From, To>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        let graph_layout = layout;
        let layout = layout
            .relation(seed.relation)
            .cloned()
            .ok_or_else(|| invalid_seed(seed.relation))?;
        let from_key = seed.from.into_string();
        let to_key = seed.to.into_string();
        if !self
            .pending_entity_keys
            .contains(&(layout.from, from_key.clone()))
            || !self
                .pending_entity_keys
                .contains(&(layout.to, to_key.clone()))
        {
            return Err(invalid_seed(
                "typed relation endpoints must be seeded in the current batch",
            ));
        }
        require_declared_name(graph_layout, Subject::Relation, layout.kind, seed.name.is_some())?;
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
                name: seed.name,
            });
        Ok(())
    }
}

/// A schemed kind's seed must declare its name, and an unschemed kind's seed
/// cannot: Relational would refuse the first and nothing can mint the second.
fn require_declared_name(
    layout: &super::schema_layout::WorthQueryPrimaryGraphLayout,
    subject: Subject,
    kind: KindId,
    named: bool,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let schemed = layout
        .persistent_names()
        .is_some_and(|names| names.scheme_for_kind(subject, kind).is_some());
    match (schemed, named) {
        (true, false) => Err(invalid_seed(
            "a seed of a persistently named kind must declare its persistent name",
        )),
        (false, true) => Err(invalid_seed(
            "a seed names a kind that declares no persistent-name scheme",
        )),
        _ => Ok(()),
    }
}

fn invalid_seed(subject: impl Into<String>) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::InvalidSchemaMember,
        subject,
    )
}
