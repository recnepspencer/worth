use worth_query_declaration::facade::application_schema::ErasedApplicationSchemaDeclaration;

use crate::application_operation::{
    compile_portable_operation_contract_records,
    WorthQueryPortableOperationContractSpineDenialKind as OperationDenial,
};
use crate::application_schema::compile_portable_native_contract_records;
use crate::application_schema::WorthQueryApplicationSchemaContractCatalogDenialKind as NativeDenial;

use super::{
    WorthQueryPortableApplicationOperationContractRecord,
    WorthQueryPortableNativeAspectContractRecord,
};
use crate::package::WorthQueryPortablePackageValidationDenial;

/// Runtime-neutral 9.16.1.1 contract meaning retained by one validated package.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationContractSpine {
    native_aspects: Vec<WorthQueryPortableNativeAspectContractRecord>,
    operations: Vec<WorthQueryPortableApplicationOperationContractRecord>,
}

impl WorthQueryPortableApplicationContractSpine {
    pub fn native_aspects(&self) -> &[WorthQueryPortableNativeAspectContractRecord] {
        &self.native_aspects
    }

    pub fn operations(&self) -> &[WorthQueryPortableApplicationOperationContractRecord] {
        &self.operations
    }
}

pub(crate) fn compile_application_contract_spine(
    schemas: &[ErasedApplicationSchemaDeclaration],
) -> Result<WorthQueryPortableApplicationContractSpine, WorthQueryPortablePackageValidationDenial> {
    let mut native_aspects = Vec::new();
    let mut operations = Vec::new();
    for schema in schemas {
        let schema_native = compile_portable_native_contract_records(schema).map_err(|denial| {
            let kind = match denial.kind() {
                NativeDenial::DuplicateAspectIdentity => {
                    DenialKind::ApplicationContractDuplicateAspectIdentity
                }
                NativeDenial::DuplicateAspectLocus => {
                    DenialKind::ApplicationContractDuplicateAspectLocus
                }
                NativeDenial::DuplicateFieldLocus => {
                    DenialKind::ApplicationContractDuplicateFieldLocus
                }
                NativeDenial::RevisionZero => DenialKind::ApplicationContractRevisionZero,
                NativeDenial::MissingAspectFieldClosure => {
                    DenialKind::ApplicationContractMissingAspectFieldClosure
                }
                NativeDenial::FieldWithoutAspect => {
                    DenialKind::ApplicationContractFieldWithoutAspect
                }
                NativeDenial::InvalidAspectKey => DenialKind::ApplicationContractInvalidAspectKey,
                NativeDenial::InvalidFieldKey => DenialKind::ApplicationContractInvalidFieldKey,
                NativeDenial::InvalidAspectShape => {
                    DenialKind::ApplicationContractInvalidAspectShape
                }
                NativeDenial::ProjectionMaskRejected => {
                    DenialKind::ApplicationContractProjectionMaskRejected
                }
                NativeDenial::CanonicalContractRejected
                | NativeDenial::CanonicalEntryBudgetExceeded
                | NativeDenial::CanonicalEncodedByteBudgetExceeded => unreachable!(
                    "portable native contract compilation performs no canonical preparation"
                ),
            };
            WorthQueryPortablePackageValidationDenial::invalid_application_contract_spine(
                kind,
                format!("{}:{}", schema.name(), denial.subject()),
            )
        })?;
        let schema_operations = compile_portable_operation_contract_records(schema, &schema_native)
            .map_err(|denial| {
                let kind = match denial {
                    OperationDenial::MissingNativeAspect => {
                        DenialKind::ApplicationOperationContractMissingNativeAspect
                    }
                    OperationDenial::MissingNativeField => {
                        DenialKind::ApplicationOperationContractMissingNativeField
                    }
                    OperationDenial::InvalidProjectionMask => {
                        DenialKind::ApplicationOperationContractInvalidProjectionMask
                    }
                    OperationDenial::AmbiguousExternalEffect => {
                        DenialKind::ApplicationOperationContractAmbiguousExternalEffect
                    }
                    OperationDenial::AmbiguousAftermath => {
                        DenialKind::ApplicationOperationContractAmbiguousAftermath
                    }
                    OperationDenial::AmbiguousBody => {
                        DenialKind::ApplicationOperationContractAmbiguousBody
                    }
                };
                WorthQueryPortablePackageValidationDenial::invalid_application_contract_spine(
                    kind,
                    schema.name(),
                )
            })?;
        native_aspects.extend(schema_native);
        operations.extend(schema_operations);
    }
    Ok(WorthQueryPortableApplicationContractSpine {
        native_aspects,
        operations,
    })
}

use crate::package::WorthQueryPortablePackageValidationDenialKind as DenialKind;
