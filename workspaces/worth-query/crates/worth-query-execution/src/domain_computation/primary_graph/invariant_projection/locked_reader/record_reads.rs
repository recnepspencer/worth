//! Entity/field reads of the selected native snapshot.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryEntityResolutionRefusalKind as ResolutionRefusal;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as NativeReadStop;

#[cfg(test)]
mod tests;

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
        let execution = self.execution;
        if !self.work_budget.can_afford(3) {
            return Err(WorthQueryEntityResolutionDenial::new(
                WorthQueryEntityResolutionDenialKind::Stopped(NativeReadStop::WorkExhausted),
                field.field(),
            ));
        }
        let value = Field::Binding::encode(&value).map_err(|_| {
            WorthQueryEntityResolutionDenial::new(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::ValueEncodingRejected,
                ),
                field.field(),
            )
        })?;
        let truth = self.entity_resolution.at_snapshot(
            self.runtime,
            self.snapshot,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )?;
        let (resolved, examined) = truth.resolve_with_work(
            execution,
            field.entity(),
            field.aspect(),
            field.field(),
            value,
        );
        let admitted = self.work_budget.consume(1 + examined);
        if admitted {
            self.work.record_lookup(examined);
        }
        let resolved = resolved.inspect_err(|denial| {
            if let WorthQueryEntityResolutionDenialKind::Stopped(cause) = denial.kind() {
                self.native_read_stop = Some(cause);
            }
        })?;
        if !admitted {
            return Err(WorthQueryEntityResolutionDenial::new(
                WorthQueryEntityResolutionDenialKind::Stopped(NativeReadStop::WorkExhausted),
                field.field(),
            ));
        }
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
        let _execution = self.execution;
        match self.resolve_entity(field, value) {
            Ok(identity) => Ok(Some(identity)),
            Err(denial)
                if denial.kind()
                    == WorthQueryEntityResolutionDenialKind::Refused(
                        ResolutionRefusal::UnknownEntity,
                    ) =>
            {
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
        let execution = self.execution;
        if self.native_read_stop.is_some() || self.value_decode.stopped() {
            return None;
        }
        if !self.identity_is_local(identity, field.entity()) {
            return None;
        }
        if !self.work_budget.can_afford(1) {
            return None;
        }
        assert!(self.work_budget.consume(1), "projection work was admitted");
        self.realized_scope.record(identity.entity_id);
        let locator = self
            .layout
            .field_locator(field.entity(), field.aspect(), field.field())?
            .clone();
        let reads = &mut self.work;
        let decoder = self.value_decode;
        let observed = crate::domain_computation::primary_graph::application_attempt::observe_field_value_borrowed_admitted(
            self.runtime,
            execution,
            self.snapshot,
            identity.entity_id,
            identity.kind,
            &locator,
            |raw| {
                reads.record_fields(1);
                raw.and_then(|value| decoder.decode::<Field::Binding>(field.entity(), field.aspect(), field.field(), value))
            },
        )
        ;
        match observed {
            Ok(value) => value.flatten(),
            Err(cause) => {
                self.native_read_stop.get_or_insert(cause);
                None
            }
        }
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
