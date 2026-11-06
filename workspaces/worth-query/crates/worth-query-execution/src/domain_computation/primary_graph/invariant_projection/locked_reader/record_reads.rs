//! Entity/field reads of the selected native snapshot.
use super::*;

impl<Schema> WorthQueryApplicationInvariantProjectionReader<'_, Schema>
where
    Schema: ApplicationSchema,
{
    pub const fn version(&self) -> worth_relational::facade::identity::VersionId {
        self.native.version(self.snapshot)
    }

    /// Reads performed so far by this snapshot projection; diagnostic only.
    pub const fn work(&self) -> WorthQueryInvariantProjectionWork {
        self.work
    }

    #[inline(always)]
    pub fn resolve_entity<Aspect, Entity, Field, Value>(
        &mut self,

        field: Field,
        value: Value,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Entity>, WorthQueryEntityResolutionDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
    {
        Ok(self
            .native_entity()
            .resolve(
                field.identity(),
                &super::super::super::value_decode::ScalarFieldEncoder::<Field::Binding>::new(
                    &value,
                ),
            )?
            .into_typed())
    }

    #[inline(always)]
    pub fn resolve_optional_entity<Aspect, Entity, Field, Value>(
        &mut self,

        field: Field,
        value: Value,
    ) -> Result<
        Option<WorthQueryInvariantEntityIdentity<Schema, Entity>>,
        WorthQueryEntityResolutionDenial,
    >
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
    {
        let identity = self.native_entity().resolve_optional(
            field.identity(),
            &super::super::super::value_decode::ScalarFieldEncoder::<Field::Binding>::new(&value),
        )?;
        Ok(super::super::native_read::typed_optional(identity))
    }

    #[inline(always)]
    pub fn field<Entity, Aspect, Field, Value>(
        &mut self,

        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: Field,
    ) -> Option<Value>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
        Field::Binding: ApplicationReadableScalarValueBinding,
    {
        let mut decoded =
            super::super::super::value_decode::ScalarFieldDecoder::<Field::Binding>::new();
        self.native_read()
            .field(identity.native_identity(), field.identity(), &mut decoded);
        decoded.value
    }

    pub(in crate::domain_computation::primary_graph::invariant_projection) fn identity_is_local<
        Entity,
    >(
        &self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        entity: &str,
    ) -> bool {
        identity.native_identity().local_to(
            self.authority_identity,
            self.native
                .candidate_identity()
                .map(|id| id.qualification()),
            entity,
        )
    }
}

impl<'r, S> WorthQueryApplicationInvariantProjectionReader<'r, S> {
    #[inline(always)]
    fn native_entity(&mut self) -> super::super::native_read::NativeEntity<'_, 'r> {
        let resolution = self.entity_resolution;
        self.native_read().entity_resolution(resolution)
    }
}
