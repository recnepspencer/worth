use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryMarkerIdentity, ApplicationQueryOptionalResultFieldRef,
    ApplicationQueryResultFieldRef, ApplicationQueryResultRelationCardinality,
    ApplicationQueryResultRelationRef, ApplicationQueryResultTraversal, ExactlyOneResult,
    ManyResults, OptionalOneResult,
};
use worth_query_declaration::facade::portable_identity::WorthQueryPortableType;
use worth_query_installation::facade::{
    ApplicationFieldUnit, ApplicationReadableScalarValueBinding, OptionalApplicationFieldValue,
    RequiredApplicationFieldValue, WritePosture,
};

use super::{
    projection_denial, relation_cardinality_denial, WorthQueryApplicationProjectedRelation,
    WorthQueryApplicationProjectionDenial, WorthQueryApplicationProjectionDenialKind,
    WorthQueryApplicationProjectionRow, WorthQueryApplicationProjectionRows,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationOmission {
    classification: String,
    required_disclosure: AspectValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationDisclosed<Value> {
    Disclosed(Value),
    Omitted(WorthQueryApplicationOmission),
}

impl WorthQueryApplicationOmission {
    pub fn classification(&self) -> &str {
        &self.classification
    }

    pub const fn required_disclosure(&self) -> &AspectValue {
        &self.required_disclosure
    }
}

impl<Value> WorthQueryApplicationDisclosed<Value> {
    pub(super) fn into_required(
        self,
        kind: WorthQueryApplicationProjectionDenialKind,
    ) -> Result<Value, WorthQueryApplicationProjectionDenial> {
        match self {
            Self::Disclosed(value) => Ok(value),
            Self::Omitted(omission) => Err(projection_denial(kind, omission.classification)),
        }
    }
}

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
        let value = Field::Binding::decode(projected.value()).map_err(|_| {
            projection_denial(
                WorthQueryApplicationProjectionDenialKind::FieldTypeMismatch,
                projected.result_path(),
            )
        })?;
        Ok(WorthQueryApplicationDisclosed::Disclosed(value))
    }

    pub fn disclosed_optional_field<Slot, Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &self,
        selector: ApplicationQueryOptionalResultFieldRef<
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
    ) -> Result<WorthQueryApplicationDisclosed<Option<Value>>, WorthQueryApplicationProjectionDenial>
    where
        Field: OptionalApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
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
        if !projected.matches_optional(&selector) {
            return Err(projection_denial(
                WorthQueryApplicationProjectionDenialKind::FieldContractMismatch,
                projected.result_path(),
            ));
        }
        let value = Field::Binding::decode(projected.value()).map_err(|_| {
            projection_denial(
                WorthQueryApplicationProjectionDenialKind::FieldTypeMismatch,
                projected.result_path(),
            )
        })?;
        Ok(WorthQueryApplicationDisclosed::Disclosed(Some(value)))
    }

    pub fn disclosed_optional<Slot, Relation, From, To, Direction>(
        &self,
        selector: ApplicationQueryResultRelationRef<
            Query,
            Slot,
            Schema,
            Relation,
            From,
            To,
            Direction,
            OptionalOneResult,
        >,
    ) -> Result<
        WorthQueryApplicationDisclosed<
            Option<WorthQueryApplicationProjectionRow<'_, Schema, Query>>,
        >,
        WorthQueryApplicationProjectionDenial,
    >
    where
        Direction: ApplicationQueryResultTraversal,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        self.disclosed_relation(&selector).and_then(|disclosure| {
            disclosure.map(|relation| match relation.rows() {
                [] => Ok(None),
                [row] => Ok(Some(WorthQueryApplicationProjectionRow::new(
                    self.node.child(row),
                    self.governance,
                ))),
                _ => Err(relation_cardinality_denial(relation)),
            })
        })
    }

    pub fn disclosed_one<Slot, Relation, From, To, Direction>(
        &self,
        selector: ApplicationQueryResultRelationRef<
            Query,
            Slot,
            Schema,
            Relation,
            From,
            To,
            Direction,
            ExactlyOneResult,
        >,
    ) -> Result<
        WorthQueryApplicationDisclosed<WorthQueryApplicationProjectionRow<'_, Schema, Query>>,
        WorthQueryApplicationProjectionDenial,
    >
    where
        Direction: ApplicationQueryResultTraversal,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        self.disclosed_relation(&selector).and_then(|disclosure| {
            disclosure.map(|relation| match relation.rows() {
                [row] => Ok(WorthQueryApplicationProjectionRow::new(
                    self.node.child(row),
                    self.governance,
                )),
                _ => Err(relation_cardinality_denial(relation)),
            })
        })
    }

    pub fn disclosed_many<Slot, Relation, From, To, Direction>(
        &self,
        selector: ApplicationQueryResultRelationRef<
            Query,
            Slot,
            Schema,
            Relation,
            From,
            To,
            Direction,
            ManyResults,
        >,
    ) -> Result<
        WorthQueryApplicationDisclosed<WorthQueryApplicationProjectionRows<'_, Schema, Query>>,
        WorthQueryApplicationProjectionDenial,
    >
    where
        Direction: ApplicationQueryResultTraversal,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        self.disclosed_relation(&selector).and_then(|disclosure| {
            disclosure.map(|relation| {
                Ok(WorthQueryApplicationProjectionRows {
                    rows: relation.rows(),
                    disclosure_parent: self.node,
                    governance: self.governance,
                    _marker: std::marker::PhantomData,
                })
            })
        })
    }

    pub(super) fn disclosed_relation<Slot, Relation, From, To, Direction, Cardinality>(
        &self,
        selector: &ApplicationQueryResultRelationRef<
            Query,
            Slot,
            Schema,
            Relation,
            From,
            To,
            Direction,
            Cardinality,
        >,
    ) -> Result<
        WorthQueryApplicationDisclosed<&WorthQueryApplicationProjectedRelation>,
        WorthQueryApplicationProjectionDenial,
    >
    where
        Direction: ApplicationQueryResultTraversal,
        Cardinality: ApplicationQueryResultRelationCardinality,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        let slot = selector.slot_key();
        if let Some(omission) = self.omission(&slot) {
            return Ok(WorthQueryApplicationDisclosed::Omitted(omission));
        }
        if !self.governance.is_disclosed(&slot) {
            return Err(projection_denial(
                WorthQueryApplicationProjectionDenialKind::RelationContractMismatch,
                selector.slot_type(),
            ));
        }
        let projected = self.node.relation(selector.slot_type()).ok_or_else(|| {
            projection_denial(
                WorthQueryApplicationProjectionDenialKind::RelationNotProjected,
                selector.slot_type(),
            )
        })?;
        if projected.matches(selector) {
            Ok(WorthQueryApplicationDisclosed::Disclosed(projected))
        } else {
            Err(projection_denial(
                WorthQueryApplicationProjectionDenialKind::RelationContractMismatch,
                projected.result_path(),
            ))
        }
    }

    fn omission(
        &self,
        slot: &worth_query_declaration::facade::application_query::ApplicationQueryResultSlotKey,
    ) -> Option<WorthQueryApplicationOmission> {
        self.governance
            .omission(slot)
            .map(
                |(classification, required_disclosure)| WorthQueryApplicationOmission {
                    classification: classification.to_string(),
                    required_disclosure: required_disclosure.clone(),
                },
            )
    }
}

impl<Value> WorthQueryApplicationDisclosed<Value> {
    fn map<Output>(
        self,
        disclosed: impl FnOnce(Value) -> Result<Output, WorthQueryApplicationProjectionDenial>,
    ) -> Result<WorthQueryApplicationDisclosed<Output>, WorthQueryApplicationProjectionDenial> {
        match self {
            Self::Disclosed(value) => {
                disclosed(value).map(WorthQueryApplicationDisclosed::Disclosed)
            }
            Self::Omitted(omission) => Ok(WorthQueryApplicationDisclosed::Omitted(omission)),
        }
    }
}
