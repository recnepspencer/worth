import { createSignals } from "./index.js";

const signals = await createSignals({ deployment: "mainThreadCompatibility" });
const typedResourceLine = null as unknown as import("./types/resource/resource_lifecycle.js").ResourceLine<
  { id: string },
  { title: string }
>;
const typedAttachmentResourceLine = null as unknown as import("./types/resource/resource_lifecycle.js").ResourceLine<
  { id: string },
  { evidence: { digest: string; name: string } }
>;

const phaseEightResourceForm = signals.form({
  source: signals.form.source.resourceLine(typedResourceLine, { id: "task-resource" }),
  fields: ({ field }) => ({
    title: field("title"),
  }),
});
const phaseEightResourceActionForm = signals.form({
  source: signals.form.source.resourceLine(typedResourceLine, { id: "task-resource-action" }),
  fields: ({ field }) => ({
    title: field("title"),
  }),
  actions: ({ submit, action }) => ({
    submit: submit({
      resourceEffectProfile: signals.resource.effects.branchNative(),
    }),
    saveResourceDraft: action("saveResourceDraft", {
      resourceAction: { kind: "patchPlan", fields: ["title"] },
      resourceEffectProfile: signals.resource.effects.branchNative(),
    }),
    refreshResourceSource: action("refreshResourceSource", {
      resourceAction: { kind: "refresh" },
    }),
    replayResourceSource: action("replayResourceSource", {
      resourceAction: { kind: "replayExact" },
    }),
    restoreResourceSource: action("restoreResourceSource", {
      resourceAction: { kind: "restoreExact" },
    }),
    rollbackResourceEffect: action("rollbackResourceEffect", {
      resourceAction: { kind: "rollbackLastEffect" },
    }),
  }),
});
const phaseEightResourceAttachmentForm = signals.form({
  source: signals.form.source.resourceLine(typedAttachmentResourceLine, { id: "task-resource-attachment" }),
  fields: ({ evidence }) => ({
    evidence: evidence<{ digest: string; name: string }>("evidence", {
      attachmentIdentity: "digest",
    }),
  }),
});

const phaseEightResourceSourceKind = phaseEightResourceForm.resourceSource()?.sourceKind;
const phaseEightResourceShapeKind = phaseEightResourceForm.resourceSource()?.shape.familyKind;
const phaseEightResourceSourceDigest = phaseEightResourceForm.verification().digests.resourceSourceDigest;
const phaseEightResourceShapeDigest = phaseEightResourceForm.verification().digests.resourceShapeDigest;
const phaseEightResourceEffectProfileName = phaseEightResourceForm.resourceSource()?.effectProfile.profile?.name;
const phaseEightResourceVisibleSelectionKind = phaseEightResourceForm.resourceSource()?.visibleSelection.kind;
const phaseEightResourceVerificationPackageDigest = phaseEightResourceForm.resourceSource()?.verification.packageDigest;
const phaseEightResourceEffectCloseoutDigest =
  phaseEightResourceForm.verification().digests.resourceEffectCloseoutMatrixDigest;
const phaseEightResourceVisibleBranchDigest =
  phaseEightResourceForm.verification().digests.resourceVisibleBranchSelectionDigest;
const phaseEightResourceDriftStatus = phaseEightResourceForm.resourceDrift().summary.status;
const phaseEightAttachmentTransferBinding =
  phaseEightResourceAttachmentForm.attachmentTransfers().fields[0]?.bindingKind;
const phaseEightAttachmentTransferSurfaceFields =
  phaseEightResourceAttachmentForm.attachmentTransfers().summary.transferSurfaceFields;
const phaseEightAttachmentTransferDigest =
  phaseEightResourceAttachmentForm.verification().digests.attachmentTransferDigest;
const phaseEightResourceTransferDigest =
  phaseEightResourceAttachmentForm.verification().digests.resourceTransferDigest;
const phaseEightReplayRestoreDigest = phaseEightResourceForm.verification().digests.replayRestoreDigest;
const phaseEightResourceSubmitEffectProfileSource =
  phaseEightResourceActionForm.actionPlan("submit").resourceEffectProfile.source;
const phaseEightResourceCustomActionSource =
  phaseEightResourceActionForm.actionPlan("saveResourceDraft").resourceAction.source;
