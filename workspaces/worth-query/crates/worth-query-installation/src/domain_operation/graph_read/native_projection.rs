use std::cmp::Ordering;
use std::fmt;
use std::sync::Arc;

use worth_foundational::facade::{
    canonical_basis_sequence_material, prepare_aspect_contract_for_canonical_basis,
    prepare_aspect_mask_for_canonical_basis, prepare_canonical_basis_bundle,
    prepare_canonical_export_bundle, AspectContract, AspectMask, CanonicalBasisReadyArtifact,
    CanonicalEquivalenceBasis, CanonicalExportReadyArtifact, CanonicalProducerShape,
    CanonicalizationRuleVersion, ProjectionMask,
};

use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;

#[derive(Clone)]
pub struct WorthQueryOperationNativeProjectionContract {
    contract: AspectContract,
    canonical_contract_basis: Arc<CanonicalBasisReadyArtifact>,
    canonical_mask_basis: CanonicalBasisReadyArtifact,
    canonical_export: Arc<CanonicalExportReadyArtifact>,
    canonical_contract_material: Arc<str>,
    mask: AspectMask<ProjectionMask>,
}

impl WorthQueryOperationNativeProjectionContract {
    pub fn new(
        contract: AspectContract,
        mask: AspectMask<ProjectionMask>,
    ) -> Result<Self, worth_foundational::facade::MaskAdmissibilityDenial> {
        contract.admits_projection_mask(&mask)?;
        let version = CanonicalizationRuleVersion::new("worth-query-native-contract-v1")
            .expect("the fixed Query native-contract canonicalization version is valid");
        let canonical_contract_basis =
            prepare_aspect_contract_for_canonical_basis(version.clone(), contract.clone())
                .into_result()
                .expect("a constructed Foundational aspect contract has canonical material");
        let canonical_mask_basis = prepare_aspect_mask_for_canonical_basis(
            version.clone(),
            contract.key().clone(),
            mask.clone(),
        )
        .into_result()
        .expect("an admitted Foundational projection mask has canonical material");
        let canonical_bundle = prepare_canonical_basis_bundle(
            version,
            [
                canonical_contract_basis.clone(),
                canonical_mask_basis.clone(),
            ],
        )
        .into_result()
        .expect("the native contract and mask form a coherent canonical bundle");
        let canonical_export = prepare_canonical_export_bundle(
            "worth-query-installed-native-projection-v1",
            CanonicalProducerShape::NativeFoundational,
            CanonicalEquivalenceBasis::ExactCanonicalBasis,
            canonical_bundle,
        )
        .into_result()
        .expect("the coherent native projection bundle is export-ready");
        let canonical_contract_material =
            canonical_basis_sequence_material(canonical_contract_basis.payload());
        Ok(Self {
            contract,
            canonical_contract_basis: Arc::new(canonical_contract_basis),
            canonical_mask_basis,
            canonical_export: Arc::new(canonical_export),
            canonical_contract_material: Arc::from(canonical_contract_material),
            mask,
        })
    }

    pub(crate) fn from_installed(
        installed: &crate::application_schema::WorthQueryInstalledApplicationAspectContract,
        mask: AspectMask<ProjectionMask>,
    ) -> Result<Self, worth_foundational::facade::MaskAdmissibilityDenial> {
        installed.contract().admits_projection_mask(&mask)?;
        let version = CanonicalizationRuleVersion::new("worth-query-native-contract-v1")
            .expect("the fixed Query native-contract canonicalization version is valid");
        let canonical_mask_basis = prepare_aspect_mask_for_canonical_basis(
            version.clone(),
            installed.contract().key().clone(),
            mask.clone(),
        )
        .into_result()
        .expect("an admitted Foundational projection mask has canonical material");
        let canonical_bundle = prepare_canonical_basis_bundle(
            version,
            [
                installed.canonical_contract_basis().clone(),
                canonical_mask_basis.clone(),
            ],
        )
        .into_result()
        .expect("the installed native contract and mask form a coherent canonical bundle");
        let canonical_export = prepare_canonical_export_bundle(
            "worth-query-installed-native-projection-v1",
            CanonicalProducerShape::NativeFoundational,
            CanonicalEquivalenceBasis::ExactCanonicalBasis,
            canonical_bundle,
        )
        .into_result()
        .expect("the coherent native projection bundle is export-ready");
        Ok(Self {
            contract: installed.contract().clone(),
            canonical_contract_basis: installed.retain_canonical_contract_basis(),
            canonical_mask_basis,
            canonical_export: Arc::new(canonical_export),
            canonical_contract_material: installed.retain_canonical_contract_material(),
            mask,
        })
    }

    pub fn contract(&self) -> &AspectContract {
        &self.contract
    }

    pub fn mask(&self) -> &AspectMask<ProjectionMask> {
        &self.mask
    }

    pub(crate) fn canonical_contract_basis(&self) -> &CanonicalBasisReadyArtifact {
        self.canonical_contract_basis.as_ref()
    }

    pub(crate) fn canonical_mask_basis(&self) -> &CanonicalBasisReadyArtifact {
        &self.canonical_mask_basis
    }

    pub(crate) fn canonical_export(&self) -> &CanonicalExportReadyArtifact {
        self.canonical_export.as_ref()
    }

    pub(crate) fn canonical_contract_material(&self) -> &str {
        self.canonical_contract_material.as_ref()
    }

    pub(crate) fn canonical_order(&self, candidate: &Self) -> Ordering {
        self.canonical_contract_material
            .cmp(&candidate.canonical_contract_material)
            .then_with(|| {
                self.mask
                    .is_whole_aspect()
                    .cmp(&candidate.mask.is_whole_aspect())
            })
            .then_with(|| compare_mask_paths(&self.mask, &candidate.mask))
    }
}

fn compare_mask_paths(
    left: &AspectMask<ProjectionMask>,
    right: &AspectMask<ProjectionMask>,
) -> Ordering {
    left.paths().iter().cmp(right.paths().iter())
}

impl fmt::Debug for WorthQueryOperationNativeProjectionContract {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorthQueryOperationNativeProjectionContract")
            .field("contract", &self.contract)
            .field("mask", &self.mask)
            .finish_non_exhaustive()
    }
}

impl PartialEq for WorthQueryOperationNativeProjectionContract {
    fn eq(&self, other: &Self) -> bool {
        self.contract == other.contract && self.mask == other.mask
    }
}

impl Eq for WorthQueryOperationNativeProjectionContract {}

/// Schema-bound application projection compiled from one installed catalog locus.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOperationApplicationProjectionScope {
    schema: ApplicationSchemaBindingIdentity,
    entity: String,
    aspect: worth_foundational::facade::AspectKey,
    projection: WorthQueryOperationNativeProjectionContract,
}

impl WorthQueryOperationApplicationProjectionScope {
    pub(crate) fn new(
        schema: ApplicationSchemaBindingIdentity,
        entity: String,
        aspect: worth_foundational::facade::AspectKey,
        projection: WorthQueryOperationNativeProjectionContract,
    ) -> Self {
        Self {
            schema,
            entity,
            aspect,
            projection,
        }
    }

    pub const fn schema(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema
    }

    pub fn entity(&self) -> super::WorthQueryOperationEntityReadScopeRef<'_> {
        super::WorthQueryOperationEntityReadScopeRef::new(&self.schema, &self.entity)
    }

    pub const fn aspect(&self) -> &worth_foundational::facade::AspectKey {
        &self.aspect
    }

    pub const fn projection(&self) -> &WorthQueryOperationNativeProjectionContract {
        &self.projection
    }
}
