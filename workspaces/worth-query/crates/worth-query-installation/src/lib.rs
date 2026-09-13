//! Portable Query installation authority.
//!
//! This package owns callback-free domain package meaning, installation
//! admission, generation affinity, conflict semantics, and disposable
//! installed indexes. Execution providers and workspaces are downstream.

#![forbid(unsafe_code)]

mod admission;
mod application_ability;
mod application_aftermath;
mod application_capability;
mod application_operation;
mod application_principal_binding;
mod application_query;
mod application_schema;
mod authority_cryptography;
mod canonical_digest_derivation;
mod canonical_hash_encoding;
mod canonical_work;
mod domain_computation;
mod domain_operation;
mod generation;
mod graph_obligation;
mod installed_domain_operation;
mod installed_graph_participation;
mod installed_handle_denial;
mod installed_index;
mod installed_operation;
mod package;
mod package_requirements;

#[cfg(test)]
mod admission_profile_tests;
#[cfg(test)]
mod application_principal_binding_tests;
#[cfg(test)]
mod application_schema_tests;
#[cfg(test)]
mod conditional_application_operation_test_fixture;
#[cfg(test)]
mod domain_computation_admission_tests;
#[cfg(test)]
mod domain_computation_artifact_fixture;
#[cfg(test)]
mod domain_computation_authority_tests;
#[cfg(test)]
mod domain_computation_canonical_identity_tests;
#[cfg(test)]
mod domain_computation_conflict_tests;
#[cfg(test)]
mod domain_computation_evidence_conflict_tests;
#[cfg(test)]
mod domain_computation_evidence_schema_tests;
#[cfg(test)]
mod domain_computation_evolution_tests;
#[cfg(test)]
mod domain_computation_occurrence_tests;
#[cfg(test)]
mod domain_computation_reproducibility_tests;
#[cfg(test)]
mod domain_computation_search_tests;
#[cfg(test)]
mod domain_computation_transformation_tests;
#[cfg(test)]
mod domain_computation_validation_tests;
#[cfg(test)]
mod domain_computation_workflow_closure_tests;
#[cfg(test)]
mod domain_computation_workflow_test_support;
#[cfg(test)]
mod package_validation_tests;

pub mod facade {
    pub use worth_foundational::facade::{
        AbsenceLaw, AspectBinding, AspectContract, AspectContractRevision, AspectEvolutionPolicy,
        AspectIdentity, AspectKey, AspectMask, AspectValue, AuthoritativeAspectChangeKind,
        CanonicalFieldPath, ContractValidatedAspectValueView, FieldDeclaration, FieldKey,
        FieldRequirement, InternedString, ProjectionMask, ScalarAspectType, StructAspectShape,
    };
    pub use worth_query_declaration::facade::application_capability::{
        ApplicationCapabilityRef, ErasedApplicationCapabilityContract,
    };
    pub use worth_query_declaration::facade::application_schema::{
        ApplicationAuthorizationPath, ApplicationAuthorizationPathEffect,
        ApplicationAuthorizationPredicate, ApplicationAuthorizationTraversal,
        ApplicationAuthorizationTraversalDirection, ApplicationEntityRef, ApplicationFieldPresence,
        ApplicationFieldRef, ApplicationFieldUnit, ApplicationIdentityScalarValueBinding,
        ApplicationOperationDecisionReadTarget, ApplicationOperationProgramTarget,
        ApplicationReadableScalarValueBinding, ApplicationRelationCardinality,
        ApplicationRelationCrossContextPolicy, ApplicationRelationDeletionPolicy,
        ApplicationRelationEndpoints, ApplicationRelationIntegrity, ApplicationRelationRef,
        ApplicationScalarValueBinding, ApplicationSchema, ApplicationSchemaBindingIdentity,
        ApplicationSchemaMember, ApplicationSignedAggregateValueBinding,
        ApplicationStructuredValueBinding, ApplicationValueDecodeDenial,
        ApplicationValueEncodeDenial, ApplicationValueValidationDenial,
        DeclaredApplicationFieldValue, EqualityPosture, EqualityPredicate,
        ErasedApplicationSchemaDeclaration, OperationCreates, OperationDeletes, OperationLinks,
        OperationReads, OperationUnlinks, OperationWrites, OptionalApplicationFieldValue,
        RequiredApplicationFieldValue, WorthQueryExternalEffectCorrelationFamily,
        WritableCapability, WritePosture,
    };
    pub use worth_query_declaration::facade::authentication::{
        WorthQueryExternalPrincipalIdentity, WorthQueryPrincipalMappingStatus,
    };

