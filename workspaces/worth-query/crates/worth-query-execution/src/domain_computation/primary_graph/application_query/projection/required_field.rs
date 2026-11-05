use super::*;
use crate::domain_computation::vocabulary::primary_graph::projection_denial::WorthQueryApplicationProjectionDenial;
use crate::domain_computation::vocabulary::primary_graph::projection_denial::WorthQueryApplicationProjectionDenialKind;

impl<'row, Schema, Query> WorthQueryApplicationProjectionRow<'row, Schema, Query> {
    pub fn disclosed_field<Slot, Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &self,
        selector: ApplicationQueryResultFieldRef<
            Query,
            Slot,
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            Equality,
            Unit,
        >,
    ) -> Result<WorthQueryApplicationDisclosed<Value>, WorthQueryApplicationProjectionDenial>
    where
        Field: RequiredApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        let projected = match self.disclosed_required_field(selector)? {
            WorthQueryApplicationDisclosed::Disclosed(projected) => projected,
            WorthQueryApplicationDisclosed::Omitted(omission) => {
                return Ok(WorthQueryApplicationDisclosed::Omitted(omission))
            }
        };
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
        Ok(WorthQueryApplicationDisclosed::Disclosed(value))
    }

    fn disclosed_required_field<Slot, Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &self,
        selector: ApplicationQueryResultFieldRef<
            Query,
            Slot,
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            Equality,
            Unit,
        >,
    ) -> Result<
        WorthQueryApplicationDisclosed<&super::WorthQueryApplicationProjectedField>,
        WorthQueryApplicationProjectionDenial,
    >
    where
        Field: RequiredApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        let projected = self.node.field(selector.slot_type());
        let requested;
        let slot = if let Some(actual) = projected.filter(|actual| actual.matches(&selector)) {
            actual.slot_key_identity().as_ref()
        } else {
            requested = selector.slot_key();
            &requested
        };
        if let Some(omission) = self.omission(slot) {
            return Ok(WorthQueryApplicationDisclosed::Omitted(omission));
        }
        if !self.governance.is_disclosed(slot) {
            return Err(projection_denial(
                WorthQueryApplicationProjectionDenialKind::FieldContractMismatch,
                selector.slot_type(),
            ));
        }
        let projected = self.node.field(selector.slot_type()).ok_or_else(|| {
            projection_denial(
                WorthQueryApplicationProjectionDenialKind::FieldNotProjected,
                selector.slot_type(),
            )
        })?;
        if !projected.matches(&selector) {
            return Err(projection_denial(
                WorthQueryApplicationProjectionDenialKind::FieldContractMismatch,
                projected.result_path(),
            ));
        }
        Ok(WorthQueryApplicationDisclosed::Disclosed(projected))
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn required_field<
        Slot,
        Entity,
        Aspect,
        Field,
        Value,
        Write,
        Equality,
        Unit,
    >(
        &self,
        selector: ApplicationQueryResultFieldRef<
            Query,
            Slot,
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            Equality,
            Unit,
        >,
    ) -> Result<&super::WorthQueryApplicationProjectedField, WorthQueryApplicationProjectionDenial>
    where
        Field: RequiredApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        self.disclosed_required_field(selector)?
            .into_required(WorthQueryApplicationProjectionDenialKind::FieldOmitted)
    }
}
