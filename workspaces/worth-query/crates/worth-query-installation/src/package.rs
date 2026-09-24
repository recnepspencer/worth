mod application_schema_validation;
mod artifact_closure;
mod conditional_operation_validation;
mod definition;
mod identity;
mod member_validation;
mod portable_records;
mod reconstruction;
mod validation_denial;

pub(crate) use reconstruction::work_observation::text as reconstruction_text_bytes;

use worth_proof::{
    Artifact, AuthorityMarker, AuthorityProves, AuthorityWitness, CurrentValidity,
    FreshnessScopedBasis, PhaseMarker, Proof, ProofMarker,
};

pub use definition::{WorthQueryPortableDefinition, WorthQueryPortableDefinitionKind};
pub use identity::{WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackageIdentity};
pub use portable_records::{
    WorthQueryPortableApplicationContractSpine,
    WorthQueryPortableApplicationOperationContractParts,
    WorthQueryPortableApplicationOperationContractRecord, WorthQueryPortableDomainOperationParts,
    WorthQueryPortableDomainOperationRecord, WorthQueryPortableDomainOperationSemanticParts,
    WorthQueryPortableDomainOperationSemanticRecord, WorthQueryPortableExternalEffectContractParts,
    WorthQueryPortableExternalEffectContractRecord,
    WorthQueryPortableInstalledReconciliationProcedureRecord,
    WorthQueryPortableNativeAspectContractParts, WorthQueryPortableNativeAspectContractRecord,
    WorthQueryPortableOperationGraphReadScope, WorthQueryPortableOperationTouchScope,
    WorthQueryPortablePackageExportDenial, WorthQueryPortablePackageExportDenialKind,
    WorthQueryPortablePackageExportLimits, WorthQueryPortablePackageManifest,
    WorthQueryPortablePackageManifestVersion, WorthQueryPortablePackageRecord,
    WorthQueryPortablePackageRecordFamily, WorthQueryPortablePackageRecordSet,
    WorthQueryPortablePackageRecordView, WORTH_QUERY_PORTABLE_PACKAGE_MANIFEST_VERSION,
    WORTH_QUERY_PORTABLE_PACKAGE_MAX_LOGICAL_EXPORT_BYTES,
    WORTH_QUERY_PORTABLE_PACKAGE_MAX_RECORDS,
};
pub use reconstruction::{
    WorthQueryExpectedPortablePackageIdentity, WorthQueryPortablePackageReconstruction,
    WorthQueryPortablePackageReconstructionCandidate,
    WorthQueryPortablePackageReconstructionDenial, WorthQueryPortablePackageReconstructionLimits,
    WorthQueryPortablePackageReconstructionWork, WorthQueryReconstructedPortablePackageCandidate,
    WORTH_QUERY_PORTABLE_PACKAGE_MAX_RECONSTRUCTION_CANONICAL_BYTES,
    WORTH_QUERY_PORTABLE_PACKAGE_MAX_RECONSTRUCTION_ENTRIES,
};
pub use validation_denial::{
    WorthQueryPortablePackageValidationDenial, WorthQueryPortablePackageValidationDenialKind,
};

use crate::application_operation::{
    WorthQueryApplicationConditionalOperationBinding,
    WorthQueryPortableApplicationConditionalOperationBinding,
};
use crate::canonical_work::WorthQueryCanonicalWorkEvidence;
use crate::domain_computation::WorthQueryPortableArtifactContract;
use crate::domain_operation::{
    WorthQueryPortableDomainOperationDefinition, WorthQueryValidatedDomainOperation,
};
use crate::package::portable_records::compile_application_contract_spine;
#[cfg(test)]
pub(crate) use crate::package::portable_records::verify_source_closure_for_test;
use crate::package::{
    identity::canonical_identity_with_maximum_bytes, member_validation::validate_package_members,
};
use crate::package_requirements::{
    WorthQueryInstallationCapabilityFamily, WorthQueryInstallationConfigSectionFamily,
    WorthQueryInstallationContributionCategory, WorthQueryInstallationOperatingRequirement,
};
use worth_foundational::facade::CanonicalDigestDerivationDenial;
use worth_query_declaration::facade::application_schema::{
    ApplicationSchemaDeclaration, ErasedApplicationSchemaDeclaration,
};

