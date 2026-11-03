import assert from "node:assert/strict";
import test from "node:test";
import { createRealRequestRuntime } from "../../runtime_fixture/real_request_runtime.mjs";


test("PATCH preserves its transport method and compiled mutation reconciliation proof", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const read = signals.api({}).url("/files/:id").detail({ load: ({ id }) => ({ id, name: "Before" }) });
    const resident = read.line({ id: "one" });
    const rename = signals.api({}).url("/files/:id").response(signals.resource.response.detail()()).mutation({
      semantics: "update", method: "PATCH",
      reconciles: [{ family: read, params: ({ id }) => ({ id }), fallback: "refetchRequired", detail: { kind: "replace" } }],
      load: ({ id, body }) => ({ id, name: body.name }),
    });
    const line = rename.line({ id: "one", body: { name: "After" } });
    assert.equal(line.request().method, "PATCH");
    assert.equal(line.mutationResponse().method, "PATCH");
    assert.equal(resident.value().name, "After");
    const observationalPosture = signals.api({}).url("/files/:id").body().verb("PATCH").detail({ load: ({ id }) => ({ id }) });
    assert.equal(observationalPosture.line({ id: "two", body: {} }).request().method, "PATCH");
    assert.throws(() => signals.api({}).url("/files").verb("TRACE"), /must be/);
  } finally { await runtime.cleanup(); }
});
