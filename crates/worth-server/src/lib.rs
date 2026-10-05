//! `worth-server` owns the typed server facade and network bootstrap boundary.
//!
//! Milestone 1 establishes:
//!
//! - one facade-owned bootstrap path
//! - explicit surface-family registration
//! - validated runtime assembly before serving
//! - an internal transport boundary rather than framework-shaped public API

#![forbid(unsafe_code)]

mod canonical_digest;
mod config;
mod declaration_intake;
mod diagnostics;
mod durable_product_mutation;
mod external_resource;
pub mod facade;
pub mod middleware;
mod operation_admission;
mod operation_planning;
mod operation_readiness;
mod operation_registry;
mod operation_request;
mod operation_runtime_certification;
mod operation_scheduler;
pub mod operator_evidence;
mod product_adapter;
mod product_operation_contract;
mod product_protocol_catalog;
mod product_result;
mod product_session;
mod product_session_coordination;
pub mod query_dependency_audit;
pub mod query_handoff;
mod registration;
pub mod request_context;
pub mod response;
mod runtime;
pub mod surfaces;
mod transport;
pub mod worth_native;

pub use config::{
    WorthServerBindAddress, WorthServerConfig, WorthServerConfigBuilder, WorthServerConfigError,
    WorthServerMiddlewareConfig, WorthServerMiddlewareConfigBuilder,
    WorthServerMiddlewareConfigError, WorthServerOperatorEvidenceConfig,
    WorthServerOperatorEvidenceConfigBuilder, WorthServerOperatorEvidenceConfigError,
    WorthServerQueryHandoffConfig, WorthServerQueryHandoffConfigBuilder,
    WorthServerQueryHandoffConfigError, WorthServerRequestContextConfig,
    WorthServerRequestContextConfigBuilder, WorthServerRequestContextConfigError,
    WorthServerResponseConfig, WorthServerResponseConfigBuilder, WorthServerResponseConfigError,
};
pub use declaration_intake::{
    WorthServerAdmittedDirectDeclaration, WorthServerDirectDeclaration,
    WorthServerDirectDeclarationBuilder, WorthServerDirectDeclarationDenial,
    WorthServerDirectDeclarationDenialCode, WorthServerDirectDeclarationError,
    WorthServerDirectDeclarationSource, WorthServerDirectDeclarationSourceKind,
    WorthServerDirectDeclarationSourceSupportStatus, WorthServerDirectSupportSnapshot,
    WorthServerDirectViewShape, WorthServerPreparedDirectDeclaration,
};
pub use diagnostics::WorthServerCounterSnapshot;
pub use durable_product_mutation::{
    WorthServerAdmittedDurableProductMutation, WorthServerDurableProductMutationCompletion,
    WorthServerDurableProductMutationConclusion, WorthServerDurableProductMutationContract,
    WorthServerDurableProductMutationDisposition, WorthServerDurableProductMutationExecution,
    WorthServerDurableProductMutationExecutor, WorthServerDurableProductMutationReceipt,
    WorthServerDurableProductMutationRecoveryHandle, WorthServerProductAuthorityScope,
    WorthServerProductDurabilityCapability, WorthServerProductIdempotencyRetention,
};
pub use external_resource::{
    WorthServerAdmittedExternalResourceResult, WorthServerCompletedExternalResourceExecution,
    WorthServerExternalResourceBudget, WorthServerExternalResourceExecutionBoundary,
    WorthServerExternalResourceExecutionCounters, WorthServerExternalResourceExecutionDenial,
    WorthServerExternalResourceExecutionDenialCode, WorthServerExternalResourceExecutionFailure,
    WorthServerExternalResourceExecutionOutcome, WorthServerExternalResourceIntent,
    WorthServerExternalResourceIntentBuilder, WorthServerExternalResourceIntentError,
    WorthServerExternalResourcePlanDenial, WorthServerExternalResourcePlanDenialCode,
    WorthServerExternalResourceResultAdmissionError, WorthServerExternalResourceTransport,
    WorthServerExternalResourceTransportOutcome, WorthServerExternalResourceTransportResponse,
    WorthServerLoweredExternalResourcePlan,
};
pub use facade::{WorthServer, WorthServerBuildError, WorthServerBuilder};
pub use middleware::{
    WorthServerAdmission, WorthServerAdmissionOutcome, WorthServerDenial, WorthServerDenialCode,
    WorthServerDenialPriority, WorthServerMiddlewareDeferred, WorthServerMiddlewareFacade,
    WorthServerMiddlewareFailure, WorthServerMiddlewareRebindRequired, WorthServerMiddlewareStale,
    WorthServerPipelineInput, WorthServerPipelineIntent, WorthServerPipelineStep,
    WorthServerPreparedQueryHandoffIntent, WorthServerPreparedQueryHandoffKind,
};
pub use operation_admission::{
    WorthServerOperationAdmissionDenial, WorthServerOperationAdmissionDenialCode,
    WorthServerOperationAdmissionFacade, WorthServerOperationAdmissionPosture,
    WorthServerOperationAuthorityDeclaration, WorthServerOperationAuthorityFootprint,
    WorthServerOperationAuthorityKind, WorthServerOperationAuthorityMetadata,
    WorthServerOperationAuthorizationProof, WorthServerOperationConcurrencyClass,
    WorthServerOperationConcurrencyDenial, WorthServerOperationConcurrencyDenialCode,
    WorthServerOperationConcurrencyFacade, WorthServerOperationFootprintReceipt,
    WorthServerOperationScope, WorthServerProductSessionCoordinationTarget,
    WorthServerProductSupportPosture, WorthServerSharedReadBasisKind,
};
pub use operation_planning::{
    WorthServerLoweredOperationPlan, WorthServerOperationExecutionStrategy,
    WorthServerOperationPlanCounters, WorthServerOperationPlanDenial,
    WorthServerOperationPlanDenialCode, WorthServerOperationPlanEvidencePolicy,
    WorthServerOperationPlanProof, WorthServerOperationPlanReceipt, WorthServerOperationPlanner,
    WorthServerOperationPlannerInput,
};
pub use operation_readiness::{
    WorthServerCompatibilityMutationPrecondition,
    WorthServerCompatibilityMutationPreconditionContext, WorthServerOperationPreconditionPosture,
    WorthServerOperationQuerySupportContext, WorthServerOperationReadinessClosure,
    WorthServerOperationReadinessDenial, WorthServerOperationReadinessDenialCode,
    WorthServerOperationReadinessDenialFacts, WorthServerOperationReadinessFacade,
    WorthServerOperationSupportCompositionReceipt, WorthServerOperationSupportPosture,
    WorthServerProductBasisPrecondition,
};
pub use operation_registry::{
    WorthServerOperationAuthorizationPolicy, WorthServerOperationCapabilities,
    WorthServerOperationDenial, WorthServerOperationFamily, WorthServerOperationInventory,
    WorthServerOperationInventoryRow, WorthServerOperationRegistration,
    WorthServerOperationRegistry, WorthServerOperationRegistryError,
};
pub use operation_request::{
    WorthServerOperationIdentity, WorthServerOperationInputEnvelope, WorthServerOperationRequest,
    WorthServerOperationRequestDenial, WorthServerOperationRequestDenialCode,
    WorthServerOperationRequestFacade, WorthServerOperationRequestInput,
    WorthServerOperationRequestInputBuilder, WorthServerOperationRequestReceipt,
};
pub use operation_runtime_certification::{
    WorthServerEditorLikeOperationFixture, WorthServerNoProductSemanticsCertification,
    WorthServerOperationRuntimeCloseoutDigest, WorthServerProductEditorReadinessCertification,
    WorthServerProductIdempotentRetryCertificationProof,
    WorthServerProductMutationCertificationProof,
    WorthServerProductOperationRuntimeArtifactRequirements,
    WorthServerProductOperationRuntimeCertification,
    WorthServerProductOperationRuntimeCertificationFacade,
    WorthServerProductOperationRuntimeRequirementRow,
    WorthServerProductOperationRuntimeRequirementStatus,
    WorthServerProductOperationRuntimeSupportRow,
    WorthServerProductPressureShapeCertificationProof,
    WorthServerProductRouteParityCertificationProof, WorthServerProductRouteParityEntry,
    WorthServerProductSharedReadCertificationProof,
    WorthServerProductStaleApplyDenialCertificationProof,
};
pub use operation_scheduler::{
    WorthServerExecutedOperationBatch, WorthServerOperationExecutionSlot,
    WorthServerOperationScheduler, WorthServerOperationSchedulerCounters,
    WorthServerScheduledMutationResult, WorthServerScheduledOperationBatch,
    WorthServerScheduledOperationOutcome, WorthServerScheduledOperationTraceEntry,
    WorthServerSchedulerCancellationDirective, WorthServerSchedulerCancellationPosture,
    WorthServerSchedulerCertificationSabotage, WorthServerSchedulerConflictDenial,
    WorthServerSchedulerConflictDenialCode, WorthServerSchedulerConflictDenialFacts,
    WorthServerSchedulerFailurePosture, WorthServerSchedulerRuntimeFailure,
};
pub use operator_evidence::{
    WorthServerEvidenceInput, WorthServerEvidenceTransform, WorthServerObservedCounter,
    WorthServerOperatorCounterReceipt, WorthServerOperatorEvidenceClass,
    WorthServerOperatorEvidenceFacade, WorthServerOperatorEvidenceMaterializationError,
    WorthServerOperatorEvidencePlan, WorthServerOperatorEvidenceRecord,
};
pub use product_adapter::{
    WorthServerCompletedProductOperation, WorthServerExecutedProductReadBatch,
    WorthServerLoweredProductOperationPlan, WorthServerProductAdapterCertificationCode,
    WorthServerProductAdapterCertificationError, WorthServerProductAdapterExecutionError,
    WorthServerProductAdapterRegistrationReceipt, WorthServerProductAdapterRegistry,
    WorthServerProductAdapterRegistryError, WorthServerProductApplicationAdapter,
    WorthServerProductApplicationAdapterRegistration,
    WorthServerProductOperationAuthorityRequirement, WorthServerProductOperationAuthorization,
    WorthServerProductOperationAuthorizationDenial,
    WorthServerProductOperationAuthorizationRequest, WorthServerProductOperationAuthorizer,
    WorthServerProductOperationBasisKind, WorthServerProductOperationDeclaration,
    WorthServerProductOperationDenial, WorthServerProductOperationDenialCode,
    WorthServerProductOperationDenialFacts, WorthServerProductOperationEnvelope,
    WorthServerProductOperationEnvelopeKind, WorthServerProductOperationErrorMap,
    WorthServerProductOperationErrorMaps, WorthServerProductOperationExecutionBoundary,
    WorthServerProductOperationFailure, WorthServerProductOperationInput,
    WorthServerProductOperationOutcome, WorthServerProductOperationPayload,
    WorthServerProductOperationRetryClass, WorthServerProductOperationRetryDiagnostics,
    WorthServerProductOperationRuntime, WorthServerProductOperationSuccess,
    WorthServerProductOperationSupportSnapshot, WorthServerProductOperationSurfaceDenial,
    WorthServerProductOperationSurfaceDenialCode, WorthServerProductOperationSurfaceDenialFacts,
    WorthServerProductPayloadSchemaValidator, WorthServerProductReadBatchStop,
    WorthServerProductReadTransport, WorthServerProductSchedulerAdmission,
    WorthServerScheduledProductOperation,
};
pub use product_operation_contract::{
    WorthServerProductIdempotencyConflict, WorthServerProductIdempotencyKey,
    WorthServerProductIdempotencyRecord, WorthServerProductOperationBaseDigest,
    WorthServerProductOperationRetryReceipt, WorthServerProductRebaseRequired,
    WorthServerProductSnapshotPrecondition, WorthServerProductStaleBasisDenial,
};
pub use product_protocol_catalog::{
    WorthServerProductOperationProtocol, WorthServerProductProtocolCatalog,
    WorthServerProductProtocolCatalogError, WorthServerProductSessionOperationProtocol,
};
pub use product_result::{
    WorthServerProductResultArtifact, WorthServerProductResultArtifactError,
    WorthServerProductResultArtifactErrorCode, WorthServerProductResultBody,
    WorthServerProductResultCanonicalization, WorthServerProductResultContract,
    WorthServerProductResultContractError, WorthServerProductResultContractErrorCode,
    WorthServerProductResultEncoding, WorthServerProductResultSchema,
    WorthServerProductResultValue,
};
pub use product_session::{
    WorthServerProductSession, WorthServerProductSessionClock,
    WorthServerProductSessionCounterSnapshot, WorthServerProductSessionCreationRequest,
    WorthServerProductSessionDenial, WorthServerProductSessionDenialCode,
    WorthServerProductSessionExpiryPosture, WorthServerProductSessionIdentity,
    WorthServerProductSessionLifecycle, WorthServerProductSessionRegistry,
    WorthServerProductSessionTermination, WorthServerProductSessionTerminationKind,
    WorthServerProductSessionTerminationObserver, WorthServerSystemProductSessionClock,
};
pub use product_session_coordination::{
    WorthServerCompletedProductSessionCoordination,
    WorthServerLoweredProductSessionCoordinationPlan, WorthServerProductSessionCoordinationCommand,
    WorthServerProductSessionCoordinationRuntime, WorthServerProductSessionSchedulerAdmission,
};
pub use query_dependency_audit::{
    WorthServerQueryDependencyAudit, WorthServerQueryDependencyAuditFacade,
    WorthServerQueryDependencyAuditPathKind, WorthServerQueryDependencyAuditProvenance,
    WorthServerQueryDependencyAuditReceipt, WorthServerQueryDependencyAuditRow,
    WorthServerQueryDependencyAuditRowId, WorthServerQueryDependencyClosurePosture,
    WorthServerQueryDependencyConsumerKitPosture, WorthServerQueryDependencyCoveredPathInventory,
    WorthServerQueryDependencyRuntimeReadiness, WorthServerQueryDependencyScopePosture,
    WorthServerQueryDependencySupportPinProvenance, WorthServerQueryDependencySupportPosture,
};
pub use query_handoff::{
    WorthServerQueryHandoff, WorthServerQueryHandoffDeferred, WorthServerQueryHandoffDenial,
    WorthServerQueryHandoffDenialCode, WorthServerQueryHandoffDenialFacts,
    WorthServerQueryHandoffDenialFamily, WorthServerQueryHandoffFacade,
    WorthServerQueryHandoffFailure, WorthServerQueryHandoffInput, WorthServerQueryHandoffOperation,
    WorthServerQueryHandoffOutcome, WorthServerQueryHandoffRebindRequired,
    WorthServerQueryHandoffStale, WorthServerQueryOperation, WorthServerQueryOperationKind,
    WorthServerQueryRequestedResume, WorthServerQueryRequestedResumeKind,
    WorthServerQuerySupportPosture, WorthServerQueryWorkspaceBindingError,
    WorthServerQueryWorkspaceBindingRequest, WorthServerQueryWorkspaceBindingTarget,
    WorthServerQueryWorkspaceProvider,
};
pub use registration::{
    WorthServerSurfaceFamily, WorthServerSurfaceInventory, WorthServerSurfaceRegistration,
    WorthServerSurfaceRegistryError,
};
pub use request_context::{
    WorthServerAuthenticatedPrincipal, WorthServerBranchTarget, WorthServerRequestContext,
    WorthServerRequestContextDeferred, WorthServerRequestContextDenial,
    WorthServerRequestContextDenialCode, WorthServerRequestContextFacade,
    WorthServerRequestContextFailure, WorthServerRequestContextInput,
    WorthServerRequestContextInputBuilder, WorthServerRequestContextInputError,
    WorthServerRequestContextRebindRequired, WorthServerRequestContextStale,
    WorthServerResolvedRequestContext, WorthServerTransportClass, WorthServerWorkspaceTarget,
};
pub use response::{
    WorthServerDenialBoundary, WorthServerDenialCause, WorthServerDenialEnvelope,
    WorthServerResponseEnvelope, WorthServerResponseFacade, WorthServerResponseInput,
    WorthServerResponsePlan, WorthServerResponseReceipt, WorthServerResponseTransform,
    WorthServerSuccessEnvelope, WorthServerSuccessKind, WorthServerSuccessPayload,
};
pub use surfaces::{
    BinarySurface, BinarySurfaceRoot, CompatHttpSurface, CompatHttpSurfaceRoot, IntegrationSurface,
    IntegrationSurfaceRoot, LeaseSurface, LeaseSurfaceRoot, SyncSurface, SyncSurfaceRoot,
    WorthNativeSurface, WorthNativeSurfaceRoot, WorthServerAbuseBudgetReceipt,
    WorthServerBackgroundExportRequest, WorthServerBinaryCertificationBundle,
    WorthServerBinaryCounterSet, WorthServerBinaryDownload, WorthServerBinaryDownloadAuthorization,
    WorthServerBinaryDownloadExecutionInput, WorthServerBinaryDownloadOutcome,
    WorthServerBinaryDownloadRequest, WorthServerBinaryEgressPerformanceReceipt,
    WorthServerBinaryEgressSession, WorthServerBinaryIngressSession,
    WorthServerBinaryIntegrityDigest, WorthServerBinaryPolicyDecision,
    WorthServerBinaryResumeRequest, WorthServerBinaryRetryPosture, WorthServerBinarySessionResume,
    WorthServerCacheabilityPolicy, WorthServerCanonicalFilename, WorthServerCanonicalHeaderSet,
    WorthServerCompatHttpRouteFamilies, WorthServerCompatHttpRouteFamily,
    WorthServerCompatibilityAdmittedProductMutationCommand, WorthServerCompatibilityCachePolicy,
    WorthServerCompatibilityCertificationBundle, WorthServerCompatibilityDeferred,
    WorthServerCompatibilityDenial, WorthServerCompatibilityDenialCode,
    WorthServerCompatibilityExecutionInput, WorthServerCompatibilityExecutionOutcome,
    WorthServerCompatibilityExport, WorthServerCompatibilityFacade,
    WorthServerCompatibilityFailure, WorthServerCompatibilityFileEnvelope,
    WorthServerCompatibilityInspection, WorthServerCompatibilityMutation,
    WorthServerCompatibilityMutationCommand, WorthServerCompatibilityMutationEnvelope,
    WorthServerCompatibilityMutationExecutionInput, WorthServerCompatibilityMutationOutcome,
    WorthServerCompatibilityMutationRequest, WorthServerCompatibilityMutationResult,
    WorthServerCompatibilityOpenedProductSession, WorthServerCompatibilityPreparedRequest,
    WorthServerCompatibilityProductSessionContinuation,
    WorthServerCompatibilityProductSessionFacade, WorthServerCompatibilityRead,
    WorthServerCompatibilityRebindRequired, WorthServerCompatibilityRequest,
    WorthServerCompatibilityRequestInput, WorthServerCompatibilityRequestInputBuilder,
    WorthServerCompatibilityRequestInputError, WorthServerCompatibilityRequestOutcome,
    WorthServerCompatibilityStale, WorthServerCompatibilityState, WorthServerCompatibilityStream,
    WorthServerCompatibilityUpload, WorthServerCompatibilityUploadExecutionInput,
    WorthServerCompatibilityUploadOutcome, WorthServerCompatibilityVersion,
    WorthServerConditionalRangeRequest, WorthServerConditionalRead,
    WorthServerExternalBasisRequest, WorthServerExternalCounterSet,
    WorthServerExternalEvidenceRecord, WorthServerExternalRequestContract,
    WorthServerFileMetadataReceipt, WorthServerFileMetadataTruthKind,
    WorthServerFileTransferDisposition, WorthServerFileTransferProvenance,
    WorthServerIdempotencyKey, WorthServerIdempotentRetryReceipt,
    WorthServerIngressIntegrityDigest, WorthServerIngressPerformanceReceipt,
    WorthServerMetadataNormalizationReceipt, WorthServerMultipartUpload,
    WorthServerMutationPrecondition, WorthServerNegotiatedRepresentation,
    WorthServerPreparedMultipartUpload, WorthServerRangeRequest, WorthServerReadValidator,
    WorthServerStreamCancellationKind, WorthServerStreamCancellationReceipt,
    WorthServerStreamFinishError, WorthServerStreamSelection, WorthServerStreamingChunk,
    WorthServerStreamingPerformanceReceipt, WorthServerStreamingResponse,
    WorthServerSurfaceCapabilities, WorthServerSurfaceRoot, WorthServerSurfacesFacade,
    WorthServerTransferByteClass, WorthServerTransferCleanupEvidence,
    WorthServerTransferCleanupReason, WorthServerUploadChunk, WorthServerUploadCleanupReason,
    WorthServerUploadCleanupReceipt, WorthServerUploadContentEncoding,
    WorthServerUploadExpectation, WorthServerUploadManifest, WorthServerUploadPart,
    WorthServerUploadTransferMode, WorthServerVerifiedBinaryIngress,
};
pub use transport::{
    WorthServerAdmittedTransportCaller, WorthServerDeclaredRoute, WorthServerOperationRouter,
    WorthServerOperationalRoute, WorthServerOperationalRouteKind,
    WorthServerOperationalRouteOutcome, WorthServerProjectedRouter, WorthServerQueryRequestScope,
    WorthServerRouteAssembly, WorthServerRouteAssemblyError, WorthServerRouteBranchTarget,
    WorthServerRouteExecutionBridge, WorthServerRouteExecutionOutcome, WorthServerRouteInventory,
    WorthServerRouteInventoryRow, WorthServerRouteTransportRequest,
    WorthServerTransportCallerAdmissionRequest, WorthServerTransportCallerDenial,
    WorthServerTransportCallerVerification, WorthServerTransportCallerVerifier,
    WorthServerTransportDenial, WorthServerTransportDenialCode, WorthServerVerifiedTransportCaller,
};
pub use worth_native::{
    WorthServerDirectAsyncResultState, WorthServerDirectContextArtifact,
    WorthServerDirectDeclarationSnapshot, WorthServerDirectDeliveryClass,
    WorthServerDirectDeliveryContract, WorthServerDirectDeliveryOutcome,
    WorthServerDirectDeliveryRequest, WorthServerDirectFactReceipt, WorthServerDirectFreshnessMode,
    WorthServerDirectInspection, WorthServerDirectInspectionOutcome,
    WorthServerDirectLeaseDeclaration, WorthServerDirectLeaseDeclarationOutcome,
    WorthServerDirectMaterializationDigest, WorthServerDirectMaterializedRemaskArtifact,
    WorthServerDirectMutation, WorthServerDirectMutationOutcome, WorthServerDirectMutationResult,
    WorthServerDirectProductFlow, WorthServerDirectProjection,
    WorthServerDirectProjectionConsumption, WorthServerDirectProjectionFactReceipt,
    WorthServerDirectProjectionOutcome, WorthServerDirectProjectionRequest,
    WorthServerDirectProvenance, WorthServerDirectRead, WorthServerDirectReadOutcome,
    WorthServerDirectRemaskArtifact, WorthServerDirectRemaskDisposition,
    WorthServerDirectRemaskPosture, WorthServerDirectRetainedPosture, WorthServerDirectState,
    WorthServerDirectStateOutcome, WorthServerDirectTemporalState, WorthServerWorthNativeDeferred,
    WorthServerWorthNativeDirectFacade, WorthServerWorthNativeFacade,
    WorthServerWorthNativeFailure, WorthServerWorthNativePreparationOutcome,
    WorthServerWorthNativePreparedSession, WorthServerWorthNativeProductFacade,
    WorthServerWorthNativeProductMutationCommand, WorthServerWorthNativeProductSessionFacade,
    WorthServerWorthNativeRebindRequired, WorthServerWorthNativeSession,
    WorthServerWorthNativeSessionDenial, WorthServerWorthNativeSessionDenialCode,
    WorthServerWorthNativeSessionInput, WorthServerWorthNativeSessionInputBuilder,
    WorthServerWorthNativeSessionInputError, WorthServerWorthNativeSessionOutcome,
    WorthServerWorthNativeStale, WorthServerWorthNativeSurfaceRoot,
};
