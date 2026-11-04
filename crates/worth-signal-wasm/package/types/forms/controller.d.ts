import type { SignalValue } from "../model.js";
import type { ComputedSignalHandle } from "../callable_surface.js";
import type {
  FormDirtyState,
  FormFieldDeclaration,
  FormFieldHandle,
  FormInputAdapterDiagnostics,
  FormFieldWritePosture,
  FormFieldsBuilder,
  FormPatchPlan,
  FormReadinessBlocker,
  FormSourceBootstrapArtifact,
  FormSource,
} from "./core.js";
import type { FormHostReport } from "./host.js";
import type { FormHostBindings } from "./host.js";
import type { FormExitReport } from "./exit.js";
import type { FormHandoffReport } from "./handoff.js";
import type { FormRouteAuthorityArtifact, FormRouteAuthorityReport } from "./route_authority.js";
import type { FormInputCapabilitiesReport } from "./input_capabilities.js";
import type { FormResourceSourceReport } from "./resource_source.js";
import type { FormResourceDriftReport } from "./resource_drift.js";
import type { FormResourceMergeArtifact, FormResourceMergeReport } from "./resource_merge.js";
import type { FormAttachmentsReport } from "./attachments.js";
import type { FormAttachmentTransfersReport } from "./attachment_transfers.js";
import type { FormMediaReport } from "./media.js";
import type { FormMessagesReport } from "./messages.js";
import type {
  FormCollaborationDeclaration,
  FormCollaborationReport,
} from "./collaboration.js";
import type { FormInteractionReport } from "./interaction.js";
import type { FormBoundInput, FormBoundInputOptions } from "./input_binding.js";
import type { FormNavigationReport } from "./navigation.js";
import type { FormAccessibilityReport } from "./accessibility.js";
import type { FormLayoutReport } from "./layout.js";
import type { FormPresentationDeclaration, FormPresentationReport } from "./presentation.js";
import type {
  FormLayoutMeasurementDeclaration,
  FormLayoutMeasurementReport,
  FormLayoutRowMeasurement,
  FormLayoutMeasurementCause,
  FormLayoutSnapshotArtifact,
} from "./measurement.js";
import type { FormActionsBuilder, FormActionsReport } from "./actions.js";
import type {
  FormActionExecutionArtifact,
  FormActionResultArtifact,
} from "./actions.js";
import type {
  FormAdmissionBuilder,
  FormAdmissionReport,
} from "./admission.js";
import type {
  FormAvailabilityBuilder,
  FormAvailabilityReport,
} from "./availability.js";
import type {
  FormMessageArtifact,
  FormAsyncValidationLifecycleArtifact,
  FormValidationBuilder,
  FormValidationReport,
} from "./validation.js";
import type {
  FormStepsBuilder,
  FormStepsReport,
} from "./steps.js";
import type { FormCanonicalizationArtifact } from "./canonicalization.js";
import type {
  FormSourceAuthorityDiagnostics,
  FormSourceDeclaration,
  FormSourceFactory,
} from "./sources.js";
import type { FormReplayRestoreArtifact } from "./replay_restore.js";
import type { FormResetArtifact } from "./reset.js";
import type {
  FormDiagnosticsHistoryArtifact,
  FormDiagnosticsSummaryReport,
} from "./diagnostics.js";
import type { FormStateHistoryArtifact } from "./state_history.js";
import type { FormPresentationHistoryArtifact } from "./presentation.js";
import type { FormVerificationPackage } from "./verification.js";
import type { MergePolicyPreviewRequest } from "../diagnostics.js";
import type { FormControllerActionBindings } from "./controller_actions.js";
import type { FormControllerPresentationBindings } from "./controller_presentation.js";
import type { RouteFormsAuthorityArtifact } from "../router_admission_surface.js";
import type { AdmittedRouteCapability } from "../router_admission_surface.js";

export interface FormDeclaration<
  TSource = SignalValue,
  TFields extends Record<string, FormFieldDeclaration> = Record<string, FormFieldDeclaration>,
