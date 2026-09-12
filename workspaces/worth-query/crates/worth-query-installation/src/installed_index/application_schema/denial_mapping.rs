use super::super::{
    WorthQueryInstalledPackageIndexDenial, WorthQueryInstalledPackageIndexDenialKind,
};
use crate::application_schema::WorthQueryApplicationValueBindingInstallationDenialKind as ValueBindingDenialKind;
use crate::application_schema::{
    ApplicationSchemaCompilationDenial, WorthQueryApplicationSchemaContractCatalogDenial,
    WorthQueryApplicationSchemaContractCatalogDenialKind as CatalogDenialKind,
    WorthQueryInstalledApplicationSchemaDenial, WorthQueryInstalledApplicationSchemaDenialKind,
};

pub(super) fn map_compilation_denial(
    schema: &str,
    denial: ApplicationSchemaCompilationDenial,
) -> WorthQueryInstalledApplicationSchemaDenial {
    let (kind, subject) = match denial {
        ApplicationSchemaCompilationDenial::Capability(denial) => {
            let kind = match denial.kind() {
                crate::application_capability::WorthQueryApplicationCapabilityInstallationDenialKind::CanonicalEntryLimitExceeded => {
                    WorthQueryInstalledApplicationSchemaDenialKind::CanonicalEntryBudgetExceeded
                }
                crate::application_capability::WorthQueryApplicationCapabilityInstallationDenialKind::CanonicalByteLimitExceeded => {
                    WorthQueryInstalledApplicationSchemaDenialKind::CanonicalEncodedByteBudgetExceeded
                }
                crate::application_capability::WorthQueryApplicationCapabilityInstallationDenialKind::CapabilityNotInstalled
                | crate::application_capability::WorthQueryApplicationCapabilityInstallationDenialKind::CapabilityMeaningChanged
                | crate::application_capability::WorthQueryApplicationCapabilityInstallationDenialKind::ForeignRuntime
                | crate::application_capability::WorthQueryApplicationCapabilityInstallationDenialKind::StaleGeneration
                | crate::application_capability::WorthQueryApplicationCapabilityInstallationDenialKind::PackageIdentityChanged
                | crate::application_capability::WorthQueryApplicationCapabilityInstallationDenialKind::SchemaMeaningChanged
                | crate::application_capability::WorthQueryApplicationCapabilityInstallationDenialKind::CanonicalDigestSlotRejected
                | crate::application_capability::WorthQueryApplicationCapabilityInstallationDenialKind::AuthorityMismatch => {
                    WorthQueryInstalledApplicationSchemaDenialKind::CapabilityInstallationDenied
                }
            };
            (kind, denial.subject().to_string())
        }
        ApplicationSchemaCompilationDenial::Canonical(
            worth_foundational::facade::CanonicalDigestDerivationDenial::EntryLimitExceeded {
                ..
            },
        ) => (
            WorthQueryInstalledApplicationSchemaDenialKind::CanonicalEntryBudgetExceeded,
            schema.to_string(),
        ),
        ApplicationSchemaCompilationDenial::Canonical(
            worth_foundational::facade::CanonicalDigestDerivationDenial::EncodedByteLimitExceeded {
                ..
            },
        ) => (
            WorthQueryInstalledApplicationSchemaDenialKind::CanonicalEncodedByteBudgetExceeded,
            schema.to_string(),
        ),
        ApplicationSchemaCompilationDenial::Canonical(
            worth_foundational::facade::CanonicalDigestDerivationDenial::UnsupportedAlgorithm
            | worth_foundational::facade::CanonicalDigestDerivationDenial::RuleVersionMismatch
            | worth_foundational::facade::CanonicalDigestDerivationDenial::InputDomainMismatch
            | worth_foundational::facade::CanonicalDigestDerivationDenial::InputShapeMismatch,
        ) => (
            WorthQueryInstalledApplicationSchemaDenialKind::CanonicalDigestSlotRejected,
            schema.to_string(),
        ),
        ApplicationSchemaCompilationDenial::Contribution(_) => (
            WorthQueryInstalledApplicationSchemaDenialKind::SchemaMeaningChanged,
            schema.to_string(),
        ),
        ApplicationSchemaCompilationDenial::Query(denial) => {
            let kind = match denial.kind() {
                crate::application_query::WorthQueryApplicationQueryInstallationDenialKind::CanonicalEntryBudgetExceeded => WorthQueryInstalledApplicationSchemaDenialKind::CanonicalEntryBudgetExceeded,
                crate::application_query::WorthQueryApplicationQueryInstallationDenialKind::CanonicalEncodedByteBudgetExceeded => WorthQueryInstalledApplicationSchemaDenialKind::CanonicalEncodedByteBudgetExceeded,
                crate::application_query::WorthQueryApplicationQueryInstallationDenialKind::CanonicalDigestSlotRejected => WorthQueryInstalledApplicationSchemaDenialKind::CanonicalDigestSlotRejected,
                _ => WorthQueryInstalledApplicationSchemaDenialKind::QueryInstallationDenied,
            };
            (kind, denial.subject().to_string())
        }
        ApplicationSchemaCompilationDenial::Operation(denial) => (
            WorthQueryInstalledApplicationSchemaDenialKind::OperationInstallationDenied,
            denial.operation().to_string(),
        ),
        ApplicationSchemaCompilationDenial::ValueBinding(denial) => {
            let kind = match denial.kind() {
                ValueBindingDenialKind::MissingBinding => {
                    WorthQueryInstalledApplicationSchemaDenialKind::FieldBindingMissing
                }
                ValueBindingDenialKind::UnexpectedBinding => {
                    WorthQueryInstalledApplicationSchemaDenialKind::FieldBindingUnexpected
                }
                ValueBindingDenialKind::ContractMismatch => {
                    WorthQueryInstalledApplicationSchemaDenialKind::FieldBindingContractMismatch
                }
                ValueBindingDenialKind::NativeContractMissing => {
                    WorthQueryInstalledApplicationSchemaDenialKind::FieldBindingNativeContractMissing
                }
            };
            (kind, denial.subject().to_string())
        }
    };
    WorthQueryInstalledApplicationSchemaDenial::new(kind, subject)
}

