use std::collections::BTreeMap;

use worth_ui_dsl::WorthUiSealedSemanticPackage;

use crate::capability::{MeasurementConstraint, MosaicSizingContractId};
use crate::declaration::{
    UiDeclarationArtifact, UiDeclarationLowering, UiDeclaredMeasurementBasisSource,
    UiDeclaredMeasurementConstraintModifier,
};
use crate::facade::prepared_application_authority::WorthUiPreparedDeclarationSourceIdentity;
use crate::source::{
    WorthUiLegallyStructuredArtifactInput, WorthUiLegallyStructuredArtifactInputNode,
    WorthUiMosaicStructureFacts,
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorthUiPreparedDeclarationMaterial {
    artifacts: Box<[UiDeclarationArtifact]>,
    identity: WorthUiPreparedDeclarationSourceIdentity,
}

impl WorthUiPreparedDeclarationMaterial {
    pub(crate) fn admit_authored_component_references(
        &mut self,
        snapshot: &crate::capability::CapabilitySnapshot,
    ) -> Result<
        (),
        (
            usize,
            crate::declaration::UiDeclarationComponentReferenceDenial,
        ),
    > {
        for (declaration_index, artifact) in self.artifacts.iter_mut().enumerate() {
            if let Err(reason) = artifact.admit_component_reference(snapshot) {
                return Err((declaration_index, reason));
            }
        }
        Ok(())
    }

    pub(crate) fn identity(&self) -> &WorthUiPreparedDeclarationSourceIdentity {
        &self.identity
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        Vec<UiDeclarationArtifact>,
        WorthUiPreparedDeclarationSourceIdentity,
    ) {
        (self.artifacts.into_vec(), self.identity)
    }

    pub(crate) fn admit_authored_appearance_attachments(
        &mut self,
        snapshot: &crate::capability::CapabilitySnapshot,
    ) -> Result<(), (usize, crate::declaration::UiAppearanceRoleAttachmentDenial)> {
        for (declaration_index, artifact) in self.artifacts.iter_mut().enumerate() {
            if artifact.authored_appearance_role_attachment().is_none() {
                continue;
            }
            if let Err(reason) = artifact.admit_appearance_role_attachment(snapshot) {
                return Err((declaration_index, reason));
            }
        }
        Ok(())
    }
}

pub(in crate::runtime::source_ingress) fn prepare_declaration_material(
    package: &WorthUiSealedSemanticPackage,
    structured: &WorthUiLegallyStructuredArtifactInput,
) -> Result<WorthUiPreparedDeclarationMaterial, ()> {
    let mut artifacts = Vec::with_capacity(package.declaration_lowering_receipts().len() + 1);
    artifacts.push(UiDeclarationLowering::lower_runtime_bootstrap());
    artifacts.extend(
        package
            .declaration_lowering_receipts()
            .into_iter()
            .map(UiDeclarationLowering::lower),
    );
    let claims = runtime_structural_claims(structured)?;
    admit_runtime_structural_claims(&mut artifacts, &claims);
    let component_measurement_claims = runtime_component_measurement_claims(structured);
    admit_component_measurement_claims(&mut artifacts, &component_measurement_claims);
    Ok(WorthUiPreparedDeclarationMaterial {
        artifacts: artifacts.into_boxed_slice(),
        identity: WorthUiPreparedDeclarationSourceIdentity::from_semantic_package(
            package.identity().clone(),
        ),
    })
}

fn runtime_component_measurement_claims(
    structured: &WorthUiLegallyStructuredArtifactInput,
) -> BTreeMap<
    RuntimeStructuralClaimKey,
    (
        Option<UiDeclaredMeasurementBasisSource>,
        crate::declaration::UiDeclaredMeasurementMode,
    ),
> {
    let mut claims = BTreeMap::new();
    for module_id in structured.module_ids() {
        let Some(module) = structured.module(module_id) else {
            continue;
        };
        for node in module.nodes() {
            let WorthUiLegallyStructuredArtifactInputNode::Component(component) = node else {
                continue;
            };
            let Some(contract) = component.descriptor().allocation_measurement_contract() else {
                continue;
            };
            let (basis, mode) = match contract {
                crate::capability::ComponentAllocationMeasurementContract::FillViewport => (
                    Some(UiDeclaredMeasurementBasisSource::ViewportExtent),
                    crate::declaration::UiDeclaredMeasurementMode::FillViewport,
                ),
                crate::capability::ComponentAllocationMeasurementContract::ViewportInset(inset) => {
                    (
                        Some(UiDeclaredMeasurementBasisSource::ViewportExtent),
                        crate::declaration::UiDeclaredMeasurementMode::ViewportInset {
                            horizontal_logical_points: inset.horizontal_logical_points(),
                            vertical_logical_points: inset.vertical_logical_points(),
                        },
                    )
                }
                crate::capability::ComponentAllocationMeasurementContract::ViewportRegion(
                    region,
                ) => (
                    Some(UiDeclaredMeasurementBasisSource::ViewportExtent),
                    crate::declaration::UiDeclaredMeasurementMode::ViewportRegion {
                        horizontal: region.horizontal(),
                        vertical: region.vertical(),
                    },
                ),
                crate::capability::ComponentAllocationMeasurementContract::FixedLogicalSize {
                    width,
                    height,
                } => (
                    None,
                    crate::declaration::UiDeclaredMeasurementMode::FixedLogicalSize {
                        width,
                        height,
                    },
                ),
            };
            claims.insert(
                (
                    component.provenance().module_path().to_owned(),
                    component.provenance().declaration_index(),
                ),
                (basis, mode),
            );
        }
    }
    claims
}

fn admit_component_measurement_claims(
    artifacts: &mut [UiDeclarationArtifact],
    claims: &BTreeMap<
        RuntimeStructuralClaimKey,
        (
            Option<UiDeclaredMeasurementBasisSource>,
            crate::declaration::UiDeclaredMeasurementMode,
        ),
    >,
) {
    for artifact in artifacts {
        let provenance = artifact.provenance().source_provenance();
        let key = (
            provenance.module_path().to_owned(),
            provenance.declaration_index(),
        );
        if let Some((basis, mode)) = claims.get(&key).copied() {
            artifact.admit_source_backed_measurement_mode(Some(mode));
            artifact.admit_source_backed_measurement_basis_source(basis);
        }
    }
}

struct RuntimeStructuralClaims {
    mosaic_membership_name: Box<str>,
    measurement_constraint_modifier: Option<UiDeclaredMeasurementConstraintModifier>,
    measurement_basis_source: Option<UiDeclaredMeasurementBasisSource>,
    mosaic_sizing_contract_id: MosaicSizingContractId,
}

type RuntimeStructuralClaimKey = (String, usize);

enum RuntimeStructuralClaimEntry {
    NonStructuralDeclaration,
    StructuralDeclarationWithoutSizing,
    Denied,
    Admitted {
        key: RuntimeStructuralClaimKey,
        claims: RuntimeStructuralClaims,
    },
}

fn runtime_structural_claims(
    structured: &WorthUiLegallyStructuredArtifactInput,
) -> Result<BTreeMap<RuntimeStructuralClaimKey, RuntimeStructuralClaims>, ()> {
    let mut claims = BTreeMap::new();
    for module_id in structured.module_ids() {
        let Some(module) = structured.module(module_id) else {
            continue;
        };
        for node in module.nodes() {
            match structural_claim_entry(node) {
                RuntimeStructuralClaimEntry::NonStructuralDeclaration
                | RuntimeStructuralClaimEntry::StructuralDeclarationWithoutSizing => {}
                RuntimeStructuralClaimEntry::Denied => return Err(()),
                RuntimeStructuralClaimEntry::Admitted {
                    key,
                    claims: admitted,
                } => {
                    claims.insert(key, admitted);
                }
            }
        }
    }
    Ok(claims)
}

fn structural_claim_entry(
    node: &WorthUiLegallyStructuredArtifactInputNode,
) -> RuntimeStructuralClaimEntry {
    let (family, authored_identity, fallback_identity, provenance, structure) = match node {
        WorthUiLegallyStructuredArtifactInputNode::Component(node) => (
            "component",
            node.authored_identity(),
            node.descriptor().id().as_str(),
            node.provenance(),
            node.structure(),
        ),
        WorthUiLegallyStructuredArtifactInputNode::Surface(node) => (
            "surface",
            node.authored_identity(),
            node.descriptor().id().as_str(),
            node.provenance(),
            node.structure(),
        ),
        WorthUiLegallyStructuredArtifactInputNode::Binding(node) => (
            "binding",
            node.authored_identity(),
            node.view_binding().id().as_str(),
            node.provenance(),
            node.structure(),
        ),
        WorthUiLegallyStructuredArtifactInputNode::Import(_)
        | WorthUiLegallyStructuredArtifactInputNode::Token(_) => {
            return RuntimeStructuralClaimEntry::NonStructuralDeclaration;
        }
    };
    let membership_identity = membership_identity(family, authored_identity, fallback_identity);
    match structural_claims(provenance.module_path(), membership_identity, structure) {
        Ok(Some(claims)) => RuntimeStructuralClaimEntry::Admitted {
            key: (
                provenance.module_path().to_owned(),
                provenance.declaration_index(),
            ),
            claims,
        },
        Ok(None) => RuntimeStructuralClaimEntry::StructuralDeclarationWithoutSizing,
        Err(()) => RuntimeStructuralClaimEntry::Denied,
    }
}

fn structural_claims(
    module_path: &str,
    membership_identity: String,
    structure: &WorthUiMosaicStructureFacts,
) -> Result<Option<RuntimeStructuralClaims>, ()> {
    let Some(sizing_contract_id) = structure.unique_root_sizing_contract_id().map_err(|_| ())?
    else {
        return Ok(None);
    };
    Ok(Some(RuntimeStructuralClaims {
        mosaic_membership_name: format!("source-artifact:{module_path}|{membership_identity}")
            .into(),
        measurement_constraint_modifier: measurement_constraint_modifier(structure),
        measurement_basis_source: measurement_basis_source(structure),
        mosaic_sizing_contract_id: sizing_contract_id,
    }))
}

fn admit_runtime_structural_claims(
    artifacts: &mut [UiDeclarationArtifact],
    claims: &BTreeMap<RuntimeStructuralClaimKey, RuntimeStructuralClaims>,
) {
    for artifact in artifacts {
        let provenance = artifact.provenance().source_provenance();
        let key = (
            provenance.module_path().to_owned(),
            provenance.declaration_index(),
        );
        let Some(claims) = claims.get(&key) else {
            continue;
        };
        artifact.admit_source_backed_mosaic_sizing_contract_id(
            claims.mosaic_sizing_contract_id.clone(),
        );
        artifact.admit_source_backed_mosaic_membership_name(claims.mosaic_membership_name.as_ref());
        artifact.admit_source_backed_measurement_constraint_modifier(
            claims.measurement_constraint_modifier,
        );
        artifact.admit_source_backed_measurement_basis_source(claims.measurement_basis_source);
    }
}

fn membership_identity(
    family: &str,
    authored_identity: Option<&str>,
    fallback_identity: &str,
) -> String {
    match authored_identity {
        Some(identity) => format!("{family}:authored:{identity}"),
        None => format!("{family}:identity:{fallback_identity}"),
    }
}

fn measurement_basis_source(
    structure: &WorthUiMosaicStructureFacts,
) -> Option<UiDeclaredMeasurementBasisSource> {
    use crate::capability::MosaicSizingBehavior;

    structure
        .root_regions()
        .iter()
        .any(|region| {
            matches!(
                region.descriptor().sizing_behavior(),
                Some(MosaicSizingBehavior::OverlayAnchored)
            )
        })
        .then_some(UiDeclaredMeasurementBasisSource::PortalAnchor)
}

fn measurement_constraint_modifier(
    structure: &WorthUiMosaicStructureFacts,
) -> Option<UiDeclaredMeasurementConstraintModifier> {
    structure
        .root_regions()
        .iter()
        .any(|region| {
            region
                .sizing_contract()
                .and_then(|(_, descriptor)| descriptor.named_measurement())
                .is_some_and(|measurement| {
                    !matches!(
                        measurement.constraint(),
                        MeasurementConstraint::Unconstrained
                    )
                })
        })
        .then_some(UiDeclaredMeasurementConstraintModifier::Bounded)
}
