import { useMemo } from "react";
import { useResourceLine, useRouterSession } from "worth-signals-wasm/react";
import { check, mountView } from "./observation.jsx";
import { delayedDetails } from "./resource_delivery.jsx";

export async function routerDelivery(signals, store) {
  const loads = delayedDetails(signals);
  const routes = signals.router.define({
    home: signals.router.route("/"),
    user: signals.router.route("/users/:id", { resources: {
      detail: signals.router.resourceLine(loads.family, { params: ({ params }) => ({ id: params.id }) }),
    } }),
  });
  function useRouteView(s) {
    const session = useRouterSession(routes, { store: s, initialLocation: "/users/a" });
    const href = session.currentRoute?.href ?? null;
    const selection = useMemo(() => href?.startsWith("/users/")
      ? { id: href.slice("/users/".length) } : null, [href]);
    const detail = useResourceLine(loads.family, selection, s);
    return {
      value: { href, status: detail.status?.kind ?? null, name: detail.value?.name ?? null },
      controls: session,
    };
  }
  const view = await mountView(store, useRouteView,
    { href: "/users/a", status: "pending", name: null }, { signalSubscription: false });
  try {
    const moved = view.waitForValue({ href: "/users/b", status: "pending", name: null });
    const navigation = view.controls().navigate("/users/b");
    await moved;
    const report = await navigation;
    check(report.outcome().kind === "admitted", "navigation did not cross admission");
    check(report.outcome().route().resource("detail").line() === loads.family.line({ id: "b" }),
      "route binding and React must reuse one resource line");
    const fulfilled = { href: "/users/b", status: "fulfilled", name: "B" };
    const ready = view.waitForValue(fulfilled);
    loads.complete("b", "B");
    await ready;
    const unchanged = view.stayAt(fulfilled);
    loads.complete("a", "Late A");
    await unchanged;
    const back = view.waitForValue({ href: "/users/a", status: "fulfilled", name: "Late A" });
    await view.controls().navigate("/users/a", { historyMethod: "pop" });
    await back;
    const home = view.waitForValue({ href: "/", status: null, name: null });
    await view.controls().navigate("/", { historyMethod: "replace" });
    await home;
    return { initialRoute: true, push: true, pop: true, replace: true,
      boundResource: true, latePreviousRouteIsolated: true };
  } finally {
    view.close();
    await signals.settleAuthoredWork();
  }
}

export async function deniedNavigation(signals, store) {
  const auth = signals.router.host.string("auth");
  const authenticated = signals.router.prerequisite("authenticated", {
    consumes: [auth],
    evaluate: ({ consume, allow, redirect }) => consume(auth) === "signedIn"
      ? allow({ reason: "authenticated" }) : redirect({ href: "/login", reason: "signInRequired" }),
  });
  const routes = signals.router.define({
    home: signals.router.route("/"),
    private: signals.router.route("/private", { admission: [authenticated] }),
  });
  const view = await mountView(store, s => {
    const session = useRouterSession(routes, { store: s, initialLocation: "/" });
    return { value: { href: session.currentRoute?.href ?? null,
      outcome: session.story.events.at(-1)?.outcomeKind ?? null }, controls: session };
  }, { href: "/", outcome: "admitted" }, { signalSubscription: false });
  try {
    const redirected = view.waitForValue({ href: "/", outcome: "redirect" });
    const denied = await view.controls().navigate("/private", { facts: { auth: "anonymous" } });
    check(denied.outcome().kind === "redirect", "unauthenticated navigation was admitted");
    await redirected;
    const admitted = view.waitForValue({ href: "/private", outcome: "admitted" });
    await view.controls().navigate("/private", { facts: { auth: "signedIn" } });
    await admitted;
    const missing = view.waitForValue({ href: "/private", outcome: "notFound" });
    const report = await view.controls().navigate("/missing");
    check(report.outcome().kind === "notFound", "missing route did not return notFound");
    await missing;
    return { redirectVisible: true, subsequentAdmission: true, notFoundPreservesRoute: true };
  } finally { view.close(); }
}