    pub use crate::admission::{
        WorthQueryAdmittedPortableDomainPackage, WorthQueryArtifactVersionSupport,
        WorthQueryInstallationAdmissionDenial, WorthQueryInstallationAdmissionDenialKind,
        WorthQueryInstallationAdmissionIdentity, WorthQueryInstallationAdmissionProfile,
        WorthQueryInstallationSupportStatus,
    };
    pub use crate::application_ability::{
        WorthQueryAbilityInstallationDenial, WorthQueryAbilityInstallationDenialKind,
        WorthQueryInstalledAbility,
    };
    pub use crate::application_aftermath::{
        aftermath_owner_identity_digest, derive_published_posture,
        AftermathLoweringCorrespondenceCatalog, CompensatableNextActionContract,
        CompensateNextAction, InstalledAftermathNextActionContract,
        InstalledAftermathPostcondition, InstalledAftermathRecoveryContract, InstalledCompensation,
        InstalledCorrectionAuthority, InstalledCorrectionMechanism,
        InstalledExternalEffectContract, InstalledExternalEffectPosture,
        InstalledLoweringCorrespondence, InstalledLoweringCorrespondenceRef,
        InstalledPreImageDemand, InstalledPreImageLocus, InstalledRecordedInverse,
        IrreversibleNextActionContract, PublishedAftermathPosture, ReconcilableNextActionContract,
        ReconcileNextAction, ReversibleNextActionContract, UndoViaRecordedInverse,
        WorthQueryAftermathCanonicalArtifact, WorthQueryAftermathInstallationDenial,
        WorthQueryAftermathInstallationDenialKind, WorthQueryInstalledAftermathContract,
        WorthQueryInstalledAftermathIdentity, WorthQueryInstalledReconciliationProcedure,
    };
    pub use crate::application_capability::{
        derive_capability_revocation_proposal_identity, derive_delegation_proposal_identity,
        WorthQueryApplicationCapabilityInstallationDenial,
        WorthQueryApplicationCapabilityInstallationDenialKind,
        WorthQueryCapabilityCanonicalArtifact, WorthQueryCapabilityLookupEvidence,
        WorthQueryCapabilityRevocationProposalBasis, WorthQueryCapabilityRevocationProposalDenial,
        WorthQueryDelegationProposalIdentityBasis, WorthQueryDelegationProposalIdentityDenial,
        WorthQueryInstalledApplicationCapability, WorthQueryInstalledApplicationCapabilityIdentity,
        WorthQueryInstalledApplicationCapabilityPlanSource,
    };
    pub use crate::application_operation::{
        WorthQueryApplicationConditionalOperationBinding,
        WorthQueryApplicationOperationInstallationDenial,
        WorthQueryApplicationOperationInstallationDenialKind,
        WorthQueryCompiledApplicationOperationContracts,
        WorthQueryConditionalApplicationOperationDenial,
        WorthQueryConditionalApplicationOperationDenialKind, WorthQueryInstalledAbilityRequirement,
        WorthQueryInstalledApplicationConditionalNode,
        WorthQueryInstalledApplicationConditionalOperation,
        WorthQueryInstalledApplicationEffectEmission,
        WorthQueryInstalledApplicationMutationBinding, WorthQueryInstalledApplicationOperation,
        WorthQueryInstalledApplicationOperationAuthorization,
        WorthQueryInstalledApplicationOperationExecutionPosture,
        WorthQueryInstalledApplicationOperationGraphAuthority,
        WorthQueryInstalledAuthorizationPath, WorthQueryInstalledBoundMutationOperation,
        WorthQueryInstalledBoundMutationPrincipal, WorthQueryInstalledHostConditionalProvider,
        WorthQueryInstalledMutationPrecondition, WorthQueryInstalledNamedClockConditionalNode,
        WorthQueryInstalledTemporalConditionalOperation, WorthQueryOperationEmissionContract,
        WorthQueryPortableApplicationConditionalOperationBinding,
        WorthQueryPortableApplicationConditionalOperationBindingParts,
        APPLICATION_AUTHORIZATION_FACT_FAMILY, APPLICATION_DECISION_FACT_FAMILY,
        APPLICATION_EXECUTION_ACCESS_PRODUCT_FAMILY, APPLICATION_EXECUTION_ALLOCATOR_FAMILY,
        APPLICATION_EXECUTION_PROVIDER_FAMILY, APPLICATION_EXECUTION_SAFE_POINT_FAMILY,
        APPLICATION_INVARIANT_SLOT,
    };
    pub use crate::application_principal_binding::{
        WorthQueryInstalledPrincipalBinding, WorthQueryPrincipalBindingInstallationDenial,
        WorthQueryPrincipalBindingInstallationDenialKind,
    };
    pub use crate::application_query::{
        prepare_canonical_read_graph_planning_basis, WorthQueryApplicationCanonicalArtifact,
        WorthQueryApplicationQueryCanonicalWorkPolicy,
        WorthQueryApplicationQueryInstallationDenial,
        WorthQueryApplicationQueryInstallationDenialKind, WorthQueryApplicationQueryLimitDenial,
        WorthQueryInstalledApplicationContinuationContract,
        WorthQueryInstalledApplicationLiveContract, WorthQueryInstalledApplicationQuery,
        WorthQueryInstalledApplicationQueryAuthorization,
        WorthQueryInstalledApplicationQueryBinding, WorthQueryInstalledApplicationQueryIdentity,
        WorthQueryInstalledApplicationQueryLimits, WorthQueryInstalledApplicationReadFamilyBinding,
        WorthQueryInstalledBoundPrincipal, WorthQueryInstalledBoundQuery,
        WorthQueryInstalledGraphOrdering, WorthQueryInstalledGraphPredicate,
        WorthQueryInstalledGraphProjection, WorthQueryInstalledGraphReadContract,
        WorthQueryInstalledGraphRelation, WorthQueryInstalledRootPath,
        WorthQueryInstalledRootPathGuard, WorthQueryInstalledRootPathStep,
        WorthQueryPreparedReadGraphPlanningContract, WorthQueryReadGraphGuardView,
        WorthQueryReadGraphOrderingMechanism, WorthQueryReadGraphOrderingView,
        WorthQueryReadGraphPlanningContract, WorthQueryReadGraphPredicateView,
        WorthQueryReadGraphProjectionView, WorthQueryReadGraphRelationDirection,
        WorthQueryReadGraphRelationView,
    };
    pub use crate::application_schema::{
        WorthQueryInstalledApplicationAspectContract, WorthQueryInstalledApplicationAspectLocus,
        WorthQueryInstalledApplicationContribution,
        WorthQueryInstalledApplicationContributionCatalog, WorthQueryInstalledApplicationInvariant,
        WorthQueryInstalledApplicationInvariantCatalog,
        WorthQueryInstalledApplicationInvariantDescriptor, WorthQueryInstalledApplicationSchema,
        WorthQueryInstalledApplicationSchemaContractCatalog,
        WorthQueryInstalledApplicationSchemaContractCatalogCounters,
        WorthQueryInstalledApplicationSchemaDenial, WorthQueryInstalledApplicationSchemaDenialKind,
        WorthQueryInstalledApplicationValueBinding,
        WorthQueryInstalledApplicationValueBindingCatalog,
    };
    pub use crate::canonical_work::{
        WorthQueryCanonicalWorkEvidence, WorthQueryCanonicalWorkPhases,
    };
    pub use crate::domain_computation::*;
    pub use crate::domain_operation::*;
    pub use crate::generation::{
        WorthQueryInstallationGeneration, WorthQueryInstallationRuntimeIdentity,
    };
    pub use crate::graph_obligation::{
        inspect_installed_graph_obligations, WorthQueryGraphObligationAdoptionDenial,
        WorthQueryGraphObligationAdoptionDenialKind, WorthQueryGraphObligationAdoptionProof,
        WorthQueryGraphObligationAdoptionRow, WorthQueryInstalledGraphAuthorizationRequirement,
        WorthQueryInstalledGraphCapabilityRequirement, WorthQueryInstalledGraphObligation,
        WorthQueryInstalledGraphObligationEffectPosture,
        WorthQueryInstalledGraphObligationIdentity, WorthQueryInstalledGraphObligationInspection,
        WorthQueryInstalledGraphObligationInstallationEvidence,
        WorthQueryInstalledGraphObligationKind, WorthQueryInstalledGraphObligationLookup,
        WorthQueryInstalledGraphObligationOwner, WorthQueryInstalledGraphObligationResourcePosture,
        WorthQueryInstalledGraphObligationSelectionBasis, WorthQueryInstalledGraphObligationSet,
        WorthQueryInstalledGraphObligationSetIdentity,
        WorthQueryInstalledGraphObligationSubjectKind,
        WorthQueryInstalledGraphObligationTerminalRequirement,
    };
    pub use crate::installed_domain_operation::{
        WorthQueryConditionalDependencyLookupDenial,
        WorthQueryInstalledConditionalDependencyAuthority,
        WorthQueryInstalledDomainOperationAuthority,
    };
    pub use crate::installed_graph_participation::WorthQueryInstalledGraphParticipationAuthority;
    pub use crate::installed_handle_denial::{
        WorthQueryDomainHandleDenial, WorthQueryDomainHandleDenialKind,
    };
    pub use crate::installed_index::{
        WorthQueryInstalledPackageAuthority, WorthQueryInstalledPackageIndex,
        WorthQueryInstalledPackageIndexCounters, WorthQueryInstalledPackageIndexDenial,
        WorthQueryInstalledPackageIndexDenialKind, WorthQueryInstalledPackageIndexRebuildReport,
        WorthQueryInstalledPackageIndexRelation,
    };
    pub use crate::installed_operation::WorthQueryInstalledOperationAuthority;
    pub use crate::package::{
        WorthQueryExpectedPortablePackageIdentity, WorthQueryPortableApplicationContractSpine,
        WorthQueryPortableApplicationOperationContractParts,
        WorthQueryPortableApplicationOperationContractRecord, WorthQueryPortableDefinition,
        WorthQueryPortableDefinitionKind, WorthQueryPortableDomainIdentity,
        WorthQueryPortableDomainOperationParts, WorthQueryPortableDomainOperationRecord,
        WorthQueryPortableDomainOperationSemanticParts,
        WorthQueryPortableDomainOperationSemanticRecord, WorthQueryPortableDomainPackage,
        WorthQueryPortableDomainPackageIdentity, WorthQueryPortableExternalEffectContractParts,
        WorthQueryPortableExternalEffectContractRecord,
        WorthQueryPortableInstalledReconciliationProcedureRecord,
        WorthQueryPortableNativeAspectContractParts, WorthQueryPortableNativeAspectContractRecord,
        WorthQueryPortableOperationGraphReadScope, WorthQueryPortableOperationTouchScope,
        WorthQueryPortablePackageExportDenial, WorthQueryPortablePackageExportDenialKind,
        WorthQueryPortablePackageExportLimits, WorthQueryPortablePackageManifest,
        WorthQueryPortablePackageManifestVersion, WorthQueryPortablePackageReconstruction,
        WorthQueryPortablePackageReconstructionCandidate,
        WorthQueryPortablePackageReconstructionDenial,
        WorthQueryPortablePackageReconstructionLimits, WorthQueryPortablePackageReconstructionWork,
        WorthQueryPortablePackageRecord, WorthQueryPortablePackageRecordFamily,
        WorthQueryPortablePackageRecordSet, WorthQueryPortablePackageRecordView,
        WorthQueryPortablePackageValidationDenial, WorthQueryPortablePackageValidationDenialKind,
        WorthQueryReconstructedPortablePackageCandidate, WorthQueryValidatedPortableDomainPackage,
        WORTH_QUERY_PORTABLE_PACKAGE_MANIFEST_VERSION,
        WORTH_QUERY_PORTABLE_PACKAGE_MAX_LOGICAL_EXPORT_BYTES,
        WORTH_QUERY_PORTABLE_PACKAGE_MAX_RECONSTRUCTION_CANONICAL_BYTES,
        WORTH_QUERY_PORTABLE_PACKAGE_MAX_RECONSTRUCTION_ENTRIES,
        WORTH_QUERY_PORTABLE_PACKAGE_MAX_RECORDS,
    };
    pub use crate::package_requirements::{
        WorthQueryInstallationCapabilityFamily, WorthQueryInstallationConfigSectionFamily,
        WorthQueryInstallationContributionCategory, WorthQueryInstallationOperatingRequirement,
    };
}
