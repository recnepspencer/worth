import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { dirname, extname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

import { loadSignalsModule } from "../signals_runtime/module_loading/load_signals_module.mjs";

const packageDirectory = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const routerDocsDirectory = resolve(packageDirectory, "../docs/router");

const canonicalDocuments = [
  "index.md",
  "projection/route_schema_authoring.md",
  "projection/projected_candidates.md",
  "admission/admit.md",
  "transitions/transition_artifacts.md",
  "history/browser_history_story.md",
  "recovery/stale_deep_link_recovery.md",
  "breadcrumbs/breadcrumb_declarations.md",
  "resources/route_resource_declarations.md",
  "forms/route_authority_handoff.md",
  "runtime_placement/worker_first_default.md",
  "diagnostics/diagnostics_surfaces.md",
];

function markdownFiles(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = resolve(directory, entry.name);
    return entry.isDirectory() ? markdownFiles(path) : extname(path) === ".md" ? [path] : [];
  });
}

function localMarkdownLinks(source) {
  return [...source.matchAll(/\[[^\]]+\]\(([^)]+\.md(?:#[^)]+)?)\)/g)]
    .map((match) => match[1].split("#", 1)[0])
    .filter((target) => !target.startsWith("http"));
}

test("Router documentation has one complete, link-safe learning spine", () => {
  for (const relativePath of canonicalDocuments) {
    const path = resolve(routerDocsDirectory, relativePath);
    assert.equal(existsSync(path), true, `missing canonical Router doc: ${relativePath}`);
    assert.ok(readFileSync(path, "utf8").length > 500, `thin canonical Router doc: ${relativePath}`);
  }

  for (const path of markdownFiles(routerDocsDirectory)) {
    const source = readFileSync(path, "utf8");
    for (const target of localMarkdownLinks(source)) {
      assert.equal(
        existsSync(resolve(dirname(path), target)),
        true,
        `broken Router doc link in ${path}: ${target}`,
      );
    }
  }
});

test("Router documentation does not teach internal or nonexistent APIs", () => {
  const source = markdownFiles(routerDocsDirectory)
    .map((path) => readFileSync(path, "utf8"))
    .join("\n");

  assert.doesNotMatch(source, /bridge\.browserHistoryStory/);
  assert.doesNotMatch(source, /bridge\.admitBrowserHistoryIngress/);
  assert.doesNotMatch(source, /router\.navigate\s*\(/);
  assert.doesNotMatch(source, /createHistoryStub\s*\(/);
  assert.doesNotMatch(source, /runtime executes in (?:a |the )?worker/i);
  assert.match(source, /The host still owns the browser/i);
  assert.match(source, /Projection.*not permission/is);
});

test("Router published type examples compile against the package surface", () => {
  const tsc = resolve(packageDirectory, "../node_modules/typescript/bin/tsc");
  const smoke = resolve(packageDirectory, "documentation-router-types-smoke.ts");
  const result = spawnSync(process.execPath, [
    tsc,
    "--noEmit",
    "--target", "ES2022",
    "--module", "ES2022",
    "--moduleResolution", "bundler",
    "--strict",
    "--skipLibCheck",
    "--lib", "ES2023,DOM,DOM.Iterable,ESNext.Disposable",
    smoke,
    "--pretty", "false",
  ], { encoding: "utf8" });

  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
});

test("Router guide flow executes against the public compatibility runtime", async () => {
  const { createSignals, cleanup } = await loadSignalsModule({ rawSurface: "real" });
  const signals = await createSignals({ deployment: "mainThreadCompatibility" });

  try {
    const signedIn = signals.router.host.boolean("signedIn");
    const requireSignIn = signals.router.prerequisite("require-sign-in", {
      consumes: [signedIn],
      evaluate: ({ consume, allow, redirect }) => consume(signedIn)
        ? allow({ reason: "signedIn" })
        : redirect({ href: "/sign-in", reason: "signInRequired" }),
    });
    const appRoute = signals.router.route("/app");
    const routes = signals.router.define({
      signIn: signals.router.route("/sign-in"),
      app: signals.router.layout(appRoute, { outlet: "main" }, {
        projectDetail: signals.router.route("/app/projects/:projectId", {
          search: { tab: signals.router.search.optional.string() },
          admission: [requireSignIn],
          breadcrumb: signals.router.breadcrumb({
            id: "project",
            label: ({ params }) => `Project ${params.projectId}`,
          }),
        }),
      }),
    });
    const location = routes.app.projectDetail.to({
      params: { projectId: "p7" },
      search: { tab: "files" },
    });

    assert.equal(location.href, "/app/projects/p7?tab=files");
    assert.equal(routes.project(location.href)?.routeId, routes.app.projectDetail.descriptor().routeId);
    assert.equal((await routes.admit(location.href, { signedIn: false })).kind, "redirect");

    const admitted = await routes.admit(location.href, { signedIn: true });
    assert.equal(admitted.kind, "admitted");
    assert.equal(admitted.route().breadcrumb()?.label, "Project p7");

    const ingress = signals.router.browserHistory.load(location.href, {
      routeIdentity: location.routeId,
    });
    const ingressReport = await routes.admitBrowserHistoryIngress(ingress, { signedIn: true });
    const story = signals.router.browserHistory.story(ingressReport);
    assert.equal(story.current()?.href, location.href);

    const writeback = signals.router.browserHistory.writeback.replace(location, {
      routeIdentity: location.routeId,
    });
    story.record(await routes.applyBrowserHistoryWriteback(writeback, { signedIn: true }));
    assert.equal(story.events().length, 2);
    assert.equal(story.auditability().summary().currentHref, location.href);
  } finally {
    signals.free();
    await cleanup();
  }
});
