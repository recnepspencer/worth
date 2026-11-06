//! Entity/field reads of the selected native snapshot.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryEntityResolutionRefusalKind as ResolutionRefusal;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as NativeReadStop;

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
        if !self.work_budget.can_afford(3) {
            return Err(WorthQueryEntityResolutionDenial::new(
                WorthQueryEntityResolutionDenialKind::Stopped(NativeReadStop::WorkExhausted),
                field.field(),
            ));
        }
        let column = field
            .installed(self.value_bindings.fragments())
            .map_err(|_| {
                WorthQueryEntityResolutionDenial::new(
                    WorthQueryEntityResolutionDenialKind::Refused(
                        ResolutionRefusal::FieldNotInstalled,
                    ),
                    field.field(),
                )
            })?;
        Field::Binding::validate(&value).map_err(|_| {
            WorthQueryEntityResolutionDenial::new(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::ValueEncodingRejected,
                ),
                column.path(),
            )
        })?;
        let value = Field::Binding::encode(&value).map_err(|_| {
            WorthQueryEntityResolutionDenial::new(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::ValueEncodingRejected,
                ),
                field.field(),
            )
        })?;
        self.resolve_encoded_entity(
            &worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus::new(
                field.entity(),
                field.aspect(),
                field.field(),
            ),
            value,
        )
    }

    pub(in crate::domain_computation::primary_graph::invariant_projection) fn resolve_encoded_entity<
        Entity,
    >(
        &mut self,
        field: &worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus,
        value: worth_foundational::facade::AspectValue,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Entity>, WorthQueryEntityResolutionDenial>
    {
        let execution = self.execution;
        if !self.work_budget.can_afford(3) {
            return Err(WorthQueryEntityResolutionDenial::new(
                WorthQueryEntityResolutionDenialKind::Stopped(NativeReadStop::WorkExhausted),
                field.field(),
            ));
        }
        let (resolved, examined) = self.native.resolve_entity(
            self.entity_resolution,
            execution,
            self.snapshot,
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
        self.realized_scope.record(resolved.entity);
        Ok(WorthQueryInvariantEntityIdentity {
            entity_id: resolved.entity,
            kind: resolved.kind,
            entity: Arc::from(field.entity()),
            authority_identity: self.authority_identity,
            phase: resolved.phase,
            _marker: PhantomData,
        })
    }

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
        if field.installed(self.value_bindings.fragments()).is_err() {
            self.value_decode.refuse_content(crate::domain_computation::primary_graph::WorthQueryValueDecodeDenial::BindingContractMismatch);
            return None;
        }
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
        let observed = self.native.observe_field(
            execution,
            self.snapshot,
            identity.entity_id,
            identity.kind,
            &locator,
            |observed| {
                let raw = observed.value();
                reads.record_fields(1);
                raw.and_then(|value| {
                    decoder.decode::<Field::Binding>(
                        field.entity(),
                        field.aspect(),
                        field.field(),
                        value,
                    )
                })
            },
        );
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
        identity.authority_identity == self.authority_identity
            && identity.entity.as_ref() == entity
            && identity.phase.is_none_or(|phase| {
                Some(phase)
                    == self
                        .native
                        .candidate_identity()
                        .map(|id| id.qualification())
            })
    }
}
