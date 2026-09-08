use super::{
    artifact_inventory::{ArtifactGranule, ArtifactInventory, FrameGrammar},
    frame_checksum,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

mod journal;
mod common_scope;
mod common_target;
mod enclosures;
mod common_mutation;

pub(super) use common_target::common_inspection_target;
pub(super) use common_mutation::{audit_enclosures,enclosing_paths};

pub(super) fn inspection_target(
    target: &ArtifactGranule, operator: ArtifactOperator,
) -> worth_store::physical_runtime::PhysicalIntegrityScrubTarget {
    if target.grammar == FrameGrammar::Common { target.scrub_target() }
    else { journal::inspection_target(target, operator) }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(super) enum ArtifactOperator {
    CoveredByte,
    Checksum,
    Length,
    ScopeSubstitution,
    Pointer,
    Remove,
    Duplicate,
    SelectiveAggregate,
    Truncate,
    EnvelopeVersion,
    RecordVersion,
}
impl ArtifactOperator {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::CoveredByte => "B",
            Self::Checksum => "K",
            Self::Length => "L",
            Self::ScopeSubstitution => "S",
            Self::Pointer => "P",
            Self::Remove => "R",
            Self::Duplicate => "D",
            Self::SelectiveAggregate => "S-aggregate",
            Self::Truncate => "T",
            Self::EnvelopeVersion => "U-schema",
            Self::RecordVersion => "U-format",
        }
    }
}

pub(super) fn operators(granule: &ArtifactGranule) -> Vec<ArtifactOperator> {
    use ArtifactOperator::*;
    if granule.grammar != FrameGrammar::Common { return journal::operators(granule); }
        let mut operators = vec![
            CoveredByte,
            Checksum,
            Length,
            ScopeSubstitution,
            Truncate,
            EnvelopeVersion,
            RecordVersion,
        ];
        if !matches!(granule.family,"bootstrap_catalog"|"inline_page"|"extent_chunk") {
            operators.push(Pointer);
        }
        operators
}

pub(super) fn apply(
    root: &Path,
    baseline: &Path,
    inventory: &ArtifactInventory,
    index: usize,
    operator: ArtifactOperator,
) {
    let granule = &inventory.granules[index];
    if granule.grammar != FrameGrammar::Common {
        return journal::apply(root, baseline, inventory, index, operator);
    }
    common_mutation::apply(root,baseline,inventory,index,operator);
}

pub(super) fn common_render(baseline:&Path,inventory:&ArtifactInventory,index:usize,operator:ArtifactOperator)->Vec<u8> {
    let granule=&inventory.granules[index];
    let mut bytes = std::fs::read(baseline.join(&granule.path)).unwrap();
    let start = granule.offset();
    let end = start + granule.length();
    match operator {
        ArtifactOperator::CoveredByte => {
            bytes[super::artifact_process::covered_byte_offset(granule)] ^= 1
        }
        ArtifactOperator::Checksum => bytes[start + 44] ^= 1,
        ArtifactOperator::Length => {
            let length = u32::from_le_bytes(bytes[start + 24..start + 28].try_into().unwrap()) + 1;
            bytes[start + 24..start + 28].copy_from_slice(&length.to_le_bytes());
            frame_checksum::refresh_checksum(&mut bytes[start..end]);
        }
        ArtifactOperator::EnvelopeVersion => {
            bytes[start + 9] = 3;
            frame_checksum::refresh_checksum(&mut bytes[start..end]);
        }
        ArtifactOperator::RecordVersion => {
            bytes[start + 10..start + 12].copy_from_slice(&2_u16.to_le_bytes());
            frame_checksum::refresh_checksum(&mut bytes[start..end]);
        }
        ArtifactOperator::Truncate => bytes.truncate(start + granule.length() / 2),
        ArtifactOperator::ScopeSubstitution => {
            common_scope::substitute(&mut bytes,baseline,inventory,index);
        }
        ArtifactOperator::Pointer => common_scope::corrupt_pointer(&mut bytes,granule),
        ArtifactOperator::Remove | ArtifactOperator::Duplicate | ArtifactOperator::SelectiveAggregate => unreachable!("presence operators have a separate namespace contract"),
    }
    bytes
}

pub(super) fn audit(
    before: &[u8],
    after: &[u8],
    granule: &ArtifactGranule,
    operator: ArtifactOperator,
) {
    use ArtifactOperator::*;
    if granule.grammar != FrameGrammar::Common { return journal::audit(before, after, granule, operator); }
    let start = granule.offset();
    let end = start + granule.length();
    if operator == Truncate {
        assert_eq!(after.len(), start + granule.length() / 2);
        assert_eq!(after, &before[..after.len()]);
        return;
    }
    assert_eq!(before.len(), after.len());
    assert_eq!(before[..start], after[..start]);
    assert_eq!(before[end..], after[end..]);
    let changed = before
        .iter()
        .zip(after)
        .enumerate()
        .filter_map(|(i, (a, b))| (a != b).then_some(i - start))
        .collect::<Vec<_>>();
    assert!(!changed.is_empty());
    let allowed = match operator {
        CoveredByte => vec![super::artifact_process::covered_byte_offset(granule) - start],
        Checksum => vec![44],
        Length => (24..28).chain(44..48).collect(),
        EnvelopeVersion => std::iter::once(9).chain(44..48).collect(),
        RecordVersion => (10..12).chain(44..48).collect(),
        ScopeSubstitution => common_scope::substitution_fields(granule).into_iter().flatten().chain(44..48).collect(),
        Pointer => common_scope::pointer_field(granule).chain(44..48).collect(),
        Truncate => unreachable!(),
        Remove | Duplicate | SelectiveAggregate => unreachable!(),
    };
    assert!(changed.iter().all(|index| allowed.contains(index)));
    if matches!(granule.grammar, FrameGrammar::Common) {
        assert_eq!(
            frame_checksum::checksum_is_valid(&after[start..end]),
            !matches!(operator, CoveredByte | Checksum)
        );
    }
    match operator {
        CoveredByte | Checksum => assert_eq!(changed.len(), 1),
        Length => assert_eq!(
            u32::from_le_bytes(after[start + 24..start + 28].try_into().unwrap()) as usize + 48,
            granule.length() + 1
        ),
        EnvelopeVersion => assert_eq!(after[start + 9], 3),
        RecordVersion => assert_eq!(&after[start + 10..start + 12], &2_u16.to_le_bytes()),
        ScopeSubstitution => {
            for range in common_scope::substitution_fields(granule) {
                assert_ne!(&before[start+range.start..start+range.end],&after[start+range.start..start+range.end]);
            }
        }
        Pointer => {let range=common_scope::pointer_field(granule);assert_ne!(&before[start+range.start..start+range.end],&after[start+range.start..start+range.end]);}
        Truncate => unreachable!(),
        Remove | Duplicate | SelectiveAggregate => unreachable!(),
    }
}
