import type {
  RuntimeFormController,
  SignalsFormActionBinding,
} from "./model.js";

export function readActionBinding(form: RuntimeFormController, actionId: string): SignalsFormActionBinding {
  const plan = form.actionPlan(actionId);
  const debug = form.debugAction(actionId);
  const latestExecution = debug.latestExecution;
  return Object.freeze({
    plan,
    debug,
    disabled: plan.status !== "accepted" || !plan.readiness.canRun || debug.pending,
    pending: debug.pending,
    latestExecution,
    resultKind: readExecutionResultKind(latestExecution),
    execute() {
      return form.executeAction(actionId);
    },
  });
}

export function readExecutionResultKind(execution: unknown): string | null {
  if (!execution || typeof execution !== "object" || !("resultKind" in execution)) {
    return null;
  }
  return typeof execution.resultKind === "string" ? execution.resultKind : null;
}

/**
 * One enumerable, memoized getter per action id. A component that renders
 * one action's button reads one binding; the other actions' plans and debug
 * reports are never built. Each binding is built at most once per call of
 * this function (so once per form snapshot when used from `useMemo`).
 * Cost: O(actions) to define the getters, O(1) plan + debug read per
 * accessed action.
 */
export function createLazyActionBindings<TActionIds extends string>(
  form: RuntimeFormController,
  actionIds: readonly TActionIds[],
): Readonly<Record<TActionIds, SignalsFormActionBinding>> {
  const bindings: Record<string, SignalsFormActionBinding> = {};
  for (const actionId of actionIds) {
    let binding: SignalsFormActionBinding | null = null;
    Object.defineProperty(bindings, actionId, {
      enumerable: true,
      get() {
        if (binding === null) {
          binding = readActionBinding(form, actionId);
        }
        return binding;
      },
    });
  }
  return Object.freeze(bindings) as Readonly<Record<TActionIds, SignalsFormActionBinding>>;
}
