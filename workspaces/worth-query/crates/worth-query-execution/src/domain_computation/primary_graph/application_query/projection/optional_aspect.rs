//! Projection of a Required child under a genuinely absent Optional parent.
use super::disclosed::*;
use super::*;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryMarkerIdentity, ApplicationQueryOptionalAspectFieldRef,
};
use worth_query_declaration::facade::portable_identity::WorthQueryPortableType;
use worth_query_installation::facade::ApplicationReadableScalarValueBinding;
impl<'row, Schema, Query> WorthQueryApplicationProjectionRow<'row, Schema, Query> {
    pub fn disclosed_optional_aspect_field<Slot, Entity, Aspect, Field, Value>(
        &self,
        selector: ApplicationQueryOptionalAspectFieldRef<Query, Slot, Schema, Field>,
    ) -> Result<WorthQueryApplicationDisclosed<Option<Value>>, WorthQueryApplicationProjectionDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
        Field::Binding: ApplicationReadableScalarValueBinding,

        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        let slot = selector.slot_key();
        if let Some(omission) = self.omission(&slot) {
            return Ok(WorthQueryApplicationDisclosed::Omitted(omission));
        }
        if !self.governance.is_disclosed(&slot) {
            return Err(projection_denial(
                WorthQueryApplicationProjectionDenialKind::FieldContractMismatch,
                selector.slot_type(),
            ));
        }
        let Some(projected) = self.node.field(selector.slot_type()) else {
            return Ok(WorthQueryApplicationDisclosed::Disclosed(None));
        };
        if projected.slot_key_identity().as_ref() != &slot {
            return Err(projection_denial(
                WorthQueryApplicationProjectionDenialKind::FieldContractMismatch,
                projected.result_path(),
            ));
        }
        let value = self
            .decode
            .decode::<Field::Binding>(
                selector.entity(),
                selector.aspect(),
                selector.field(),
                projected.value(),
            )
            .ok_or_else(|| {
                projection_denial(
                    WorthQueryApplicationProjectionDenialKind::FieldTypeMismatch,
                    projected.result_path(),
                )
            })?;
        Ok(WorthQueryApplicationDisclosed::Disclosed(Some(value)))
    }

    pub fn optional_aspect_field<Slot, Entity, Aspect, Field, Value>(
        &self,
        selector: ApplicationQueryOptionalAspectFieldRef<Query, Slot, Schema, Field>,
    ) -> Result<Option<Value>, WorthQueryApplicationProjectionDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
        Field::Binding: ApplicationReadableScalarValueBinding,

        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        self.disclosed_optional_aspect_field(selector)?
            .into_required(WorthQueryApplicationProjectionDenialKind::FieldOmitted)
    }
}
