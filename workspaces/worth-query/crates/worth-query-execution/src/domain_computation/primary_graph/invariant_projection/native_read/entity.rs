//! Native entity resolution shares the original snapshot, work and affinity context.
use super::*;
use crate::domain_computation::primary_graph::{
    value_decode::FieldEncoder, WorthQueryEntityResolutionRefusalKind as ResolutionRefusal,
};
use crate::domain_computation::primary_graph::{
    WorthQueryEntityResolutionDenial, WorthQueryEntityResolutionDenialKind,
};
use worth_query_declaration::facade::application_schema::{
    fragment::ColumnIdentity, ApplicationFieldBindingLocus,
};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as NativeReadStop;

pub(in crate::domain_computation::primary_graph::invariant_projection) struct NativeEntity<'a, 'r> {
    native: NativeRead<'a, 'r>,
    entity_resolution: &'r super::super::super::WorthQueryInstalledEntityResolutionContext,
}
impl<'a, 'r> NativeRead<'a, 'r> {
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn entity_resolution(
        self,
        resolution: &'r super::super::super::WorthQueryInstalledEntityResolutionContext,
    ) -> NativeEntity<'a, 'r> {
        NativeEntity {
            native: self,
            entity_resolution: resolution,
        }
    }
}
impl NativeEntity<'_, '_> {
    pub fn resolve(
        &mut self,
        field: ColumnIdentity,
        encoder: &dyn FieldEncoder,
    ) -> Result<WorthQueryInvariantEntityIdentity<(), ()>, WorthQueryEntityResolutionDenial> {
        if !self.native.budget.can_afford(3) {
            return Err(WorthQueryEntityResolutionDenial::new(
                WorthQueryEntityResolutionDenialKind::Stopped(NativeReadStop::WorkExhausted),
                field.field(),
            ));
        }
        let column = field
            .installed(self.native.bindings.fragments())
            .map_err(|_| {
                WorthQueryEntityResolutionDenial::new(
                    WorthQueryEntityResolutionDenialKind::Refused(
                        ResolutionRefusal::FieldNotInstalled,
                    ),
                    field.field(),
                )
            })?;
        encoder.validate().map_err(|_| {
            WorthQueryEntityResolutionDenial::new(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::ValueEncodingRejected,
                ),
                column.path(),
            )
        })?;
        let encoded = encoder.encode_value().map_err(|_| {
            WorthQueryEntityResolutionDenial::new(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::ValueEncodingRejected,
                ),
                field.field(),
            )
        })?;
        self.resolve_encoded(&field.locus(), encoded)
    }
    pub fn resolve_optional(
        &mut self,
        field: ColumnIdentity,
        encoder: &dyn FieldEncoder,
    ) -> Result<Option<WorthQueryInvariantEntityIdentity<(), ()>>, WorthQueryEntityResolutionDenial>
    {
        match self.resolve(field, encoder) {
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
    pub fn resolve_encoded(
        &mut self,
        field: &ApplicationFieldBindingLocus,
        value: worth_foundational::facade::AspectValue,
    ) -> Result<WorthQueryInvariantEntityIdentity<(), ()>, WorthQueryEntityResolutionDenial> {
        let execution = self.native.execution;
        if !self.native.budget.can_afford(3) {
            return Err(WorthQueryEntityResolutionDenial::new(
                WorthQueryEntityResolutionDenialKind::Stopped(NativeReadStop::WorkExhausted),
                field.field(),
            ));
        }
        let (resolved, examined) = self.native.native.resolve_entity(
            self.entity_resolution,
            execution,
            self.native.snapshot,
            field.entity(),
            field.aspect(),
            field.field(),
            value,
        );
        let admitted = self.native.budget.consume(1 + examined);
        if admitted {
            self.native.work.record_lookup(examined);
        }
        let resolved = resolved.inspect_err(|denial| {
            if let WorthQueryEntityResolutionDenialKind::Stopped(cause) = denial.kind() {
                *self.native.stopped = Some(cause);
            }
        })?;
        if !admitted {
            return Err(WorthQueryEntityResolutionDenial::new(
                WorthQueryEntityResolutionDenialKind::Stopped(NativeReadStop::WorkExhausted),
                field.field(),
            ));
        }
        self.native.scope.record(resolved.entity);
        Ok(WorthQueryInvariantEntityIdentity {
            entity_id: resolved.entity,
            kind: resolved.kind,
            entity: Arc::from(field.entity()),
            authority_identity: self.native.authority,
            phase: resolved.phase,
            _marker: PhantomData,
        })
    }
}
pub(in crate::domain_computation::primary_graph::invariant_projection) fn typed_optional<S, E>(
    identity: Option<WorthQueryInvariantEntityIdentity<(), ()>>,
) -> Option<WorthQueryInvariantEntityIdentity<S, E>> {
    identity.map(WorthQueryInvariantEntityIdentity::into_typed)
}