#[derive(Clone, Debug)]
pub struct WorthQueryPortableDomainPackage {
    identity: WorthQueryPortableDomainIdentity,
    capabilities: Vec<WorthQueryInstallationCapabilityFamily>,
    configuration: Vec<WorthQueryInstallationConfigSectionFamily>,
    operating: Vec<WorthQueryInstallationOperatingRequirement>,
    definitions: Vec<WorthQueryPortableDefinition>,
    domain_operations: Vec<WorthQueryPortableDomainOperationDefinition>,
    artifact_contracts: Vec<WorthQueryPortableArtifactContract>,
    application_schemas: Vec<ErasedApplicationSchemaDeclaration>,
    conditional_application_operations:
        Vec<WorthQueryPortableApplicationConditionalOperationBinding>,
    contributions: Vec<WorthQueryInstallationContributionCategory>,
}

impl WorthQueryPortableDomainPackage {
    pub fn new(identity: WorthQueryPortableDomainIdentity) -> Self {
        Self {
            identity,
            capabilities: Vec::new(),
            configuration: Vec::new(),
            operating: Vec::new(),
            definitions: Vec::new(),
            domain_operations: Vec::new(),
            artifact_contracts: Vec::new(),
            application_schemas: Vec::new(),
            conditional_application_operations: Vec::new(),
            contributions: Vec::new(),
        }
    }

    pub fn requires_capability(mut self, value: impl Into<String>) -> Self {
        self.capabilities
            .push(WorthQueryInstallationCapabilityFamily::new(value));
        self
    }

    pub fn requires_configuration(mut self, value: impl Into<String>) -> Self {
        self.configuration
            .push(WorthQueryInstallationConfigSectionFamily::new(value));
        self
    }

    pub fn requires_operating_posture(mut self, value: impl Into<String>) -> Self {
        self.operating
            .push(WorthQueryInstallationOperatingRequirement::new(value));
        self
    }

    pub fn definition(mut self, definition: WorthQueryPortableDefinition) -> Self {
        self.definitions.push(definition);
        self
    }

    pub fn domain_operation(
        mut self,
        definition: WorthQueryPortableDomainOperationDefinition,
    ) -> Self {
        self.domain_operations.push(definition);
        self
    }

    pub fn artifact_contract(mut self, contract: WorthQueryPortableArtifactContract) -> Self {
        self.artifact_contracts.push(contract);
        self
    }

    pub fn application_schema<Schema>(
        mut self,
        declaration: ApplicationSchemaDeclaration<Schema>,
    ) -> Self {
        self.application_schemas.push(declaration.into_erased());
        self
    }

    pub fn conditional_application_operation<Schema, ApplicationOperation, Input, D, O, F>(
        mut self,
        binding: WorthQueryApplicationConditionalOperationBinding<
            Schema,
            ApplicationOperation,
            Input,
            D,
            O,
            F,
        >,
    ) -> Self {
        self.conditional_application_operations
            .push(binding.into_portable());
        self
    }

    #[doc(hidden)]
    pub fn conditional_application_operation_erased(
        mut self,
        binding: WorthQueryPortableApplicationConditionalOperationBinding,
    ) -> Self {
        self.conditional_application_operations.push(binding);
        self
    }

    #[doc(hidden)]
    pub fn application_schema_erased(
        mut self,
        declaration: ErasedApplicationSchemaDeclaration,
    ) -> Self {
        self.application_schemas.push(declaration);
        self
    }

    pub fn permits_contribution(mut self, value: impl Into<String>) -> Self {
        self.contributions
            .push(WorthQueryInstallationContributionCategory::new(value));
        self
    }

    pub fn validate(
        self,
    ) -> Result<WorthQueryValidatedPortableDomainPackage, WorthQueryPortablePackageValidationDenial>
    {
        self.validate_with_canonical_work_limit(u64::MAX)
    }

