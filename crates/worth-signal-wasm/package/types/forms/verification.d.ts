import type { FormActionsReport } from "./actions.js";
import type { FormAdmissionReport } from "./admission.js";
import type { FormAvailabilityReport } from "./availability.js";
import type { FormStepsReport } from "./steps.js";
import type { FormValidationReport } from "./validation.js";
import type { FormHostReport } from "./host.js";
import type { FormExitReport } from "./exit.js";
import type { FormHandoffReport } from "./handoff.js";
import type { FormRouteAuthorityReport } from "./route_authority.js";
import type { FormAttachmentsReport } from "./attachments.js";
import type { FormAttachmentTransfersReport } from "./attachment_transfers.js";
import type { FormMessagesReport } from "./messages.js";
import type { FormMediaReport } from "./media.js";
import type { FormCollaborationReport } from "./collaboration.js";
import type { FormInteractionReport } from "./interaction.js";
import type { FormNavigationReport } from "./navigation.js";
import type { FormAccessibilityReport } from "./accessibility.js";
import type { FormLayoutReport } from "./layout.js";
import type { FormLayoutMeasurementReport } from "./measurement.js";
import type { FormPresentationReport } from "./presentation.js";
import type { FormInputCapabilitiesReport } from "./input_capabilities.js";
import type { FormResourceDriftReport } from "./resource_drift.js";
import type { FormResourceSourceReport } from "./resource_source.js";
import type { FormSourceAuthorityDiagnostics } from "./sources.js";

