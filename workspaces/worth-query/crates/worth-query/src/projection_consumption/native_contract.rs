use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use worth_foundational::facade::{
    canonical_basis_sequence_material, prepare_aspect_contract_for_canonical_basis, AbsenceLaw,
    AspectContract, AspectShape, AspectValuePosture, CanonicalFieldPath,
    CanonicalizationRuleVersion, FieldKey,
};

use super::ProjectionFactFieldPath;

static NEXT_NATIVE_SELECTION_IDENTITY: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeclaredNativeAspectContractBasis {
    contract: AspectContract,
    canonical_contract_material: String,
}

impl DeclaredNativeAspectContractBasis {
    pub(crate) fn new(contract: AspectContract) -> Arc<Self> {
        let version = CanonicalizationRuleVersion::new("worth-query-native-contract-v1")
            .expect("the fixed Query native-contract canonicalization version is valid");
        let ready = prepare_aspect_contract_for_canonical_basis(version, contract.clone())
            .into_result()
            .expect("an installed Foundational aspect contract has canonical material");
        Arc::new(Self {
            contract,
            canonical_contract_material: canonical_basis_sequence_material(ready.payload()),
        })
    }

    pub(crate) fn contract(&self) -> &AspectContract {
        &self.contract
    }

    pub(crate) fn canonical_contract_material(&self) -> &str {
        &self.canonical_contract_material
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeclaredNativeFactContract {
    basis: Arc<DeclaredNativeAspectContractBasis>,
    selection_identity: u64,
    field_path: ProjectionFactFieldPath,
    expected_shape: AspectValuePosture,
    absence: AbsenceLaw,
}

impl DeclaredNativeFactContract {
    pub(crate) fn whole(
        basis: Arc<DeclaredNativeAspectContractBasis>,
        projection_is_whole_aspect: bool,
    ) -> Result<Self, DeclaredNativeFactContractDenial> {
        if !projection_is_whole_aspect {
            return Err(DeclaredNativeFactContractDenial::WholeAspectNotProjected);
        }
        let expected_shape = match basis.contract().shape() {
            AspectShape::Scalar(family) => AspectValuePosture::Scalar(*family),
            AspectShape::Struct(_) => AspectValuePosture::Struct,
            AspectShape::Opaque(_) | AspectShape::Reference(_) => {
                return Err(DeclaredNativeFactContractDenial::UnsupportedAspectShape)
            }
        };
        let field_path =
            ProjectionFactFieldPath::from_native_aspect_key(basis.contract().key().clone());
        let absence = basis.contract().absence();
        Ok(Self {
            basis,
            selection_identity: next_selection_identity(),
            field_path,
            expected_shape,
            absence,
        })
    }

    pub(crate) fn field(
        basis: Arc<DeclaredNativeAspectContractBasis>,
        projected_paths: &[CanonicalFieldPath],
        projection_is_whole_aspect: bool,
        field: &FieldKey,
    ) -> Result<Self, DeclaredNativeFactContractDenial> {
        let AspectShape::Struct(shape) = basis.contract().shape() else {
            return Err(DeclaredNativeFactContractDenial::FieldRequiresStruct);
        };
        let Some(declaration) = shape.field(field) else {
            return Err(DeclaredNativeFactContractDenial::UnknownField);
        };
        let projected = projection_is_whole_aspect
            || projected_paths
                .iter()
                .any(|path| path.fields() == std::slice::from_ref(field));
        if !projected {
            return Err(DeclaredNativeFactContractDenial::FieldNotProjected);
        }
        let expected_shape = AspectValuePosture::Scalar(declaration.value_type());
        let absence = declaration.absence();
        let field_path = ProjectionFactFieldPath::from_native_keys(
            basis.contract().key().clone(),
            field.clone(),
        );
        Ok(Self {
            basis,
            selection_identity: next_selection_identity(),
            field_path,
            expected_shape,
            absence,
        })
    }

    pub(crate) fn contract(&self) -> &AspectContract {
        self.basis.contract()
    }

    pub(crate) fn field_path(&self) -> &ProjectionFactFieldPath {
        &self.field_path
    }

    pub(crate) fn expected_shape(&self) -> AspectValuePosture {
        self.expected_shape
    }

    pub(crate) fn absence(&self) -> AbsenceLaw {
        self.absence
    }

    pub(crate) fn canonical_contract_material(&self) -> &str {
        self.basis.canonical_contract_material()
    }

    pub(crate) fn selection_identity(&self) -> u64 {
        self.selection_identity
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DeclaredNativeFactContractDenial {
    WholeAspectNotProjected,
    FieldRequiresStruct,
    UnknownField,
    FieldNotProjected,
    UnsupportedAspectShape,
}

fn next_selection_identity() -> u64 {
    NEXT_NATIVE_SELECTION_IDENTITY.fetch_add(1, Ordering::Relaxed)
}
