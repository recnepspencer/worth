//! Lowering a declared [`NameSelection`] onto Relational's selection at the
//! reader's basis.

use std::collections::BTreeSet;

use worth_query_declaration::facade::application_naming::NameSelection;
use worth_relational::facade::lineage::{
    PersistentNameKey, PersistentNameObservationView, PersistentNameSelection,
    PersistentNameSubjectKind,
};

/// A declared selection as Relational's resolver takes it.
pub(super) enum LoweredSelection {
    Relational(PersistentNameSelection),
    /// A requested carrier has no recorded birth at the basis, so no live
    /// descendant can be bounded by it.
    UnrecordedCarrier,
}

/// Lowers `selection`, admitting each carrier only from a birth the index
/// recorded at the basis.
///
/// Complexity: O(carriers) expected probes.
pub(super) fn lower_selection(
    view: &PersistentNameObservationView<'_>,
    selection: &NameSelection,
) -> LoweredSelection {
    let lowered = match selection {
        NameSelection::Exact => PersistentNameSelection::Exact,
        NameSelection::AllDescendants => PersistentNameSelection::AllDescendants,
        NameSelection::KeepSide { discriminator } => PersistentNameSelection::KeepSide {
            discriminator: discriminator.as_bytes().to_vec(),
        },
        NameSelection::BoundedByCarriers { carriers } => {
            let mut lowered = BTreeSet::new();
            for carrier in carriers.as_slice() {
                let recorded = view
                    .recorded_name(PersistentNameSubjectKind::Entity, carrier.as_bytes())
                    .or_else(|| {
                        view.recorded_name(PersistentNameSubjectKind::Relation, carrier.as_bytes())
                    });
                match recorded {
                    Some(PersistentNameKey::Entity(identity))
                    | Some(PersistentNameKey::Relation(identity)) => {
                        lowered.insert(identity);
                    }
                    Some(PersistentNameKey::Group(_)) | None => {
                        return LoweredSelection::UnrecordedCarrier;
                    }
                }
            }
            PersistentNameSelection::BoundedByCarriers { carriers: lowered }
        }
    };
    LoweredSelection::Relational(lowered)
}