pub(super) fn map_index_denial_to_schema_denial(
    denial: WorthQueryInstalledPackageIndexDenial,
) -> WorthQueryInstalledApplicationSchemaDenial {
    let kind = match denial.kind() {
        WorthQueryInstalledPackageIndexDenialKind::DomainNotInstalled => {
            WorthQueryInstalledApplicationSchemaDenialKind::DomainNotInstalled
        }
        WorthQueryInstalledPackageIndexDenialKind::ForeignRuntime => {
            WorthQueryInstalledApplicationSchemaDenialKind::ForeignRuntime
        }
        WorthQueryInstalledPackageIndexDenialKind::StaleGeneration => {
            WorthQueryInstalledApplicationSchemaDenialKind::StaleGeneration
        }
        WorthQueryInstalledPackageIndexDenialKind::PackageIdentityChanged => {
            WorthQueryInstalledApplicationSchemaDenialKind::PackageIdentityChanged
        }
        WorthQueryInstalledPackageIndexDenialKind::AdmissionIdentityChanged => {
            WorthQueryInstalledApplicationSchemaDenialKind::AdmissionIdentityChanged
        }
        WorthQueryInstalledPackageIndexDenialKind::AuthorityMismatch => {
            WorthQueryInstalledApplicationSchemaDenialKind::AuthorityMismatch
        }
        WorthQueryInstalledPackageIndexDenialKind::AuthorityEntropyUnavailable
        | WorthQueryInstalledPackageIndexDenialKind::ConflictingPackage
        | WorthQueryInstalledPackageIndexDenialKind::ConflictingAdmissionProfile
        | WorthQueryInstalledPackageIndexDenialKind::ConflictingDefinition
        | WorthQueryInstalledPackageIndexDenialKind::OperationNotInstalled
        | WorthQueryInstalledPackageIndexDenialKind::OperationSemanticsChanged
        | WorthQueryInstalledPackageIndexDenialKind::ConflictingArtifactContract
        | WorthQueryInstalledPackageIndexDenialKind::ArtifactContractNotInstalled
        | WorthQueryInstalledPackageIndexDenialKind::ArtifactContractSemanticsChanged
        | WorthQueryInstalledPackageIndexDenialKind::ConflictingApplicationSchema
        | WorthQueryInstalledPackageIndexDenialKind::ApplicationSchemaDuplicateAspectIdentity
        | WorthQueryInstalledPackageIndexDenialKind::ApplicationSchemaDuplicateAspectLocus
        | WorthQueryInstalledPackageIndexDenialKind::ApplicationSchemaDuplicateFieldLocus
        | WorthQueryInstalledPackageIndexDenialKind::ApplicationSchemaAspectRevisionZero
        | WorthQueryInstalledPackageIndexDenialKind::ApplicationSchemaMissingAspectFieldClosure
        | WorthQueryInstalledPackageIndexDenialKind::ApplicationSchemaFieldWithoutAspect
        | WorthQueryInstalledPackageIndexDenialKind::ApplicationSchemaInvalidAspectContract
        | WorthQueryInstalledPackageIndexDenialKind::ConflictingConditionalApplicationOperation
        | WorthQueryInstalledPackageIndexDenialKind::ConditionalApplicationOperationNotInstalled
        | WorthQueryInstalledPackageIndexDenialKind::ConditionalApplicationOperationMeaningChanged
        | WorthQueryInstalledPackageIndexDenialKind::CanonicalEntryBudgetExceeded
        | WorthQueryInstalledPackageIndexDenialKind::CanonicalEncodedByteBudgetExceeded
        | WorthQueryInstalledPackageIndexDenialKind::CanonicalDigestSlotRejected => {
            WorthQueryInstalledApplicationSchemaDenialKind::SchemaMeaningChanged
        }
    };
    WorthQueryInstalledApplicationSchemaDenial::new(kind, denial.subject())
}

