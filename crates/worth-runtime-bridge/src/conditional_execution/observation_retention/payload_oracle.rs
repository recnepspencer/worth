use crate::conditional_execution::BridgeConditionalSemanticObservation;
use std::alloc::Layout;
use std::mem::{align_of, size_of};
use std::sync::atomic::AtomicUsize;
use worth_foundational::facade::*;

pub(super) fn arc_allocation<T>() -> usize {
    Layout::new::<[AtomicUsize; 2]>()
        .extend(Layout::new::<T>())
        .unwrap()
        .0
        .pad_to_align()
        .size()
}

pub(super) fn observation_bytes(payload_bytes: usize, mask_bytes: usize) -> u64 {
    // The backing embeds its guard and retains one preallocated observation Vec.
    (arc_allocation::<super::super::baseline::Backing>()
        + size_of::<BridgeConditionalSemanticObservation>()
        + payload_bytes
        + mask_bytes) as u64
}

pub(super) fn baseline_bytes() -> u64 {
    arc_allocation::<super::super::BridgeObservationBaselines>() as u64
}

pub(super) fn available_mask_bytes() -> usize {
    size_of::<CanonicalFieldPath>() + size_of::<FieldKey>() + "available".len()
}

pub(super) fn scalar(value: AspectValue) -> ContractValidatedAspectArtifact {
    let contract = crate::snapshot::SnapshotReadContract::scalar(
        AspectKey::new("balance").unwrap(),
        value.value_family(),
    );
    validate(
        contract.aspect_contract(),
        ContractValidationInput::Scalar(value),
    )
}

pub(super) struct PayloadCase {
    pub artifact: ContractValidatedAspectArtifact,
    pub mask: AspectMask<ProjectionMask>,
    pub payload_bytes: usize,
    pub mask_bytes: usize,
}

pub(super) fn variable_width_cases() -> Vec<PayloadCase> {
    let scalar_values = [
        (
            AspectValue::String(InternedString::Raw("retained text".into())),
            13,
        ),
        (AspectValue::Decimal(CanonicalDecimal::new("12.5")), 4),
        (
            AspectValue::BigInt(CanonicalBigInt::new("12345678901234567890")),
            20,
        ),
        (
            AspectValue::Rational(
                CanonicalRational::new(CanonicalBigInt::new("7"), CanonicalBigInt::new("9"))
                    .unwrap(),
            ),
            2,
        ),
    ];
    let mut cases = scalar_values
        .into_iter()
        .map(|(value, allocation_bytes)| {
            let artifact = scalar(value);
            assert_canonical_scalar_allocation(&artifact, allocation_bytes);
            PayloadCase {
                artifact,
                mask: AspectMask::whole_aspect(),
                // The validated payload and its contract each own "balance".
                payload_bytes: 2 * "balance".len() + allocation_bytes,
                mask_bytes: 0,
            }
        })
        .collect::<Vec<_>>();
    cases.push(struct_case());
    cases
}

fn assert_canonical_scalar_allocation(artifact: &ContractValidatedAspectArtifact, expected: usize) {
    let ContractValidatedAspectValueView::Scalar(value) = artifact.payload().view() else {
        panic!("scalar fixture must retain a scalar");
    };
    // Inspect actual canonical buffers independently of admission's accounting.
    let actual = match value {
        AspectValue::String(InternedString::Raw(value)) => value.capacity(),
        AspectValue::Decimal(value) => value.0.capacity(),
        AspectValue::BigInt(value) => value.0.capacity(),
        AspectValue::Rational(value) => {
            value.numerator.0.capacity() + value.denominator.0.capacity()
        }
        _ => panic!("fixture must exercise an allocated scalar family"),
    };
    assert_eq!(actual, expected);
}

fn struct_case() -> PayloadCase {
    let fields = [
        (
            "available",
            AspectValue::String(InternedString::Raw("stored".into())),
        ),
        (
            "ratio",
            AspectValue::Rational(
                CanonicalRational::new(CanonicalBigInt::new("7"), CanonicalBigInt::new("9"))
                    .unwrap(),
            ),
        ),
    ];
    let shape = StructAspectShape::new(fields.iter().map(|(name, value)| {
        FieldDeclaration::new(
            FieldKey::new(*name).unwrap(),
            value.value_family(),
            FieldRequirement::Required,
            AbsenceLaw::Required,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .unwrap()
    }))
    .unwrap();
    let contract = AspectContract::struct_aspect(
        AspectKey::new("holdings").unwrap(),
        AspectIdentity(701),
        AspectContractRevision(1),
        shape,
    );
    let value = StructAspectValue::new(
        fields
            .into_iter()
            .map(|(key, value)| (FieldKey::new(key).unwrap(), value)),
    )
    .unwrap();
    let artifact = validate(&contract, ContractValidationInput::Struct(value));
    // The pinned tree bound allows one node per entry plus an empty root,
    // eleven key/value slots, twelve child pointers, and a parent pointer.
    let alignment = align_of::<FieldKey>()
        .max(align_of::<AspectValue>())
        .max(align_of::<usize>());
    let tree_node = 11 * (size_of::<FieldKey>() + size_of::<AspectValue>())
        + 13 * size_of::<usize>()
        + 2 * size_of::<u16>()
        + 5 * alignment;
    PayloadCase {
        artifact,
        mask: AspectMask::new([CanonicalFieldPath::single(
            FieldKey::new("available").unwrap(),
        )]),
        payload_bytes: 2 * "holdings".len()
            + 2 * size_of::<FieldDeclaration>()
            + 2 * ("available".len() + "ratio".len())
            + "stored".len()
            + 2
            + 3 * tree_node,
        mask_bytes: available_mask_bytes(),
    }
}

fn validate(
    contract: &AspectContract,
    value: ContractValidationInput,
) -> ContractValidatedAspectArtifact {
    match validate_aspect_value(contract, value) {
        worth_proof::TransitionOutcome::Success(value) => value,
        _ => panic!("real canonical payload must satisfy its declared contract"),
    }
}
