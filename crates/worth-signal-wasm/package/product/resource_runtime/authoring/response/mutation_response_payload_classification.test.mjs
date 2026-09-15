import assert from "node:assert/strict";
import test from "node:test";

import { createRealRequestRuntime } from "../../runtime_fixture/real_request_runtime.mjs";

function declareCreateUser(runtime, load) {
  const userDetail = runtime.signals.api({}).url("/users/:userId").detail({
    load: ({ userId }) => ({ id: userId, name: "Loaded" }),
  });
  const createUser = runtime.signals.api({}).url("/users")
    .response(runtime.signals.resource.response.detail()())
    .create({
      reconciles: [
        {
          family: userDetail,
          params: ({ body }) => ({ userId: body.id }),
          fallback: "refetchRequired",
          detail: { kind: "replace" },
        },
      ],
      load,
    });
  return { userDetail, createUser };
}

test("mutation response payload digest honors toJSON so Date and JSON-aware class values digest as their wire shape", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    class Money {
      constructor(cents) {
        this.cents = cents;
        this.currency = "USD";
      }
      toJSON() {
        return { cents: this.cents, currency: this.currency };
      }
    }
    const { createUser } = declareCreateUser(runtime, ({ body }) => ({
      id: body.id,
      name: body.name,
      generatedAt: body.generatedAt,
      price: body.price,
    }));

    const first = createUser.line({
      body: {
        id: "u1",
        name: "Created",
        generatedAt: new Date("2026-09-15T12:00:00.000Z"),
        price: new Money(1250),
      },
    });
    const second = createUser.line({
      body: {
        id: "u2",
        name: "Created",
        generatedAt: "2026-09-15T12:00:00.000Z",
        price: { cents: 1250, currency: "USD" },
      },
    });
    const later = createUser.line({
      body: {
        id: "u3",
        name: "Created",
        generatedAt: new Date("2026-09-15T12:00:01.000Z"),
        price: new Money(1250),
      },
    });

    assert.equal(first.status().kind, "fulfilled");
    assert.equal(second.status().kind, "fulfilled");
    assert.equal(later.status().kind, "fulfilled");
    // The committed value is the wire shape, not the class instance.
    assert.deepEqual(first.value(), {
      id: "u1",
      name: "Created",
      generatedAt: "2026-09-15T12:00:00.000Z",
      price: { cents: 1250, currency: "USD" },
    });
    assert.deepEqual({ ...second.value(), id: "u1" }, first.value());
    const digestOf = (line) =>
      line.mutationResponse().response.payloadDigest.replace(/"id":"u\d"/, "");
    assert.equal(digestOf(first), digestOf(second));
    assert.notEqual(digestOf(first), digestOf(later));
  } finally {
    await runtime.cleanup();
  }
});

test("a synchronous response the payload digest cannot represent settles the line rejected with the reason", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { createUser, userDetail } = declareCreateUser(runtime, ({ body }) => ({
      id: body.id,
      name: body.name,
      tags: new Map([["role", "admin"]]),
    }));

    const line = createUser.line({ body: { id: "u1", name: "Created" } });

    assert.deepEqual(line.status(), {
      kind: "rejected",
      operation: "initialLoad",
      message:
        "resource line value cannot represent Map at $value.tags: "
        + "JSON serializes it without its contents, so the line could not hold it faithfully",
      continuity: "noVisibleValueYet",
    });
    assert.equal(line.value(), null);
    assert.equal(line.mutationResponse(), null);
    // A synchronously settled line records one "materialized" entry that
    // carries its settled state, for a rejection exactly as for a fulfilment.
    assert.deepEqual(
      line.history().lifecycle.map((entry) => [entry.event, entry.status.kind]),
      [["materialized", "rejected"]],
    );
    // The refused response reconciled nothing: the target family's line for
    // that id loads from its own source when first materialized.
    assert.deepEqual(userDetail.line({ userId: "u1" }).value(), { id: "u1", name: "Loaded" });
  } finally {
    await runtime.cleanup();
  }
});

test("an asynchronous response the payload digest cannot represent settles rejected instead of staying pending", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { createUser } = declareCreateUser(runtime, async ({ body }) => ({
      id: body.id,
      name: body.name,
      attachments: new Set(["a.png"]),
    }));

    const execution = createUser.execute({ body: { id: "u1", name: "Created" } }, { freeOnSettle: false });
    assert.equal(execution.line.status().kind, "pending");

    const settled = await execution.settled({ timeoutMs: 1_000 });

    assert.equal(settled.resultKind, "rejected");
    assert.equal(settled.status.kind, "rejected");
    assert.match(settled.status.message, /cannot represent Set at \$value\.attachments/);
    assert.equal(execution.line.status().kind, "rejected");
    assert.deepEqual(execution.line.freshness(), {
      kind: "stale",
      reason: "initialLoadRejected",
    });
    execution.free();
  } finally {
    await runtime.cleanup();
  }
});

test("every JSON-unrepresentable value class is refused by name instead of being committed as empty", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const cases = [
      [new WeakMap(), "WeakMap"],
      [new WeakSet(), "WeakSet"],
      [Promise.resolve(1), "Promise"],
      [/re/, "RegExp"],
      [new RangeError("bad"), "RangeError"],
      [new ArrayBuffer(4), "ArrayBuffer"],
      [new Uint8Array(2), "Uint8Array"],
      [new DataView(new ArrayBuffer(2)), "DataView"],
      [new Number(1), "boxed number"],
      [new String("s"), "boxed string"],
    ];
    for (const [value, expectedName] of cases) {
      const { createUser } = declareCreateUser(runtime, ({ body }) => ({
        id: body.id,
        name: body.name,
        extra: value,
      }));
      const line = createUser.line({ body: { id: "u1", name: "Created" } });
      assert.equal(line.status().kind, "rejected", expectedName);
      assert.match(
        line.status().message,
        new RegExp(`cannot represent ${expectedName} at \\$value\\.extra`),
      );
    }
  } finally {
    await runtime.cleanup();
  }
});

test("toJSON is consulted once per value, so a toJSON that returns itself cannot recurse", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const selfReturning = {
      label: "self",
      toJSON() {
        return this;
      },
    };
    const { createUser } = declareCreateUser(runtime, ({ body }) => ({
      id: body.id,
      name: body.name,
      extra: selfReturning,
    }));
    const line = createUser.line({ body: { id: "u1", name: "Created" } });
    assert.equal(line.status().kind, "fulfilled");
    assert.match(
      line.mutationResponse().response.payloadDigest,
      /"extra":\{"label":"self"\}/,
    );
  } finally {
    await runtime.cleanup();
  }
});
