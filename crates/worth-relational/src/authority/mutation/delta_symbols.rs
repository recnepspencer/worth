//! A delta's symbol ids are admitted before its revision stamps are written.
use super::canonical_deltas::CanonicalRecordAspectDelta;
use super::field_versions::visit_changed_fields;
use crate::runtime::{PendingSymbolMeanings, SymbolReservationDenial};
use crate::symbols::data::{StringInterner, Symbol};
use worth_foundational::facade::{AspectKey, FieldKey};

pub(super) struct AdmittedDeltaSymbols<'delta, 'symbols> {
    delta: &'delta CanonicalRecordAspectDelta,
    symbols: &'symbols StringInterner,
}

impl<'delta, 'symbols> AdmittedDeltaSymbols<'delta, 'symbols> {
    pub(super) fn admit(
        delta: &'delta CanonicalRecordAspectDelta,
        symbols: &'symbols mut PendingSymbolMeanings,
    ) -> Result<Self, SymbolReservationDenial> {
        for aspect in &delta.changed_aspects {
            symbols.reserve_revision_reference(aspect.as_str())?;
        }
        for binding in &delta.evaluated_bindings {
            visit_changed_fields(binding, delta.structural_change, |field, _| {
                symbols.reserve_revision_reference(binding.aspect_key.as_str())?;
                symbols.reserve_revision_reference(field.as_str())?;
                Ok(())
            })?;
        }
        Ok(Self {
            delta,
            symbols: symbols.interner(),
        })
    }

    pub(super) fn delta(&self) -> &'delta CanonicalRecordAspectDelta {
        self.delta
    }

    pub(super) fn aspect(&self, aspect: &AspectKey) -> Symbol {
        self.symbols
            .symbol(aspect.as_str())
            .expect("delta admission includes this aspect")
    }

    pub(super) fn field(&self, field: &FieldKey) -> Symbol {
        self.symbols
            .symbol(field.as_str())
            .expect("delta admission includes this field")
    }
}
