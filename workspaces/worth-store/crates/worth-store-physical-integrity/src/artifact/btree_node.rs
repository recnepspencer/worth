use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{BTreeNodeDenial, BTreeNodeV1};

use crate::artifact::durable_frame_rejection::{input_length, wrong_scope};
use crate::localization::{
    PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause, PhysicalDamageLocalization,
    PhysicalFormatField,
};
use crate::observation::PhysicalIntegrityObservationCounters;
use crate::validation::{
    IntegrityValidatedBTreeNode, PhysicalArtifactScope, PhysicalIntegrityRejection,
    PhysicalIntegrityVersionAxis, UnsupportedPhysicalIntegrityVersion, UntrustedPhysicalArtifact,
};

#[derive(Debug)]
pub enum BTreeNodeIntegrityValidation<'media> {
    Intact(IntegrityValidatedBTreeNode<'media>),
    Rejected(PhysicalIntegrityRejection),
}

pub fn validate_btree_node<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
) -> (
    BTreeNodeIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let byte_count = artifact.byte_count();
    if scope.artifact_family() != PhysicalIntegrityArtifactFamily::BTreeNode {
        return rejected(wrong_scope(scope), byte_count);
    }
    if let Some(rejection) = input_length(scope, byte_count) {
        return rejected(rejection, byte_count);
    }
    let node = match BTreeNodeV1::decode(artifact.bytes()) {
        Ok(node) => node,
        Err(denial) => return rejected(node_denial(scope, denial, artifact.bytes()), byte_count),
    };
    let (_, expected_family) = scope
        .btree_node_identity()
        .expect("B-tree scope carries a record and family code");
    if node.family_code() != expected_family {
        return rejected(
            damage(
                scope,
                PhysicalDamageCause::FamilyMismatch,
                PhysicalFormatField::ArtifactFamily,
                12,
                2,
            ),
            byte_count,
        );
    }
    let validated = IntegrityValidatedBTreeNode::new(scope, node, artifact)
        .expect("decoded B-tree node satisfies its exact validation scope");
    (
        BTreeNodeIntegrityValidation::Intact(validated),
        PhysicalIntegrityObservationCounters::one_intact(
            PhysicalIntegrityArtifactFamily::BTreeNode,
            byte_count,
        ),
    )
}

fn rejected<'media>(
    rejection: PhysicalIntegrityRejection,
    byte_count: u64,
) -> (
    BTreeNodeIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    (
        BTreeNodeIntegrityValidation::Rejected(rejection),
        PhysicalIntegrityObservationCounters::one_rejected(
            PhysicalIntegrityArtifactFamily::BTreeNode,
            byte_count,
            rejection,
        ),
    )
}

fn node_denial(
    scope: PhysicalArtifactScope,
    denial: BTreeNodeDenial,
    bytes: &[u8],
) -> PhysicalIntegrityRejection {
    match denial {
        BTreeNodeDenial::UnsupportedVersion => {
            PhysicalIntegrityRejection::Unsupported(UnsupportedPhysicalIntegrityVersion::new(
                scope,
                PhysicalIntegrityVersionAxis::BTreeNode,
                u32::from(bytes[8]),
            ))
        }
        BTreeNodeDenial::WrongMagic => damage(
            scope,
            PhysicalDamageCause::WrongMagic,
            PhysicalFormatField::Magic,
            0,
            8,
        ),
        BTreeNodeDenial::IntegrityMismatch => damage(
            scope,
            PhysicalDamageCause::ChecksumMismatch,
            PhysicalFormatField::Checksum,
            96,
            4,
        ),
        BTreeNodeDenial::InvalidRecordIdentity => {
            whole_damage(scope, PhysicalDamageCause::ChildReferenceMismatch)
        }
        BTreeNodeDenial::InvalidFamily => damage(
            scope,
            PhysicalDamageCause::FamilyMismatch,
            PhysicalFormatField::ArtifactFamily,
            12,
            2,
        ),
        BTreeNodeDenial::Truncated => whole_damage(scope, PhysicalDamageCause::Truncated),
        BTreeNodeDenial::LengthMismatch | BTreeNodeDenial::NodeTooLarge => {
            whole_damage(scope, PhysicalDamageCause::FramingLengthMismatch)
        }
        _ => whole_damage(scope, PhysicalDamageCause::MalformedStructure),
    }
}

fn damage(
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    field: PhysicalFormatField,
    offset: u64,
    length: u64,
) -> PhysicalIntegrityRejection {
    let offset = scope.byte_range().offset() + offset;
    let range = PhysicalByteRange::new(offset, length)
        .expect("node field has a bounded nonempty byte range");
    PhysicalIntegrityRejection::Damaged(PhysicalDamageLocalization::new(
        scope,
        cause,
        range,
        Some(field),
        PhysicalBlastRadius::CanonicalFrame,
    ))
}

fn whole_damage(
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
) -> PhysicalIntegrityRejection {
    PhysicalIntegrityRejection::Damaged(PhysicalDamageLocalization::new(
        scope,
        cause,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    ))
}