    pub(crate) fn validate_with_canonical_work_limit(
        mut self,
        maximum_canonical_work_bytes: u64,
    ) -> Result<WorthQueryValidatedPortableDomainPackage, WorthQueryPortablePackageValidationDenial>
    {
        // WORTH-UI-TEMPORARY-INSTRUMENTATION
        let trace = std::env::var_os("WORTH_REOPEN_TRACE").is_some();
        let started = std::time::Instant::now();
        validate_package_members(&mut self)?;
        if trace {
            eprintln!(
                "[WORTH_REOPEN_TRACE] package members: {:?}",
                started.elapsed()
            );
        }
        let validated_domain_operations = admit_domain_operations(&self.domain_operations)?;
        if trace {
            eprintln!(
                "[WORTH_REOPEN_TRACE] package operations: {:?}",
                started.elapsed()
            );
        }
        let application_contract_spine =
            compile_application_contract_spine(&self.application_schemas)?;
        if trace {
            eprintln!(
                "[WORTH_REOPEN_TRACE] package contract spine: {:?}",
                started.elapsed()
            );
        }
        let maximum_canonical_work_bytes =
            usize::try_from(maximum_canonical_work_bytes).unwrap_or(usize::MAX);
        let (identity, canonical_work) =
            canonical_identity_with_maximum_bytes(&self, maximum_canonical_work_bytes)
                .map_err(map_package_canonical_denial)?;
        if trace {
            eprintln!(
                "[WORTH_REOPEN_TRACE] package canonical identity: {:?}",
                started.elapsed()
            );
        }
        let authority =
            AuthorityWitness::from_authority_marker(PortablePackageValidationAuthority {
                _private: (),
            });
        let artifact = Artifact::with_proofs_and_current_basis(
            self,
            Proof::from_authority_witness(&authority),
            identity.clone(),
            authority,
        );
        Ok(WorthQueryValidatedPortableDomainPackage {
            artifact,
            identity,
            canonical_work,
            validated_domain_operations,
            application_contract_spine,
        })
    }
}

fn admit_domain_operations(
    operations: &[WorthQueryPortableDomainOperationDefinition],
) -> Result<Vec<WorthQueryValidatedDomainOperation>, WorthQueryPortablePackageValidationDenial> {
    let mut validated = Vec::with_capacity(operations.len());
    for operation in operations.iter().cloned() {
        let slot = operation.identity().slot();
        let operation = WorthQueryValidatedDomainOperation::admit(operation).map_err(|reason| {
            WorthQueryPortablePackageValidationDenial::invalid_domain_operation(format!(
                "{slot}:{reason}"
            ))
        })?;
        validated.push(operation);
    }
    Ok(validated)
}

#[derive(Clone, Debug)]
pub struct WorthQueryValidatedPortableDomainPackage {
    artifact: ValidatedPortablePackageArtifact,
    identity: WorthQueryPortableDomainPackageIdentity,
    canonical_work: WorthQueryCanonicalWorkEvidence,
    validated_domain_operations: Vec<WorthQueryValidatedDomainOperation>,
    application_contract_spine: WorthQueryPortableApplicationContractSpine,
}

#[derive(Clone, Debug)]
struct PortablePackageValidated;
impl PhaseMarker for PortablePackageValidated {}

#[derive(Clone, Debug)]
struct PortablePackageMeaningValidated;
impl ProofMarker for PortablePackageMeaningValidated {}

#[derive(Clone, Debug)]
struct PortablePackageValidationAuthority {
    _private: (),
}
impl AuthorityMarker for PortablePackageValidationAuthority {}
impl AuthorityProves<PortablePackageMeaningValidated> for PortablePackageValidationAuthority {}

type ValidatedPortablePackageArtifact = Artifact<
    PortablePackageValidated,
    WorthQueryPortableDomainPackage,
    Proof<PortablePackageMeaningValidated, PortablePackageValidationAuthority>,
    FreshnessScopedBasis<
        CurrentValidity,
        worth_proof::AssumptionBasis<WorthQueryPortableDomainPackageIdentity>,
    >,
>;

