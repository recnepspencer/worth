import { createSignals } from "./index.js";
import type {
  CallableSignals,
  FormController,
  RouterBrowserHistoryStory,
} from "./index.js";

interface Project {
  id: string;
  name: string;
}

declare const form: FormController;
declare const admissionFacts: Readonly<Record<string, unknown>>;

function defineOverviewRoutes(routerSignals: CallableSignals) {
  return routerSignals.router.define({
    home: routerSignals.router.route("/"),
    projectDetail: routerSignals.router.route("/projects/:projectId", {
      search: { tab: routerSignals.router.search.optional.string() },
    }),
  });
}

type OverviewRoutes = ReturnType<typeof defineOverviewRoutes>;

function overviewProjectHref(routes: OverviewRoutes, projectId: string) {
  return routes.projectDetail.href({
    params: { projectId },
    search: { tab: "files" },
  });
}

function defineOverviewAccess(routerSignals: CallableSignals) {
  const sessionReady = routerSignals.router.host.boolean("sessionReady");
  return {
    sessionReady,
    prerequisite: routerSignals.router.prerequisite("signedIn", {
      consumes: [sessionReady] as const,
      evaluate: (context) => context.consume(sessionReady)
        ? context.allow({ reason: "session admitted" })
        : context.redirect({ href: "/sign-in", reason: "sign in required" }),
    }),
  };
}

function defineOverviewRouting(routerSignals: CallableSignals) {
  const access = defineOverviewAccess(routerSignals);
  const app = routerSignals.router.route("/app", {
    breadcrumb: routerSignals.router.breadcrumb({ id: "app", label: "App" }),
  });
  return {
    access,
    routes: routerSignals.router.define({
      home: routerSignals.router.route("/"),
      signIn: routerSignals.router.route("/sign-in"),
      app: routerSignals.router.layout(app, { outlet: "main" }, {
        projectDetail: routerSignals.router.route("/app/projects/:projectId", {
          admission: [access.prerequisite],
          breadcrumb: routerSignals.router.breadcrumb({
            id: "project",
            label: ({ params }) => `Project ${params.projectId}`,
          }),
        }),
      }),
    }),
  };
}

type OverviewRouting = ReturnType<typeof defineOverviewRouting>;

async function createOverviewNavigationSession(
  routerSignals: CallableSignals,
  routing: OverviewRouting,
  href: string,
  sessionReady: boolean,
) {
  const ingress = routerSignals.router.browserHistory.load(href);
  const report = await routing.routes.admitBrowserHistoryIngress(
    ingress,
    { sessionReady },
  );
  const story = routerSignals.router.browserHistory.story(report);
  return {
    report,
    story,
    current: story.current(),
    explanation: story.auditability().summary(),
  };
}

const signals = await createSignals({ deployment: "mainThreadCompatibility" });
const overviewRoutes = defineOverviewRoutes(signals);
overviewProjectHref(overviewRoutes, "p7") satisfies string;
const overviewRouting = defineOverviewRouting(signals);
void createOverviewNavigationSession(signals, overviewRouting, "/", true);
const api = signals.api({ baseUrl: "/api" });
const projectFamily = api.url("/projects/:projectId").detail<Project>({
  load: async ({ projectId }) => ({ id: String(projectId), name: "Project" }),
});

const signedIn = signals.router.host.boolean("signedIn");
const projectAvailable = signals.router.resource.boolean("projectAvailable");
const mayOpenProject = signals.router.prerequisite("may-open-project", {
  consumes: [signedIn, projectAvailable] as const,
  evaluate: ({ consume, allow, redirect, notFound }) => {
    if (!consume(signedIn)) {
      return redirect({ href: "/sign-in", reason: "signInRequired" });
    }
    return consume(projectAvailable)
      ? allow({ reason: "projectAvailable" })
      : notFound({ reason: "projectMissing" });
  },
});

const recoverDeletedProject = signals.router.recovery(
  "recover-deleted-project",
  ({ terminalArtifact, fallback }) => terminalArtifact.kind === "notFound"
    ? fallback({ href: "/app/projects", reason: "projectMissing" })
    : null,
);

const appRoute = signals.router.route("/app", {
  breadcrumb: signals.router.breadcrumb({ id: "app", label: "App" }),
});
const projectResource = signals.router.resourceLine(projectFamily, {
  params: ({ params }) => ({ projectId: params.projectId }),
  prefetch: "intent",
});

const routes = signals.router.define({
  home: signals.router.route("/"),
  signIn: signals.router.route("/sign-in"),
  app: signals.router.layout(appRoute, { outlet: "main" }, {
    projects: signals.router.route("/app/projects", {
      breadcrumb: signals.router.breadcrumb({
        id: "projects",
        label: "Projects",
      }),
    }),
    projectDetail: signals.router.route("/app/projects/:projectId", {
      search: { tab: signals.router.search.optional.string() },
      admission: [mayOpenProject],
      recovery: [recoverDeletedProject],
      forms: signals.router.forms("project-form", { continuity: "defer" }),
      resources: { detail: projectResource },
      breadcrumb: signals.router.breadcrumb({
        id: "project",
        label: ({ params }) => `Project ${params.projectId}`,
        parent: signals.router.breadcrumbParent({ carry: true }),
      }),
    }),
  }),
});

const location = routes.app.projectDetail.to({
  params: { projectId: "p7" },
  search: { tab: "files" },
});
location.href satisfies string;
location.plan().explain();

const candidate = routes.project(location.href);
if (candidate) {
  candidate.route().breadcrumbTrail();
  candidate.layouts();
  candidate.outlets();
  candidate.admission(admissionFacts).provenance();

  const preview = candidate.prefetch("intent");
  preview.resources();
  preview.free();
}

const outcome = await routes.admit(location.href, admissionFacts);
outcome.diagnostics();
outcome.provenance();

if (outcome.kind === "admitted") {
  const formsAuthority = outcome.route().formsAuthority();
  if (formsAuthority) form.reportRouteAuthority(formsAuthority);
  outcome.route().resource("detail").current();
  await routes.transition(outcome, "/app/projects", {
    continuity: "preserve-visible-while-pending",
  });
}

const ingress = signals.router.browserHistory.load(location.href, {
  routeIdentity: routes.app.projectDetail.descriptor().routeId,
});
const ingressReport = await routes.admitBrowserHistoryIngress(
  ingress,
  admissionFacts,
);
const story: RouterBrowserHistoryStory =
  signals.router.browserHistory.story(ingressReport);
story.current();
story.inspection();
story.auditability();

const writeback = signals.router.browserHistory.writeback.push(location, {
  routeIdentity: location.routeId,
});
const writebackReport = await routes.applyBrowserHistoryWriteback(
  writeback,
  admissionFacts,
);
story.record(writebackReport);

signals.free();
