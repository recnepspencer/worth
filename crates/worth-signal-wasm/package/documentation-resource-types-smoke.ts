import {
  createSignals,
  resourceParamIdentity,
  resourceParams,
  resourcePatch,
} from "./index.js";

interface Project {
  id: string;
  name: string;
  revision: number;
}

interface Task {
  id: string;
  title: string;
}

interface ProjectUpdate {
  name: string;
  revision: number;
}

const signals = await createSignals();
const api = signals.api({ baseUrl: "/api" });

const projectDetail = api.url("/projects/:projectId").detail<Project>({
  async load({ projectId }, request) {
    if (!request.target.url) throw new Error("project URL was not admitted");
    const response = await fetch(request.target.url);
    if (!response.ok) throw new Error(`project ${projectId}: ${response.status}`);
    return response.json() as Promise<Project>;
  },
});

const project = projectDetail.line({ projectId: "project-42" });
const settlement = await project.awaitSettlement({ timeoutMs: 5_000 });
if (settlement.resultKind === "fulfilled" || settlement.resultKind === "partial") {
  settlement.value satisfies Project | null;
}

project.invalidate();
project.refresh();
project.revalidate();
project.summary();
project.diagnosticsSummary();
project.history().verificationPackage();

const updateProject = api.url("/projects/:projectId").update<Project, ProjectUpdate>({
  async load({ projectId, body }, request) {
    if (!request.target.url) throw new Error("project URL was not admitted");
    const response = await fetch(request.target.url, {
      method: request.method,
      body: JSON.stringify(body),
    });
    if (!response.ok) throw new Error(`project ${projectId}: ${response.status}`);
    return response.json() as Promise<Project>;
  },
});

updateProject.line({
  projectId: "project-42",
  body: { name: "Launch", revision: 7 },
});

const tasks = signals.api({
  effects: signals.resource.effects.branchNative(),
}).url("/tasks")
  .items((task: Task) => task.id)
  .aspect(
    "title",
    (task) => task.title,
    (task, title: string) => ({ ...task, title }),
  )
  .list({ load: async () => [] as Task[] });

const taskLine = tasks.line({});
const admission = await taskLine.patch(tasks.patch.itemAspect({
  itemId: "task-42",
  aspect: "title",
  value: "Reviewed",
}));

if ("effectId" in admission) {
  const dependentPatch = resourcePatch.dependsOn(
    tasks.patch.item({
      itemId: "task-42",
      nextItem: { id: "task-42", title: "Approved" },
    }),
    [admission.effectId],
  );
  await taskLine.patch(dependentPatch);
  taskLine.effects().get(admission.effectId);
  taskLine.effects().projection();
}

const rawDetail = signals.resource.detail({
  params: resourceParams<{ tenantId: string; documentId: string }>(),
  normalizeParams: ({ tenantId, documentId }) =>
    resourceParamIdentity(
      { tenantId, documentId },
      `${tenantId}:${documentId}`,
    ),
  load: async ({ documentId }) => ({ id: documentId }),
});

rawDetail.line({ tenantId: "tenant-1", documentId: "document-1" }).summary();

signals.free();


function patchTransportExample(signals: import("./index.js").CallableSignals) {
  return signals.api({}).url("/files/:id").mutation<{ id: string; name: string }, { name: string }, "update">({
    semantics: "update", method: "PATCH", load: async ({ id, body }: { id: string | number; body: { name: string } }) => ({ id: String(id), name: body.name }),
  });
}
void patchTransportExample;
