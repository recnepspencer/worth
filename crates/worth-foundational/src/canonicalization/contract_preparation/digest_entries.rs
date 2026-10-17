use worth_proof::{Artifact, TransitionOutcome};

use crate::aspects::{AspectContract, AspectShape};

use super::super::{
    CanonicalDigestAspectShapeKind, CanonicalDigestMaskMode, CanonicalDigestPreparationEntry,
    DigestPreparationReadyAspectContract, DigestPreparationReadyAspectContractArtifact,
};

pub fn prepare_aspect_contract_for_digest(
    contract: AspectContract,
) -> TransitionOutcome<DigestPreparationReadyAspectContractArtifact> {
    let basis = digest_basis_for_aspect_contract(&contract);

    TransitionOutcome::success(Artifact::new(DigestPreparationReadyAspectContract::new(
        contract, basis,
    )))
}

pub fn aspect_contract_digest_preparation_basis(
    ready: &DigestPreparationReadyAspectContractArtifact,
) -> &[CanonicalDigestPreparationEntry] {
    ready.payload().basis()
}

pub(super) fn digest_basis_for_aspect_contract(
    contract: &AspectContract,
) -> Vec<CanonicalDigestPreparationEntry> {
    let mut basis = vec![
        CanonicalDigestPreparationEntry::ContractHeader {
            key: contract.key().clone(),
            identity: contract.identity(),
            revision: contract.revision(),
        },
        CanonicalDigestPreparationEntry::ContractAbsenceLaw {
            key: contract.key().clone(),
            absence: contract.absence(),
        },
        CanonicalDigestPreparationEntry::ContractEquivalenceBasis {
            key: contract.key().clone(),
            equivalence: contract.equivalence(),
        },
        CanonicalDigestPreparationEntry::ContractEvolutionPolicy {
            key: contract.key().clone(),
            evolution: contract.evolution(),
        },
    ];

    basis.extend(digest_basis_for_contract_masks(contract));
    basis.extend(digest_basis_for_contract_shape(contract));
    basis.sort();
    basis
}

fn digest_basis_for_contract_masks(
    contract: &AspectContract,
) -> [CanonicalDigestPreparationEntry; 3] {
    [
        CanonicalDigestPreparationEntry::ContractMaskMode {
            key: contract.key().clone(),
            mode: CanonicalDigestMaskMode::Projection,
            allowed: contract.masks().projection_allowed(),
        },
        CanonicalDigestPreparationEntry::ContractMaskMode {
            key: contract.key().clone(),
            mode: CanonicalDigestMaskMode::Mutation,
            allowed: contract.masks().mutation_allowed(),
        },
        CanonicalDigestPreparationEntry::ContractMaskMode {
            key: contract.key().clone(),
            mode: CanonicalDigestMaskMode::Diagnostic,
            allowed: contract.masks().diagnostic_allowed(),
        },
    ]
}

fn digest_basis_for_contract_shape(
    contract: &AspectContract,
) -> Vec<CanonicalDigestPreparationEntry> {
    match contract.shape() {
        AspectShape::Scalar(scalar) => vec![
            CanonicalDigestPreparationEntry::ContractShape {
                key: contract.key().clone(),
                shape: CanonicalDigestAspectShapeKind::Scalar,
            },
            CanonicalDigestPreparationEntry::ContractScalarShape {
                key: contract.key().clone(),
                scalar: *scalar,
            },
        ],
        AspectShape::Struct(shape) => {
            let mut basis = vec![CanonicalDigestPreparationEntry::ContractShape {
                key: contract.key().clone(),
                shape: CanonicalDigestAspectShapeKind::Struct,
            }];
            basis.extend(shape.fields().iter().map(|field| {
                CanonicalDigestPreparationEntry::ContractStructField {
                    key: contract.key().clone(),
                    field: field.key().clone(),
                    value_type: field.value_type(),
                    requirement: field.requirement(),
                    absence: field.absence(),
                    evolution: field.evolution(),
                }
            }));
            basis.extend(shape.fields().iter().filter_map(|field| {
                field.codec().map(
                    |codec| CanonicalDigestPreparationEntry::ContractFieldCodec {
                        key: contract.key().clone(),
                        field: field.key().clone(),
                        codec,
                    },
                )
            }));
            basis
        }
        AspectShape::Opaque(opaque) => vec![
            CanonicalDigestPreparationEntry::ContractShape {
                key: contract.key().clone(),
                shape: CanonicalDigestAspectShapeKind::Opaque,
            },
            CanonicalDigestPreparationEntry::ContractOpaqueShape {
                key: contract.key().clone(),
                opaque: *opaque,
            },
        ],
        AspectShape::Reference(reference) => vec![
            CanonicalDigestPreparationEntry::ContractShape {
                key: contract.key().clone(),
                shape: CanonicalDigestAspectShapeKind::Reference,
            },
            CanonicalDigestPreparationEntry::ContractReferenceShape {
                key: contract.key().clone(),
                reference: *reference,
            },
        ],
        AspectShape::Content => vec![CanonicalDigestPreparationEntry::ContractShape {
            key: contract.key().clone(),
            shape: CanonicalDigestAspectShapeKind::Content,
        }],
    }
}
