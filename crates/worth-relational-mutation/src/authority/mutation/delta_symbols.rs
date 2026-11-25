//! A delta's symbol ids are admitted before its revision stamps are written.
use super::canonical_deltas::CanonicalRecordAspectDelta;
use super::field_versions::visit_changed_fields;
use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::mutation_input::MutationSymbolMeanings;
use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::reservations::SymbolReservationDenial;
use worth_relational_contracts::internal_port::symbols::data::symbol::Symbol;
use worth_foundational::facade::{AspectKey, FieldKey};

pub(super) struct AdmittedDeltaSymbols<'delta, 'symbols> {
    delta: &'delta CanonicalRecordAspectDelta,
    symbols: &'symbols dyn MutationSymbolMeanings,
}

impl<'delta, 'symbols> AdmittedDeltaSymbols<'delta, 'symbols> {
    pub(super) fn admit(
        delta: &'delta CanonicalRecordAspectDelta,
        symbols: &'symbols mut dyn MutationSymbolMeanings,
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
        Ok(Self { delta, symbols })
    }

    pub(super) fn delta(&self) -> &'delta CanonicalRecordAspectDelta {
        self.delta
    }

    pub(super) fn aspect(&self, aspect: &AspectKey) -> Symbol {
        self.symbols
            .symbol(aspect.as_str())
            .expect(/* Proof: Delta symbol admission walks these exact aspect and field keys before constructing this private admitted delta view. */ "delta admission includes this aspect")
    }

    pub(super) fn field(&self, field: &FieldKey) -> Symbol {
        self.symbols
            .symbol(field.as_str())
            .expect(/* Proof: Delta symbol admission walks these exact aspect and field keys before constructing this private admitted delta view. */ "delta admission includes this field")
    }
}

/// Require exactly the revision references of `deltas`: the surviving deltas
/// of a coupled candidate after superseded rounds were retracted (#119). The
/// same walk as `AdmittedDeltaSymbols::admit`. O(changed fields).
pub fn rebuild_required_revision_references(
    deltas: &[CanonicalRecordAspectDelta],
    symbols: &mut worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::reservations::PendingSymbolMeanings,
) -> Result<(), SymbolReservationDenial> {
    symbols.clear_required_references();
    for delta in deltas {
        AdmittedDeltaSymbols::admit(delta, symbols)?;
    }
    Ok(())
}
