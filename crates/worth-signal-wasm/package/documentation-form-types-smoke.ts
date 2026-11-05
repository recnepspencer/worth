import { createSignals } from "./index.js";
import type { ResourceLine } from "./types/resource/resource_lifecycle.js";

const signals = await createSignals({ deployment: "mainThreadCompatibility" });

const task = signals.input({ title: "Ship docs", done: false });
const ordinaryForm = signals.form({
  id: "task-editor",
  source: task,
  fields: ({ field }) => ({
    title: field<string>("title"),
    done: field<boolean>("done"),
  }),
});

ordinaryForm.fields.title.set("Publish docs");
ordinaryForm.setFields({ title: "Publish docs", done: true });
ordinaryForm.setFields({}, { clearDraft: ["title", "done"] });
ordinaryForm.fields.title.sourceValue();
ordinaryForm.fields.title.draftValue();
ordinaryForm.fields.title.value();
ordinaryForm.source();
ordinaryForm.draft();
ordinaryForm.effective();
ordinaryForm.dirty();
ordinaryForm.patchPlan();
ordinaryForm.readiness();
ordinaryForm.sourceAuthority();
ordinaryForm.declaration();
ordinaryForm.fieldContract();
ordinaryForm.diagnosticsSummary();

const profile = signals.input({
  name: "Ada",
  tags: [{ id: "systems", label: "Systems" }],
});

const structuredForm = signals.form({
  source: signals.form.source.signal(profile, { id: "profile" }),
  fields: ({ field, repeated }) => ({
    name: field<string>("name"),
    tags: repeated<Array<{ id: string; label: string }>>("tags", {
      itemIdentity: "id",
    }),
  }),
});

structuredForm.fields.tags.addItem({ id: "math", label: "Mathematics" });
structuredForm.fields.name.dirty().isDirty;
structuredForm.patchPlan().empty;

const validatedForm = signals.form({
  source: { email: "", seats: 1, handle: "ada" },
  fields: ({ field }) => ({
    email: field<string>("email"),
    seats: field<number, string>("seats", {
      parse: (raw) => Number.parseInt(raw, 10),
    }),
    handle: field<string>("handle"),
  }),
  validation: ({ field, asyncField }) => ({
    emailRequired: field<string>("email", (value) =>
      value.includes("@")
        ? { kind: "valid", field: "email", digest: value }
        : {
            kind: "invalid",
            field: "email",
            message: {
              code: "email.invalid",
              message: "Enter a complete email address.",
              severity: "error",
              audience: "user",
              visibility: "visible",
            },
          },
    ),
    handleAvailable: asyncField("handle", {
      triggers: ["blur", "submit"],
      debounceMs: 250,
    }),
  }),
});

validatedForm.fields.seats.input("4").commitInput();
const validationOperation = validatedForm.startAsyncValidation("handleAvailable");
validatedForm.fulfillAsyncValidation(validationOperation.operationId, {
  reason: "handle is available",
});
validatedForm.rejectAsyncValidation(validationOperation.operationId, {
  reason: "handle is already taken",
});

const governedForm = signals.form({
  source: { status: "draft", owner: "Ada", title: "Draft" },
  fields: ({ field }) => ({
    status: field<string>("status"),
    owner: field<string>("owner"),
    title: field<string>("title"),
  }),
  availability: ({ field }) => ({
    ownerPosture: field("owner", ["status"], (values) =>
      values.status === "published"
        ? { state: "readonly", draftPolicy: "freeze" }
        : "enabled",
    ),
  }),
  admission: ({ action }) => ({
    publishApproval: action("publish", "approval", ["status"], () => ({
      posture: "requiresApproval",
      actorDigest: "reviewer-42",
      reason: "publication requires independent review",
    })),
  }),
  actions: ({ submit, action }) => ({
    submit: submit({
      hostEffect: "article.update",
      hostRequirements: ["online", "credentials"],
    }),
    publish: action("publish", { hostEffect: "article.publish" }),
    saveDraft: action("saveDraft", {
      patchPolicy: "allowEmpty",
      hostEffect: "draft.store",
    }),
  }),
});

const actionPlan = governedForm.actionPlan("submit");
const actionExecution = await governedForm.executeAction("submit");
const executionUsedCurrentPlan = actionPlan.planDigest === actionExecution.planDigest;
if (actionExecution.resultKind === "pending") {
  governedForm.fulfillAction(actionExecution.operationId, {
    reason: "server accepted the patch",
    canonicalValue: { status: "draft", owner: "Ada", title: "Saved" },
  });
  governedForm.rejectAction(actionExecution.operationId, {
    reason: "save failed",
  });
}

const stepForm = signals.form({
  source: { name: "", role: "", accepted: false },
  fields: ({ field }) => ({
    name: field<string>("name"),
    role: field<string>("role"),
    accepted: field<boolean>("accepted"),
  }),
  steps: ({ step }) => ({
    identity: step("identity", ["name", "role"], { order: 1 }),
    consent: step("consent", ["accepted"], {
      order: 2,
      dependencies: ["role"],
    }),
  }),
  actions: ({ step }) => ({
    continue: step("continue", "identity", "next"),
    back: step("back", "consent", "back"),
  }),
});

stepForm.steps();

const layoutForm = signals.form({
  source: { title: "Draft", seats: 1 },
  fields: ({ field }) => ({
    title: field<string>("title", {
      label: "Title",
      description: "Shown to reviewers",
      row: "summary",
      density: "comfortable",
      accessibility: {
        readingOrder: 1,
        focusOrder: 1,
        describedBy: ["title-help"],
      },
    }),
    seats: field<number, string>("seats", {
      parse: (raw) => Number.parseInt(raw, 10),
      adapter: {
        tier: "externalImperative",
        reportsRawInput: true,
        reportsCommitBoundary: true,
      },
    }),
  }),
});

const boundSeats = layoutForm.bindInput<number, string>("seats");
boundSeats.input("4", { commit: true });
boundSeats.focus();
boundSeats.blur();
layoutForm.inputCapability("seats");
layoutForm.inputCapabilities();

const projectLine = null as unknown as ResourceLine<
  { projectId: string },
  { name: string; status: string }
>;

const resourceForm = signals.form({
  source: signals.form.source.resourceLine(projectLine, { id: "project-42" }),
  fields: ({ field }) => ({
    name: field<string>("name"),
    status: field<string>("status"),
  }),
  actions: ({ submit, action }) => ({
    submit: submit({
      resourceEffectProfile: signals.resource.effects.branchNative(),
    }),
    refresh: action("refresh", {
      resourceAction: { kind: "refresh" },
    }),
  }),
});
const resourceExecution = await resourceForm.executeAction("submit");
const resourceSubmission = resourceExecution.resourceSubmission;

void ordinaryForm;
void structuredForm;
void validatedForm;
void governedForm;
void stepForm;
void layoutForm;
void boundSeats;
void resourceForm;
void executionUsedCurrentPlan;
void resourceSubmission;