export interface FormVerificationPackage {
  readonly kind: "formVerification";
  readonly digests: {
    readonly sourceAuthorityDigest: string;
    readonly sourceAuthorityContractDigest: string;
    readonly sourceAdmissionDigest: string;
    readonly draftRestoreDigest: string;
    readonly resourceSourceDigest: string | null;
    readonly resourceDriftDigest: string;
    readonly resourceShapeDigest: string | null;
    readonly resourceLifecycleDigest: string | null;
    readonly resourceSettlementDigest: string | null;
    readonly resourceEffectProfileDigest: string;
    readonly resourceExternalCompatibilityDigest: string | null;
    readonly resourceTransferDigest: string | null;
    readonly resourceVisibleBranchSelectionDigest: string;
    readonly resourceVerificationPackageDigest: string | null;
    readonly resourceEffectCloseoutMatrixDigest: string | null;
    readonly resourceMutationResponseDigest: string | null;
    readonly resourceMutationResponseConfirmationDigest: string | null;
    readonly resourceMutationResponseContractDigest: string | null;
    readonly resourceMutationResponseCompletionDigest: string | null;
    readonly resourceMutationResponseTargetOutcomeDigest: string | null;
    readonly resourceMutationResponseCloseoutMatrixDigest: string | null;
    readonly sourceValueDigest: string;
    readonly formDeclarationDigest: string;
    readonly fieldContractDigest: string;
    readonly inputAdapterCapabilityDigest: string;
    readonly hostFactDigest: string;
    readonly inputCapabilityDigest: string;
    readonly exitDigest: string;
    readonly handoffDigest: string;
    readonly routeAuthorityDigest: string;
    readonly routeAuthorityContinuityDigest: string;
    readonly attachmentDigest: string;
    readonly attachmentTransferDigest: string;
    readonly messageDigest: string;
    readonly mediaDigest: string;
    readonly collaborationDigest: string;
    readonly collaborationEventDigest: string;
    readonly interactionDigest: string;
    readonly interactionHistoryDigest: string;
    readonly navigationDigest: string;
    readonly navigationHistoryDigest: string;
    readonly accessibilityDigest: string;
    readonly presentationOrderHintDigest: string | null;
    readonly layoutDigest: string;
    readonly layoutMeasurementDigest: string;
    readonly presentationDigest: string;
    readonly presentationSettlementAcknowledgementDigest: string | null;
    readonly sourceCompatibilityDigest: string;
    readonly draftDigest: string;
    readonly effectiveValueDigest: string;
    readonly semanticEqualityDigest: string;
    readonly patchPlanDigest: string;
    readonly readinessDigest: string;
    readonly validationDigest: string;
    readonly availabilityDependencyDigest: string;
    readonly stepDeclarationProgressDigest: string;
    readonly admissionPolicyDigest: string;
    readonly regulatedBindingDigest: string;
    readonly actionCatalogDigest: string;
    readonly actionReadinessAdmissionDigest: string;
    readonly actionPlanDigestSetDigest: string;
    readonly submitPlanDigest: string | null;
    readonly actionLifecycleDigest: string;
    readonly actionExecutionLifecycleDigest: string;
    readonly asyncValidationLifecycleDigest: string;
    readonly canonicalizationDigest: string;
    readonly replayRestoreDigest: string;
    readonly resetRollbackDigest: string;
    readonly resetHistoryDigest: string;
    readonly replayRestoreHistoryDigest: string;
    readonly stateHistoryDigest: string;
    readonly resourceDriftHistoryDigest: string;
    readonly mutationResponseReconciliationDigest: string;
    readonly sourceCompatibilityHistoryDigest: string;
    readonly presentationHistoryDigest: string;
    readonly diagnosticsHistoryDigest: string;
    readonly diagnosticsSummaryDigest: string;
    readonly diagnosticsDigest: string;
  };
  readonly sourceAuthority: FormSourceAuthorityDiagnostics;
  readonly routeAuthorityContinuity: {
    readonly kind: "routeAuthorityContinuityAudit";
    readonly handoffPosture:
      NonNullable<FormRouteAuthorityReport["summary"]["handoff"]>["posture"] | null;
    readonly routeCoupledBehavior:
      NonNullable<FormRouteAuthorityReport["summary"]["handoff"]>["routeCoupledBehavior"] | null;
    readonly draftDisposition:
      NonNullable<FormRouteAuthorityReport["summary"]["handoff"]>["draftDisposition"] | null;
    readonly draftResolution:
      NonNullable<FormRouteAuthorityReport["summary"]["draftContinuity"]>["draftResolution"] | null;
    readonly transitionKind: FormRouteAuthorityReport["summary"]["transitionKind"];
    readonly authorityAvailable: boolean;
    readonly routeCoupledSteps: {
      readonly total: number;
      readonly active: number;
      readonly unavailable: number;
    };
    readonly routeCoupledActions: {
      readonly total: number;
      readonly accepted: number;
      readonly denied: number;
    };
    readonly blockingReason: string | null;
    readonly digest: string;
  };
  readonly actionHistory: {
    readonly attempts: number;
    readonly digest: string;
  };
  readonly actionExecutionHistory: {
    readonly operations: number;
    readonly digest: string;
  };
  readonly interactionHistory: {
    readonly operations: number;
    readonly digest: string;
  };
  readonly navigationHistory: {
    readonly operations: number;
    readonly digest: string;
  };
  readonly asyncValidationHistory: {
    readonly operations: number;
    readonly digest: string;
  };
  readonly canonicalizationHistory: {
    readonly operations: number;
    readonly digest: string;
  };
  readonly resetHistory: {
    readonly operations: number;
    readonly digest: string;
  };
  readonly replayRestoreHistory: {
    readonly operations: number;
    readonly digest: string;
  };
  readonly stateHistory: {
    readonly operations: number;
    readonly digest: string;
  };
  readonly presentationHistory: {
    readonly operations: number;
    readonly digest: string;
  };
  readonly diagnosticsHistory: {
    readonly operations: number;
    readonly digest: string;
  };
  readonly sourceCompatibilityHistory: {
    readonly operations: number;
    readonly digest: string;
  };
  readonly performanceEnvelope: {
    readonly costBasis: "derivedFullReportScan";
    readonly diagnosticsSummaryBreadth: "summaryShapedNotFullHistoryMaterialization";
    readonly diagnosticsHistoryOperations: number;
    readonly fieldWriteOperations: number;
    readonly rawInputOperations: number;
    readonly actionHistoryAttempts: number;
    readonly actionExecutionOperations: number;
    readonly asyncValidationOperations: number;
    readonly canonicalizationOperations: number;
    readonly resetOperations: number;
    readonly replayRestoreOperations: number;
    readonly interactionOperations: number;
    readonly navigationOperations: number;
    readonly sourceCompatibilityOperations: number;
    readonly resourceSource: FormResourceSourceReport["counters"] | null;
    readonly resourceDrift: FormResourceDriftReport["counters"];
    readonly hostFacts: FormHostReport["counters"];
    readonly inputCapabilities: FormInputCapabilitiesReport["counters"];
    readonly exit: FormExitReport["counters"];
    readonly handoff: FormHandoffReport["counters"];
    readonly routeAuthority: FormRouteAuthorityReport["counters"];
    readonly attachments: FormAttachmentsReport["counters"];
    readonly attachmentTransfers: FormAttachmentTransfersReport["counters"];
    readonly messages: FormMessagesReport["counters"];
    readonly media: FormMediaReport["counters"];
    readonly collaboration: FormCollaborationReport["counters"];
    readonly interaction: FormInteractionReport["counters"];
    readonly navigation: FormNavigationReport["counters"];
    readonly accessibility: FormAccessibilityReport["counters"];
    readonly layout: FormLayoutReport["counters"];
    readonly layoutMeasurement: FormLayoutMeasurementReport["counters"];
    readonly presentation: FormPresentationReport["counters"];
    readonly sourceCompatibility: {
      readonly costBasis: "sourceSchemaCompatibilityDerivedScan";
      readonly incrementalStatus: "notIncremental";
      readonly schemaReads: number;
      readonly migrations: number;
      readonly compatibleDrifts: number;
      readonly unavailableDrifts: number;
      readonly historyArtifacts: number;
    };
    readonly validation: FormValidationReport["counters"];
    readonly availability: FormAvailabilityReport["counters"];
    readonly admission: FormAdmissionReport["counters"];
    readonly steps: FormStepsReport["counters"];
    readonly actions: FormActionsReport["counters"];
  };
  readonly packageDigest: string;
}
