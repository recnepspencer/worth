import {
  DEMO_FIVE_CODE,
  DEMO_FOUR_CODE,
  DEMO_ONE_CODE,
  DEMO_THREE_CODE,
  DEMO_TWO_CODE,
} from "./demoCodeSamples.ts";

export interface DemoMetadata {
  id: number;
  title: string;
  purpose: string;
  preface: string;
  difficulty: "Beginner" | "Intermediate" | "Advanced";
  primaryMessage: string;
  WORTHCode: string;
  alternativeCode: string;
  alternativeName: string;
  explanationAlternative: string;
  explanationWORTH: string;
  whatYouGet: string[];
  relatedDocsPath: string;
}

export const demoRegistry: DemoMetadata[] = [
  {
    id: 1,
    title: "Diagnostics No Other State System Has",
    purpose: "Worth has the strongest diagnostics model in web state management. Move one transfer amount, inspect the runtime's exact causal evidence, and ask any derived decision why it behaved that way.",
    preface: "Move a transfer below and across the $10,000 review line. Worth records what was touched, what recomputed, what actually changed, and which dependencies each decision read. Then click a value and ask the same runtime that made the decision to explain it.",
    difficulty: "Beginner",
    primaryMessage: "A derived decision should be able to show its work.",
    WORTHCode: DEMO_ONE_CODE,
    alternativeName: "React useState + useMemo",
    alternativeCode: `import { useState, useMemo } from "react";

const [amount, setAmount] = useState(8_000);
const fee = useMemo(() => amount * 0.004, [amount]);
const reviewLane = useMemo(
  () => amount >= 10_000 ? "Manual review" : "Automatic",
  [amount],
);`,
    explanationAlternative: "React can calculate the same answers. It cannot tell you which policy ran, whether its answer changed, or what input it read unless you build a second evidence system beside it. That is a lot of detective work for two tiny calculations.",
    explanationWORTH: "Worth owns the input, the decisions, and the evidence connecting them. React only keeps the visible comparison list. The useful opinion here is simple: if software makes an important decision, asking why should be a normal API call—not an incident-response exercise.",
    whatYouGet: [
      "One transaction—and one receipt—for each committed change",
      "A direct answer to ‘why did this value do that?’",
      "Proof that a policy can run without changing its answer",
      "An honest boundary between runtime evidence and the UI that displays it"
    ],
    relatedDocsPath: "core/diagnostics"
  },
  {
    id: 2,
    title: "The Read That Keeps a Record",
    purpose: "This looks like an ordinary product page backed by an ordinary fetch. It isn't. Give it a few seconds.",
    preface: "Every resource read here materializes as a line: a query result with a flight recorder attached. When server truth changes — whether you asked for it or not — the line keeps the receipt: status, freshness, provenance, and a lifecycle tape you can export.",
    difficulty: "Intermediate",
    primaryMessage: "When a value changes behind your back, the read itself can tell you why.",
    WORTHCode: DEMO_TWO_CODE,
    alternativeName: "Generic server-state query",
    alternativeCode: `const detail = useQuery({
  queryKey: ["product", productId],
  queryFn: () => loadProduct(productId),
});

// websocket pushes new truth into the cache
queryClient.setQueryData(["product", productId], next);

// the value changed on screen. why? when? from where?
// there is nothing to consult — the cache keeps no record.`,
    explanationAlternative: "Server-state hooks can fetch the data, and a push can rewrite the cache — but nothing records why a value changed. When users ask what happened, the answer lives in log aggregation, if anywhere.",
    explanationWORTH: "Worth materializes the read as a line. A server delivery lands with provenance, the lifecycle tape records it, and the value on screen can testify about its own history — exportable as JSON.",
    whatYouGet: [
      "A real backend push landing as a delivery effect with provenance",
      "Status and freshness as runtime truth, not app conventions",
      "A lifecycle tape read from line.history(), entry by entry",
      "Per-line recorders — switch products, each read has its own",
      "An exportable JSON line history"
    ],
    relatedDocsPath: "resources/debugging/README"
  },
  {
    id: 3,
    title: "The Form That Knows It Has Company",
    purpose: "A payout policy under dual control. Two simulated coworkers edit, comment, and lock fields around you — and the form itself decides, per person, who can write what and whether submit is allowed.",
    preface: "Three people hold the same form: you and two simulated coworkers, each with their own runtime-owned form controller over shared server truth. When Dana leases the limit field, one report produces three different verdicts — she can write, you cannot, and Priya's submit blocks only because her draft touches the leased field. Mostly watch; reach in whenever you like.",
    difficulty: "Beginner",
    primaryMessage: "One collaboration report in. One verdict out — per actor, with reasons.",
    WORTHCode: DEMO_THREE_CODE,
    alternativeName: "React Hook Form + socket glue",
    alternativeCode: `const form = useForm({ values: policy });

// hand-rolled lock state beside the form library
const [locks, setLocks] = useState({});
socket.on("lock", (msg) => setLocks(...));

// disabled props, submit guards, and tooltips
// are all app conventions the form cannot verify
<input disabled={locks.limit && locks.limit !== me} />
<button disabled={
  form.formState.isDirty && locks[touchedField]
} />`,
    explanationAlternative: "The form library owns field state; the lock state lives in app code beside it. Nothing can verify that the disabled props, submit guards, and socket handlers agree — and nothing records why a submit was refused.",
    explanationWORTH: "Collaboration posture lives inside the form controller. One reported lease produces per-actor write posture and patch-plan-aware submit verdicts, each carrying the blocker and the collaborator's name — inspectable and exportable.",
    whatYouGet: [
      "Three real form controllers over one shared source — per-actor truth",
      "Field leases that block writes with the owner's name attached",
      "Patch-plan-aware submit verdicts: leases only block drafts that touch them",
      "Presence, comments, and a collaboration event recorder",
      "An exportable JSON collaboration report"
    ],
    relatedDocsPath: "forms/collaboration/README"
  },
  {
    id: 4,
    title: "The Route That Checks Your Training",
    purpose: "A manufacturing execution portal where opening a batch step is an admission decision — checked against role, training, and the effective SOP revision, with the audit trail as a by-product.",
    preface: "You are an operator trained on SOP-042 rev B. Partway through the session, document control makes rev C effective — and the step you executed minutes ago now denies you, with the reason naming the revision. Proceed under a recorded deviation, then replay the whole session under different facts: the inspector's question, answered by the runtime.",
    difficulty: "Intermediate",
    primaryMessage: "Execution is an access decision. The router keeps the receipts.",
    WORTHCode: DEMO_FOUR_CODE,
    alternativeName: "React Router v6 + audit pipeline",
    alternativeCode: `// guards scattered across loaders
export async function loader({ request }) {
  const user = await getUser(request);
  if (!user.trainedOn("SOP-042")) {
    // which revision? checked where? logged how?
    return redirect("/denied");
  }
  return fetchStep(params);
}

// the audit trail is a separate project:
// middleware, log shipping, retention, correlation…
logAccess(user.id, request.url, "allowed?");`,
    explanationAlternative: "Guards are booleans scattered across loaders, and the audit trail is a separate logging pipeline that has to be kept honest by convention. Replaying a session under different facts — the question an auditor actually asks — has no runtime answer at all.",
    explanationWORTH: "Admission is declared on the route and evaluated against live session facts. Every attempt — admitted, denied, or under deviation — lands as a recorded decision with its reason, and the same session can be re-asked under different facts in one call.",
    whatYouGet: [
      "Admission prerequisites that return decisions with reasons, not booleans",
      "Live facts — role, training, effective revision — checked at every ingress",
      "A recorded audit trail where denials are records too",
      "Deviation-based override as one prerequisite branch, permanently recorded",
      "Session replay under different facts — the inspector's question, answered",
      "Route-owned resource lines with intent prefetch"
    ],
    relatedDocsPath: "router/admission/admit"
  },
  {
    id: 5,
    title: "Optimistic Updates That Never Lie",
    purpose: "A medical supply order stays responsive while three supplier checks resolve differently. The approved item stays, the controlled material leaves, and its required handling kit closes with it.",
    preface: "Nobody working Central Supply wants to watch a spinner negotiate with a server. This demo submits three real inventory lines through the worker-first runtime and lets the purchasing screen show the result: one approval, one rejection, one related cancellation, and no valid work lost.",
    difficulty: "Intermediate",
    primaryMessage: "The inventory responds immediately without sacrificing exact closeout or an honest audit record.",
    WORTHCode: DEMO_FIVE_CODE,
    alternativeName: "Shared snapshot rollback",
    alternativeCode: `const before = readVisibleValue();
applyOptimisticWrite(nextItem);

try {
  await save(nextItem);
} catch {
  restoreVisibleValue(before);
}`,
    explanationAlternative: "A shared snapshot remembers a value, not the causal identity of each overlapping request. Restoring it cannot express an independent success and a dependent cancellation honestly.",
    explanationWORTH: "Every admitted write keeps its own identity and declared dependencies. Rejection removes exactly the failed request and closes its dependents; confirmation reconciles exactly the accepted item. The visible inventory is rebuildable from canonical truth plus pending effects, and every closeout claim is backed by a runtime-issued receipt.",
    whatYouGet: [
      "One independently tracked optimistic effect per inventory write",
      "Declared parent/child dependencies with typed closeout",
      "A scripted server referee and a live agreement badge on the application",
      "Independent success preserved while parent and child close together",
      "Clickable audit receipts — request identity, dependencies, terminal outcome"
    ],
    relatedDocsPath: "resources/effects/README"
  },
  {
    id: 6,
    title: "Branch and Inspect Gear Designs",
    purpose: "Two writable branches record independent, exact aspect edits to one gear.",
    preface: "Fork a design branch from Main. Move either gear and select a commit to see the values stored at that point.",
    difficulty: "Advanced",
    primaryMessage: "Each branch records the exact aspects that changed.",
    WORTHCode: `const branch = await gearTruth.forkBranch({
  parentBranchId: main.id,
  expectedParentBasis: main.basis,
  name: "Design branch",
});

await gearTruth.commit({
  branchId: branch.id,
  expectedBasis: branch.basis,
  operations: gearChanges,
});`,
    alternativeName: "Component-local state",
    alternativeCode: `setGear((gear) => ({ ...gear, teeth: 24 }));`,
    explanationAlternative: "The component stores the current gear value.",
    explanationWORTH: "Local Truth records each edit against an admitted branch basis and projects the committed values to Signal.",
    whatYouGet: [
      "Two writable branches forked from one basis",
      "One-aspect commits: release a slider, commit one locus",
      "A commit graph where every node is a sealed, inspectable snapshot",
      "A live Signal projection receipt — its native basis digest advances with every commit"
    ],
    relatedDocsPath: "local-truth/branches-and-snapshots"
  }
];