> {
  id?: string;
  contract?: string;
  source: FormSource<TSource> | FormSourceDeclaration<TSource>;
  fields: TFields | FormFieldsBuilder<TFields>;
  validation?: Record<string, unknown> | FormValidationBuilder;
  availability?: Record<string, unknown> | FormAvailabilityBuilder;
  admission?: Record<string, unknown> | FormAdmissionBuilder;
  steps?: Record<string, unknown> | FormStepsBuilder;
  actions?: Record<string, unknown> | FormActionsBuilder;
  host?: FormHostBindings;
  collaboration?: FormCollaborationDeclaration;
  measurement?: FormLayoutMeasurementDeclaration;
  presentation?: FormPresentationDeclaration;
}

export interface FormController<
  TSource = SignalValue,
  TFieldHandles extends Record<string, FormFieldHandle> = Record<string, FormFieldHandle>,
> extends FormControllerPresentationBindings, FormControllerActionBindings {
  readonly fields: TFieldHandles;
  /** Atomically set/clear declared fields after validating the complete batch. */
  setFields(values: Readonly<Record<string, SignalValue>>, options?: { readonly clearDraft?: readonly string[]; readonly touch?: readonly string[] }): void | Promise<void>;
  declaration(): {
    readonly formId: string;
    readonly contract: string;
    readonly source: {
      readonly kind: string;
      readonly sourceId: string;
    };
    readonly fieldFamilies: {
      readonly scalar: number;
      readonly repeated: number;
      readonly attachment: number;
      readonly evidence: number;
    };
    readonly fieldCount: number;
  };
  source(): TSource;
  sourceAuthority(): FormSourceAuthorityDiagnostics;
  fieldContract(): ReadonlyArray<{
    readonly id: string;
    readonly name: string;
    readonly family: "scalar" | "repeated" | "attachment" | "evidence";
    readonly path: string;
    readonly collectionIdentity: null | {
      readonly kind: "field" | "resolver";
      readonly field: string | null;
      readonly posture: string;
    };
    readonly resourceLocus: null | {
      readonly kind: "collectionItems";
      readonly placement: "append" | "prepend";
      readonly posture: string;
    } | {
      readonly kind: "field";
      readonly field: string;
      readonly posture: string;
    } | {
      readonly kind: "jsonPath";
      readonly path: string;
      readonly posture: string;
    } | {
      readonly kind: "region";
      readonly region: string;
      readonly posture: string;
    };
    readonly attachment: null | {
      readonly identityKind: "field" | "resolver";
      readonly identityField: string | null;
      readonly metadata: Readonly<Record<string, SignalValue>>;
      readonly posture: string;
    };
  }>;
  inputAdapters(): ReadonlyArray<{
    readonly field: string;
    readonly path: string;
    readonly family: "scalar" | "repeated" | "attachment" | "evidence";
    readonly tier: FormInputAdapterDiagnostics["tier"];
    readonly capabilities: FormInputAdapterDiagnostics["capabilities"];
    readonly unavailable: FormInputAdapterDiagnostics["unavailable"];
  }>;
  draft(): Partial<TSource>;
  effective(): TSource;
  sourceAdmission(): FormSourceBootstrapArtifact | null;
  draftRestore(): FormSourceBootstrapArtifact | null;
  resourceSource(): FormResourceSourceReport | null;
  resourceMerge(): FormResourceMergeReport;
  resourceDrift(): FormResourceDriftReport;
  previewResourceMerge(request: MergePolicyPreviewRequest): FormResourceMergeArtifact;
  clearResourceMerge(reason?: string): FormResourceMergeArtifact;
  host(): FormHostReport;
  inputCapabilities(): FormInputCapabilitiesReport;
  inputCapability(fieldId: string): import("./input_capabilities.js").FormInputCapabilityArtifact | null;
  bindInput<TValue = SignalValue, TRaw = TValue>(
    fieldId: string,
    options?: FormBoundInputOptions<TValue, TRaw>,
  ): FormBoundInput<TValue, TRaw>;
  exit(): FormExitReport;
  handoff(): FormHandoffReport;
  routeAuthority(): FormRouteAuthorityReport;
  reportRouteAuthority(authority: RouteFormsAuthorityArtifact): FormRouteAuthorityArtifact;
  bindRouteAuthority(
    routeOrAuthority: AdmittedRouteCapability | RouteFormsAuthorityArtifact,
  ): FormRouteAuthorityArtifact;
  clearRouteAuthority(options?: { readonly reason?: string }): FormRouteAuthorityArtifact;
  controlAvailabilities(): ReadonlyArray<import("./availability.js").FormAvailabilityArtifact>;
  controlAvailability(controlId: string): import("./availability.js").FormAvailabilityArtifact | null;
  attachments(): FormAttachmentsReport;
  attachmentTransfers(): FormAttachmentTransfersReport;
  media(): FormMediaReport;
  messages(): FormMessagesReport;
  collaboration(): FormCollaborationReport;
  interaction(): FormInteractionReport;
  reportFieldInteraction(
    fieldId: string,
    event: {
      readonly kind:
        | "touch"
        | "visit"
        | "focus"
        | "blur"
        | "input"
        | "compositionStart"
        | "compositionCommit"
        | "compositionCancel";
      readonly source?: string;
      readonly rawValue?: SignalValue;
    },
  ): FormInteractionReport["history"][number];
  reportSubmitIntent(options?: {
    readonly source?: "keyboard" | "pointer" | "programmatic";
  }): FormInteractionReport["history"][number];
  clearSubmitIntent(options?: { readonly reason?: string }): FormInteractionReport["history"][number];
  navigation(): FormNavigationReport;
  accessibility(): FormAccessibilityReport;
  layout(): FormLayoutReport;
  layoutField(fieldId: string): import("./layout.js").FormLayoutFieldHint | null;
  layoutMeasurement(): FormLayoutMeasurementReport;
  recordLayoutMeasurement(
    rows: ReadonlyArray<FormLayoutRowMeasurement>,
    options?: {
      readonly cause?: FormLayoutMeasurementCause;
      readonly frameToken?: string | number | null;
    },
  ): FormLayoutSnapshotArtifact;
  dirty(): FormDirtyState;
  patchPlan(): FormPatchPlan;
  validation(): FormValidationReport;
  availability(): FormAvailabilityReport;
  admission(): FormAdmissionReport;
  steps(): FormStepsReport;
  actions(): FormActionsReport;
  sourceCompatibility(): {
    readonly posture: "notDeclared" | "current" | "compatible" | "migrated" | "unavailable";
    readonly currentSchemaVersion: string | null;
    readonly draftSchemaVersion: string | null;
    readonly reason: string | null;
    readonly artifact: {
      readonly kind: "sourceCompatibility";
      readonly artifactId: number;
      readonly posture: "compatible" | "migrated" | "unavailable";
      readonly previousSchemaVersion: string | null;
      readonly currentSchemaVersion: string | null;
      readonly previousDraftDigest: string;
      readonly nextDraftDigest: string;
      readonly nextDraft: SignalValue | null;
      readonly reason: string;
      readonly resolutionKey: string;
      readonly compatibilityDigest: string;
    } | null;
    readonly counters: {
      readonly costBasis: "sourceSchemaCompatibilityDerivedScan";
      readonly incrementalStatus: "notIncremental";
      readonly schemaReads: number;
      readonly migrations: number;
      readonly compatibleDrifts: number;
      readonly unavailableDrifts: number;
      readonly historyArtifacts: number;
    };
  };
  sourceCompatibilityHistory(): ReadonlyArray<{
    readonly kind: "sourceCompatibility";
    readonly artifactId: number;
    readonly posture: "compatible" | "migrated" | "unavailable";
    readonly previousSchemaVersion: string | null;
    readonly currentSchemaVersion: string | null;
    readonly previousDraftDigest: string;
    readonly nextDraftDigest: string;
    readonly nextDraft: SignalValue | null;
    readonly reason: string;
    readonly resolutionKey: string;
    readonly compatibilityDigest: string;
  }>;
  verification(): FormVerificationPackage;
  visibleMessages(): ReadonlyArray<FormMessageArtifact>;
  actionReadiness(actionId: string): {
    readonly action: string;
    readonly canRun: boolean;
    readonly blockers: ReadonlyArray<FormReadinessBlocker>;
  };
  fieldWritePosture(fieldId: string, capability?: "edit" | "patch"): FormFieldWritePosture;
  readiness(): {
    readonly canSubmit: boolean;
    readonly blockers: ReadonlyArray<FormReadinessBlocker>;
    readonly patchPlan: FormPatchPlan;
  };
  diagnosticsSummary(): FormDiagnosticsSummaryReport;
  diagnosticsHistory(): ReadonlyArray<FormDiagnosticsHistoryArtifact>;
  stateHistory(): ReadonlyArray<FormStateHistoryArtifact>;
  summarySignal(): ComputedSignalHandle<{
    readonly source: TSource;
    readonly draft: Partial<TSource>;
    readonly effective: TSource;
    readonly dirty: FormDirtyState;
    readonly patchPlan: FormPatchPlan;
    readonly readiness: {
      readonly canSubmit: boolean;
      readonly blockers: ReadonlyArray<FormReadinessBlocker>;
      readonly patchPlan: FormPatchPlan;
    };
    readonly visibleMessages: ReadonlyArray<FormMessageArtifact>;
  }>;
  diagnostics(): {
    readonly kind: "form";
    readonly declaration: ReturnType<FormController["declaration"]>;
    readonly fieldCount: number;
    readonly sourceAuthority: FormSourceAuthorityDiagnostics;
    readonly summary: FormDiagnosticsSummaryReport;
    readonly fieldContract: ReturnType<FormController["fieldContract"]>;
    readonly inputAdapters: ReturnType<FormController["inputAdapters"]>;
    readonly dirty: FormDirtyState;
    readonly patchPlan: FormPatchPlan;
    readonly readiness: ReturnType<FormController["readiness"]>;
    readonly validation: FormValidationReport;
    readonly availability: FormAvailabilityReport;
    readonly admission: FormAdmissionReport;
    readonly resourceSource: FormResourceSourceReport | null;
    readonly resourceMerge: FormResourceMergeReport;
    readonly resourceDrift: FormResourceDriftReport;
    readonly host: FormHostReport;
    readonly inputCapabilities: FormInputCapabilitiesReport;
    readonly exit: FormExitReport;
    readonly handoff: FormHandoffReport;
    readonly routeAuthority: FormRouteAuthorityReport;
    readonly attachments: FormAttachmentsReport;
    readonly media: FormMediaReport;
    readonly messages: FormMessagesReport;
    readonly collaboration: FormCollaborationReport;
    readonly interaction: FormInteractionReport;
    readonly navigation: FormNavigationReport;
    readonly accessibility: FormAccessibilityReport;
    readonly layout: FormLayoutReport;
    readonly layoutMeasurement: FormLayoutMeasurementReport;
    readonly presentation: FormPresentationReport;
    readonly sourceCompatibility: FormController<TSource, TFieldHandles>["sourceCompatibility"] extends () => infer TReport
      ? TReport
      : never;
    readonly steps: FormStepsReport;
    readonly actions: FormActionsReport;
    readonly actionHistory: ReadonlyArray<FormActionResultArtifact>;
    readonly actionExecutionHistory: ReadonlyArray<FormActionExecutionArtifact>;
    readonly presentationHistory: ReadonlyArray<FormPresentationHistoryArtifact>;
    readonly asyncValidationHistory: ReadonlyArray<FormAsyncValidationLifecycleArtifact>;
    readonly canonicalizationHistory: ReadonlyArray<FormCanonicalizationArtifact>;
    readonly resetHistory: ReadonlyArray<FormResetArtifact>;
    readonly stateHistory: ReadonlyArray<FormStateHistoryArtifact>;
    readonly replayRestoreHistory: ReadonlyArray<FormReplayRestoreArtifact>;
    readonly sourceCompatibilityHistory: ReturnType<FormController<TSource, TFieldHandles>["sourceCompatibilityHistory"]>;
    readonly diagnosticsHistory: ReadonlyArray<FormDiagnosticsHistoryArtifact>;
    readonly verification: FormVerificationPackage;
    readonly digest: string;
  };
  field(fieldId: string): FormFieldHandle;
}

export type FormFieldHandleFor<TDeclaration> =
  TDeclaration extends FormFieldDeclaration<infer TValue, infer TRaw, infer TFamily>
    ? FormFieldHandle<TValue, TRaw, TFamily>
    : FormFieldHandle;

export interface FormFactory {
  <
    TSource = SignalValue,
    TFields extends Record<string, FormFieldDeclaration> = Record<string, FormFieldDeclaration>,
  >(declaration: FormDeclaration<TSource, TFields>): FormController<
    TSource,
    { [K in keyof TFields]: FormFieldHandleFor<TFields[K]> }
  >;
  readonly define: <
    TSource = SignalValue,
    const TDeclaration extends FormDeclaration<TSource> = FormDeclaration<TSource>,
  >(
    declaration: TDeclaration,
  ) => TDeclaration;
  readonly source: FormSourceFactory;
}
