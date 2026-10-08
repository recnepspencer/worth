//! Entity/field reads of the selected native snapshot.
use super::*;

impl<Schema> WorthQueryApplicationInvariantProjectionReader<'_, Schema>
where
    Schema: ApplicationSchema,
{
    pub const fn version(&self) -> worth_relational::facade::identity::VersionId {
        self.snapshot.version_id()
    }

    pub fn resolve_entity<Aspect, Entity, Field, Value, Write, Unit>(
        &mut self,
        field: ApplicationFieldRef<
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            EqualityPredicate,
            Unit,
        >,
        value: Value,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Entity>, WorthQueryEntityResolutionDenial>
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        if !self.work_budget.can_afford(3) {
            return Err(WorthQueryEntityResolutionDenial::new(
                WorthQueryEntityResolutionDenialKind::ProjectionWorkBudgetExceeded,
                field.field(),
            ));
        }
        let value = Field::Binding::encode(&value).map_err(|_| {
            WorthQueryEntityResolutionDenial::new(
                WorthQueryEntityResolutionDenialKind::ValueEncodingRejected,
                field.field(),
            )
        })?;
        let truth = self.entity_resolution.at_snapshot(
            self.runtime,
            self.snapshot,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )?;
        let (resolved, examined) =
            truth.resolve_with_work(field.entity(), field.aspect(), field.field(), value);
        self.work_budget.consume(1 + examined);
        self.work.record_lookup(examined);
        let resolved = resolved?;
        self.realized_scope.record(resolved.entity_id());
        Ok(WorthQueryInvariantEntityIdentity {
            entity_id: resolved.entity_id(),
            kind: resolved.entity_kind(),
            entity: Arc::from(field.entity()),
            authority_identity: self.authority_identity,
            _marker: PhantomData,
        })
    }

    pub fn resolve_optional_entity<Aspect, Entity, Field, Value, Write, Unit>(
        &mut self,
        field: ApplicationFieldRef<
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            EqualityPredicate,
            Unit,
        >,
        value: Value,
    ) -> Result<
        Option<WorthQueryInvariantEntityIdentity<Schema, Entity>>,
        WorthQueryEntityResolutionDenial,
    >
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        match self.resolve_entity(field, value) {
            Ok(identity) => Ok(Some(identity)),
            Err(denial) if denial.kind() == WorthQueryEntityResolutionDenialKind::UnknownEntity => {
                Ok(None)
            }
            Err(denial) => Err(denial),
        }
    }

    pub fn field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Option<Value>
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        if !self.identity_is_local(identity, field.entity()) {
            return None;
        }
        if !self.work_budget.can_afford(1) {
            return None;
        }
        self.work_budget.consume(1);
        self.realized_scope.record(identity.entity_id);
        let locator = self
            .layout
            .field_locator(field.entity(), field.aspect(), field.field())?
            .clone();
        self.work.record_field();
        crate::domain_computation::primary_graph::application_attempt::observe_field_value_borrowed(
            self.runtime,
            self.snapshot,
            identity.entity_id,
            identity.kind,
            &locator,
            |raw| raw.and_then(|value| Field::Binding::decode(value).ok()),
        )
        .flatten()
    }

    pub(in crate::domain_computation::primary_graph::invariant_projection) fn identity_is_local<
        Entity,
    >(
        &self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        entity: &str,
    ) -> bool {
        identity.authority_identity == self.authority_identity && identity.entity.as_ref() == entity
    }
}