const phaseEightResourceRefreshActionSource =
  phaseEightResourceActionForm.actionPlan("refreshResourceSource").resourceAction.source;
const phaseEightResourceReplayActionSource =
  phaseEightResourceActionForm.actionPlan("replayResourceSource").resourceAction.source;
const phaseEightResourceRestoreActionSource =
  phaseEightResourceActionForm.actionPlan("restoreResourceSource").resourceAction.source;
const phaseEightResourceRollbackActionSource =
  phaseEightResourceActionForm.actionPlan("rollbackResourceEffect").resourceAction.source;
const phaseEightResourceReset = phaseEightResourceForm.reset();
const phaseEightResourceRollback = phaseEightResourceForm.rollbackLastResourceEffect();
const phaseEightResourceResetHistory = phaseEightResourceForm.resetHistory();
const phaseEightResourceReplay = phaseEightResourceForm.replayExactResourceSource();
const phaseEightResourceRestore = phaseEightResourceForm.restoreExactResourceSource();
const phaseEightReplayRestoreHistory = phaseEightResourceForm.replayRestoreHistory();

signals.form({
  source: signals.form.source.resourceLine(typedResourceLine, { id: "task-resource-invalid-action" }),
  fields: ({ field }) => ({
    title: field("title"),
  }),
  actions: ({ action }) => ({
    invalidResourceAction: action("invalidResourceAction", {
      hostEffect: "draft.store",
      // @ts-expect-error resource-line custom patch actions cannot also declare hostEffect
      resourceAction: { kind: "patchPlan" },
    }),
    invalidScopedPatchAction: action("invalidScopedPatchAction", {
      // @ts-expect-error resource-line scoped patch field ids must be strings
      resourceAction: { kind: "patchPlan", fields: [1] },
    }),
    invalidRefreshAction: action("invalidRefreshAction", {
      resourceEffectProfile: signals.resource.effects.branchNative(),
      // @ts-expect-error resource-line lifecycle actions cannot declare resourceEffectProfile
      resourceAction: { kind: "refresh" },
    }),
    invalidReplayAction: action("invalidReplayAction", {
      patchPolicy: "allowEmpty",
      // @ts-expect-error resource-line recovery actions require ignore patch policy
      resourceAction: { kind: "replayExact" },
    }),
  }),
});
signals.form({
  source: signals.form.source.resourceLine(typedResourceLine, { id: "typed-resource-profile" }),
  fields: ({ field }) => ({
    title: field("title"),
  }),
  actions: ({ submit }) => ({
    submit: submit({
      resourceEffectProfile: signals.resource.effects.branchNative(),
    }),
  }),
});
signals.form({
  source: { title: "Plain source" },
  fields: ({ field }) => ({
    title: field("title"),
  }),
  actions: ({ submit }) => ({
    submit: submit({
      // @ts-expect-error resourceEffectProfile must be a validated resource effect profile
      resourceEffectProfile: { name: "forged-profile" },
    }),
  }),
});

void phaseEightResourceForm;
void phaseEightResourceActionForm;
void phaseEightResourceAttachmentForm;
void phaseEightResourceSourceKind;
void phaseEightResourceShapeKind;
void phaseEightResourceSourceDigest;
void phaseEightResourceShapeDigest;
void phaseEightResourceEffectProfileName;
void phaseEightResourceVisibleSelectionKind;
void phaseEightResourceVerificationPackageDigest;
void phaseEightResourceEffectCloseoutDigest;
void phaseEightResourceVisibleBranchDigest;
void phaseEightResourceDriftStatus;
void phaseEightAttachmentTransferBinding;
void phaseEightAttachmentTransferSurfaceFields;
void phaseEightAttachmentTransferDigest;
void phaseEightResourceTransferDigest;
void phaseEightReplayRestoreDigest;
void phaseEightResourceSubmitEffectProfileSource;
void phaseEightResourceCustomActionSource;
void phaseEightResourceRefreshActionSource;
void phaseEightResourceReplayActionSource;
void phaseEightResourceRestoreActionSource;
void phaseEightResourceRollbackActionSource;
void phaseEightResourceReset;
void phaseEightResourceRollback;
void phaseEightResourceResetHistory;
void phaseEightResourceReplay;
void phaseEightResourceRestore;
void phaseEightReplayRestoreHistory;
