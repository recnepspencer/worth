//! Stable retained prior truth for recorded correction (R8.2 / Resolution A).
//!
//! Query retains the exact pre-image slice demanded by the installed inverse
//! from the decision read-set already observed at admission. Undo consumes this
//! carrier — it must never live-re-read the graph and call the result original.

use worth_foundational::facade::{AspectFieldLocator, AspectValue, CanonicalFieldPath, FieldKey};
use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, DeclaredApplicationFieldValue,
    InstalledPreImageLocus,
};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::transactions::RecordRef;

/// One retained field pre-image bound into the strengthened commit receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRetainedPreImageField {
    locus: InstalledPreImageLocus,
    value: AspectValue,
    encoded_bytes: usize,
    target_record: RecordRef,
    entity_kind: KindId,
    locator: AspectFieldLocator,
}

impl WorthQueryRetainedPreImageField {
    pub const fn locus(&self) -> &InstalledPreImageLocus {
        &self.locus
    }

    pub const fn value(&self) -> &AspectValue {
        &self.value
    }

    pub const fn encoded_bytes(&self) -> usize {
        self.encoded_bytes
    }

    /// Exact record from which this historical value was observed.
    pub const fn target_record(&self) -> &RecordRef {
        &self.target_record
    }

    pub const fn entity_kind(&self) -> KindId {
        self.entity_kind
    }

    pub const fn locator(&self) -> &AspectFieldLocator {
        &self.locator
    }
}

/// Exact pre-image slice retained for a recorded inverse (R8.2).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRetainedPreImage {
    fields: Vec<WorthQueryRetainedPreImageField>,
    total_encoded_bytes: usize,
    _private: (),
}

impl WorthQueryRetainedPreImage {
    pub fn fields(&self) -> &[WorthQueryRetainedPreImageField] {
        &self.fields
    }

    pub const fn total_encoded_bytes(&self) -> usize {
        self.total_encoded_bytes
    }

    pub fn field(
        &self,
        locus: &InstalledPreImageLocus,
    ) -> Option<&WorthQueryRetainedPreImageField> {
        self.fields.iter().find(|field| field.locus() == locus)
    }

    pub fn field_for<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &self,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Option<&WorthQueryRetainedPreImageField>
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        self.fields.iter().find(|retained| {
            retained.locus().entity() == field.entity()
                && retained.locus().aspect() == field.aspect()
                && retained.locus().field() == field.field()
        })
    }

    /// One exact target when every demanded field came from the same record.
    pub fn target_record(&self) -> Option<&RecordRef> {
        let first = self.fields.first()?.target_record();
        self.fields
            .iter()
            .all(|field| field.target_record() == first)
            .then_some(first)
    }

    pub(crate) fn from_retention_seal(
        seal: crate::domain_computation::primary_graph::WorthQueryRetainedPreImageSeal,
    ) -> Self {
        let (fields, total_encoded_bytes) = seal.into_parts();
        let fields = fields
            .into_iter()
            .map(|field| {
                let (locus, value, encoded_bytes, target_record, entity_kind, locator) =
                    field.into_parts();
                WorthQueryRetainedPreImageField {
                    locus,
                    value,
                    encoded_bytes,
                    target_record,
                    entity_kind,
                    locator,
                }
            })
            .collect();
        Self {
            fields,
            total_encoded_bytes,
            _private: (),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPreImageRetentionDenial {
    MissingDemandedField,
    ExceedsByteBound,
    EmptyDemand,
    /// The demanded slot was observed on more than one mutated record, so no
    /// single prior truth is named (Q8.26-C7).
    AmbiguousDemandedField,
    /// The operation mutates nothing, so there is no prior truth to invert.
    NoMutatedRecord,
}

/// The demanded field slot an observed path names, if any (Q8.26-C3).
///
/// A demand names a single field slot and has no vocabulary for a nested path,
/// so only an exactly-one-segment path names a demanded field. Reducing a longer
/// path to its first segment — as this did before slice 10 — lets a nested
/// `Account.Status` observation satisfy a demand for `Status` and bind the wrong
/// value as the prior truth.
pub fn demanded_field_slot(path: &CanonicalFieldPath) -> Option<&FieldKey> {
    match path.fields() {
        [field] => Some(field),
        _ => None,
    }
}
