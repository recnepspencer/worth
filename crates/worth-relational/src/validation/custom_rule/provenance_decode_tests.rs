//! A provenance artifact written before receipts carried a non-applicability
//! basis still decodes, and decodes as "no basis": never as vacuous truth.
use super::{CustomInvariantNonApplicability, CustomInvariantProvenance};
use crate::identity::data::VersionId;
use crate::validation::data::{
    CustomInvariantTraversalSummary, StructuralCountView, TouchedStructuralSet,
};
use crate::validation::engine::InvariantObservationKind;

fn provenance(
    non_applicability: Option<CustomInvariantNonApplicability>,
) -> CustomInvariantProvenance {
    let touched = TouchedStructuralSet::new(
        Vec::new().into(),
        Vec::new().into(),
        Vec::new().into(),
        Vec::new().into(),
        Vec::new().into(),
        Vec::new().into(),
        Vec::new().into(),
        Vec::new().into(),
        Vec::new().into(),
    );
    CustomInvariantProvenance {
        observation_kind: InvariantObservationKind::Speculative,
        version_id: VersionId::new(3),
        current_version_id: VersionId::new(3),
        touched: touched.provenance_summary(),
        counts: StructuralCountView::from_touched_scope(&touched),
        traversal: CustomInvariantTraversalSummary {
            consumed_frontier: 0,
            consumed_steps: 0,
            remaining_frontier: 0,
            remaining_steps: 0,
            max_depth: 0,
        },
        proposal_identity: None,
        work_units: std::num::NonZeroU64::MIN,
        non_applicability,
    }
}

#[test]
fn a_named_artifact_without_the_basis_decodes_as_no_basis() {
    let mut named = serde_json::to_value(provenance(Some(
        CustomInvariantNonApplicability::CandidateOutsideClosure,
    )))
    .unwrap();
    named
        .as_object_mut()
        .unwrap()
        .remove("non_applicability")
        .expect("the basis is a named field");
    let decoded: CustomInvariantProvenance = serde_json::from_value(named).unwrap();
    assert_eq!(decoded, provenance(None));
}

/// Positional encodings have no field names to miss: the older artifact is
/// one element shorter, and only a field default lets it decode.
#[test]
fn a_positional_artifact_without_the_basis_decodes_as_no_basis() {
    let current = rmp_serde::to_vec(&provenance(None)).unwrap();
    // fixarray of eight elements, the last one the basis encoded as nil.
    let (Some((&0x98, fields)), Some(&0xc0)) = (current.split_first(), current.last()) else {
        panic!("provenance encodes as an eight-element array ending in a nil basis")
    };
    let mut older = vec![0x97];
    older.extend_from_slice(&fields[..fields.len() - 1]);
    let decoded: CustomInvariantProvenance =
        rmp_serde::from_slice(&older).expect("an artifact without the basis decodes");
    assert_eq!(decoded, provenance(None));
}