pub(in crate::installed_index) fn map_catalog_denial_to_index_denial(
    denial: WorthQueryApplicationSchemaContractCatalogDenial,
) -> WorthQueryInstalledPackageIndexDenial {
    let kind = match denial.kind() {
        CatalogDenialKind::DuplicateAspectIdentity => {
            WorthQueryInstalledPackageIndexDenialKind::ApplicationSchemaDuplicateAspectIdentity
        }
        CatalogDenialKind::DuplicateAspectLocus => {
            WorthQueryInstalledPackageIndexDenialKind::ApplicationSchemaDuplicateAspectLocus
        }
        CatalogDenialKind::DuplicateFieldLocus => {
            WorthQueryInstalledPackageIndexDenialKind::ApplicationSchemaDuplicateFieldLocus
        }
        CatalogDenialKind::RevisionZero => {
            WorthQueryInstalledPackageIndexDenialKind::ApplicationSchemaAspectRevisionZero
        }
        CatalogDenialKind::MissingAspectFieldClosure => {
            WorthQueryInstalledPackageIndexDenialKind::ApplicationSchemaMissingAspectFieldClosure
        }
        CatalogDenialKind::FieldWithoutAspect => {
            WorthQueryInstalledPackageIndexDenialKind::ApplicationSchemaFieldWithoutAspect
        }
        CatalogDenialKind::CanonicalEntryBudgetExceeded => {
            WorthQueryInstalledPackageIndexDenialKind::CanonicalEntryBudgetExceeded
        }
        CatalogDenialKind::CanonicalEncodedByteBudgetExceeded => {
            WorthQueryInstalledPackageIndexDenialKind::CanonicalEncodedByteBudgetExceeded
        }
        CatalogDenialKind::InvalidAspectKey
        | CatalogDenialKind::InvalidFieldKey
        | CatalogDenialKind::InvalidAspectShape
        | CatalogDenialKind::ProjectionMaskRejected
        | CatalogDenialKind::CanonicalContractRejected => {
            WorthQueryInstalledPackageIndexDenialKind::ApplicationSchemaInvalidAspectContract
        }
    };
    WorthQueryInstalledPackageIndexDenial::new(kind, denial.subject())
}

pub(in crate::installed_index) fn map_schema_digest_denial_to_index_denial(
    schema: &str,
    denial: worth_foundational::facade::CanonicalDigestDerivationDenial,
) -> WorthQueryInstalledPackageIndexDenial {
    let kind = match denial {
        worth_foundational::facade::CanonicalDigestDerivationDenial::EntryLimitExceeded {
            ..
        } => WorthQueryInstalledPackageIndexDenialKind::CanonicalEntryBudgetExceeded,
        worth_foundational::facade::CanonicalDigestDerivationDenial::EncodedByteLimitExceeded {
            ..
        } => WorthQueryInstalledPackageIndexDenialKind::CanonicalEncodedByteBudgetExceeded,
        worth_foundational::facade::CanonicalDigestDerivationDenial::UnsupportedAlgorithm
        | worth_foundational::facade::CanonicalDigestDerivationDenial::RuleVersionMismatch
        | worth_foundational::facade::CanonicalDigestDerivationDenial::InputDomainMismatch
        | worth_foundational::facade::CanonicalDigestDerivationDenial::InputShapeMismatch => {
            WorthQueryInstalledPackageIndexDenialKind::CanonicalDigestSlotRejected
        }
    };
    WorthQueryInstalledPackageIndexDenial::new(kind, schema)
}

#[cfg(test)]
mod tests {
    use crate::application_schema::{
        WorthQueryApplicationSchemaContractCatalogDenial,
        WorthQueryApplicationSchemaContractCatalogDenialKind as CatalogKind,
    };
    use crate::installed_index::WorthQueryInstalledPackageIndexDenialKind as IndexKind;

    use super::map_catalog_denial_to_index_denial;

    #[test]
    fn canonical_catalog_budget_denials_keep_their_exact_public_kind() {
        for (catalog, expected) in [
            (
                CatalogKind::CanonicalEntryBudgetExceeded,
                IndexKind::CanonicalEntryBudgetExceeded,
            ),
            (
                CatalogKind::CanonicalEncodedByteBudgetExceeded,
                IndexKind::CanonicalEncodedByteBudgetExceeded,
            ),
        ] {
            let denial = WorthQueryApplicationSchemaContractCatalogDenial::new(catalog, "budget");
            assert_eq!(map_catalog_denial_to_index_denial(denial).kind(), expected);
        }
    }
}
