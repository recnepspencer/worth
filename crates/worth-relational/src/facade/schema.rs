//! Schema meaning and registry binding export boundary.
pub use crate::derived_relation::{
    DerivedRelationCardinality, DerivedRelationDefinitionParts, DerivedRelationDenial,
    DerivedRelationGraphLaws, DerivedRelationHandle, DerivedRelationIdentity, DerivedRelationLaw,
    DerivedRelationLawViolation, DerivedRelationMinimumReachability, DerivedRelationNode,
};
pub use crate::graph_member::{NativeMemberDomain, NativeMemberKey, ProjectedMemberCoordinate};
pub use crate::projected_member_domain::{
    ObservedProjectedMember, ProjectedMemberCaseHandle, ProjectedMemberDomainDenial,
    ProjectedMemberDomainHandle, ProjectedMemberDomainIdentity, ProjectedMemberDomainParts,
};
pub use crate::schema::aggregate_identity::{
    register_aggregate_identity_storage, register_aggregate_membership,
};
pub use crate::schema::data::{
    AcyclicityContractDeclaration, AggregateIdentityContractDeclaration, AggregateIdentitySource,
    AggregateIdentityStorage, AggregateProjectedMemberStorage, AllowedCycleClass, AspectBinding,
    AspectContractPlanRevision, AspectDeclarationTrace, AspectDeclarationTraceRow,
    AspectLoweringTrace, AspectLoweringTraceRow, CardinalityContractDeclaration,
    ConnectivityMinimumContractDeclaration, ConnectivityMinimumEnforcement, ContractId,
    DeclaredAspectContractBinding, DescriptorCanonicalBasisSupportPolicy,
    DescriptorCanonicalBasisVersion, DescriptorSemanticsSupportPolicy, DescriptorSemanticsVersion,
    DirectedTraversalKind, EndpointDeletionIntegrityDeclaration, EndpointDeletionIntegrityMode,
    EndpointKindContractDeclaration, EntityKindRegistration, FreeFormSchemaDiffIntent,
    HistoricalInterpretationSensitivity, KindAspectContractDeclarations, KindResolution,
    LoweredAcyclicityContract, LoweredAspectContractBinding, LoweredAspectContractPlan,
    LoweredCardinalityMaximumContract, LoweredCardinalityMinimumContract,
    LoweredConnectivityMinimumContract, LoweredEndpointDeletionIntegrityContract,
    LoweredEndpointKindContract, LoweredPartitionIsolationContract, LoweredRelationIntegrityPlan,
    LoweredSchemaTransitionPlan, LoweredSymmetryContract, LoweredUniquenessContract,
    MinimumCardinalityEnforcement, PairMinimumSemantics, PartitionIsolationContractDeclaration,
    PartitionIsolationMode, ProposedSchemaTransition, RelationIntegrityDeclarations,
    RelationIntegrityPlanCatalog, RelationIntegrityPlanRevision, RelationKindRegistration,
    RelationalAspectChangeKind, RelationalSchemaRegistry, SchemaAuthoritySnapshot,
    SchemaBoundaryFingerprint, SchemaBridgeDescriptor, SchemaBridgeabilityClassification,
    SchemaContinuationAdmissionObservation, SchemaContinuationClassification,
    SchemaContinuationDescriptor, SchemaDiffAtom, SchemaDiffDetail, SchemaElementKind,
    SchemaElementRef, SchemaId, SchemaLineageArtifact, SchemaLineageOrderingSemantics,
    SchemaPublicationImpact, SchemaReconciliationClassification, SchemaReconciliationDescriptor,
    SchemaReconciliationOrderingMode, SchemaReconciliationPolicy, SchemaRegistryError,
    SchemaRegistryErrorClass, SchemaStratum, SchemaSubscriberImpact, SchemaTransitionArtifact,
    SchemaTransitionBarrier, SchemaTransitionSummary, SchemaVersionId,
    SubscriberBoundaryVisibility, SymmetryContractDeclaration, SymmetryMode,
    UniquenessContractDeclaration, UniquenessScope, ValidatedSchemaTransition,
};