impl WorthQueryValidatedPortableDomainPackage {
    pub(crate) fn has_same_authoritative_meaning(&self, other: &Self) -> bool {
        self.domain_identity() == other.domain_identity()
            && self.capabilities() == other.capabilities()
            && self.configuration() == other.configuration()
            && self.operating_requirements() == other.operating_requirements()
            && self.definitions() == other.definitions()
            && self.domain_operations() == other.domain_operations()
            && self.artifact_contracts() == other.artifact_contracts()
            && self.application_schemas() == other.application_schemas()
            && self.conditional_application_operations()
                == other.conditional_application_operations()
            && self.contribution_policy() == other.contribution_policy()
    }

    pub fn domain_identity(&self) -> &WorthQueryPortableDomainIdentity {
        &self.artifact.payload().identity
    }

    pub fn identity(&self) -> &WorthQueryPortableDomainPackageIdentity {
        &self.identity
    }

    pub const fn canonical_work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.canonical_work
    }

    pub fn capabilities(&self) -> &[WorthQueryInstallationCapabilityFamily] {
        &self.artifact.payload().capabilities
    }

    pub fn configuration(&self) -> &[WorthQueryInstallationConfigSectionFamily] {
        &self.artifact.payload().configuration
    }

    pub fn operating_requirements(&self) -> &[WorthQueryInstallationOperatingRequirement] {
        &self.artifact.payload().operating
    }

    pub fn definitions(&self) -> &[WorthQueryPortableDefinition] {
        &self.artifact.payload().definitions
    }

    pub fn domain_operations(&self) -> &[WorthQueryPortableDomainOperationDefinition] {
        &self.artifact.payload().domain_operations
    }

    pub fn artifact_contracts(&self) -> &[WorthQueryPortableArtifactContract] {
        &self.artifact.payload().artifact_contracts
    }

    pub fn application_schemas(&self) -> &[ErasedApplicationSchemaDeclaration] {
        &self.artifact.payload().application_schemas
    }

    pub fn conditional_application_operations(
        &self,
    ) -> &[WorthQueryPortableApplicationConditionalOperationBinding] {
        &self.artifact.payload().conditional_application_operations
    }

    pub(crate) fn validated_domain_operations(&self) -> &[WorthQueryValidatedDomainOperation] {
        &self.validated_domain_operations
    }

    /// Exact typed application contracts retained when this package was validated.
    ///
    /// The spine is descriptive and carries no runtime binding or installation authority.
    pub fn application_contract_spine(&self) -> &WorthQueryPortableApplicationContractSpine {
        &self.application_contract_spine
    }

    pub fn contribution_policy(&self) -> &[WorthQueryInstallationContributionCategory] {
        &self.artifact.payload().contributions
    }

    /// Export complete authority-free logical meaning under the default cold-path limits.
    pub fn export_typed_records(
        &self,
    ) -> Result<WorthQueryPortablePackageRecordSet, WorthQueryPortablePackageExportDenial> {
        self.export_typed_records_with_limits(WorthQueryPortablePackageExportLimits::DEFAULT)
    }

    /// Export complete authority-free logical meaning under caller-narrowed limits.
    pub fn export_typed_records_with_limits(
        &self,
        limits: WorthQueryPortablePackageExportLimits,
    ) -> Result<WorthQueryPortablePackageRecordSet, WorthQueryPortablePackageExportDenial> {
        portable_records::export_validated_package_records(self, limits)
    }
}

fn map_package_canonical_denial(
    denial: CanonicalDigestDerivationDenial,
) -> WorthQueryPortablePackageValidationDenial {
    match denial {
        CanonicalDigestDerivationDenial::EntryLimitExceeded { .. } => {
            WorthQueryPortablePackageValidationDenial::canonical_entry_budget_exceeded()
        }
        CanonicalDigestDerivationDenial::EncodedByteLimitExceeded { maximum, attempted } => {
            WorthQueryPortablePackageValidationDenial::canonical_encoded_byte_budget_exceeded(
                maximum, attempted,
            )
        }
        _ => WorthQueryPortablePackageValidationDenial::canonical_digest_slot_rejected(),
    }
}
