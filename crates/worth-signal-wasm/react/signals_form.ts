import { useMemo, useRef } from "react";

import { useMaybeReactSignalsStore } from "./context.js";
import {
  commitMultiSelectInput,
  commitTextInput,
  setCheckboxInput,
} from "./form_input_events.js";
import { useSignalValue } from "./hooks.js";
import { createLazyActionBindings, readActionBinding } from "./signals_form_actions.js";

import type {
  ReactSignalsStore,
  RuntimeFormController,
  SignalsFormActionBinding,
  SignalsFormBinding,
  SignalsFormCheckboxBinding,
  SignalsFormFieldBinding,
  SignalsFormFieldState,
  SignalsFormMultiSelectBinding,
  SignalsFormOption,
  SignalsFormSelectBinding,
} from "./model.js";
import type {
  RuntimeFormDeclaration,
  SignalsWithFormLike,
} from "./form_model.js";
import type { FormFieldDeclaration } from "../package/types/forms/core.js";

function readFieldMessages(form: RuntimeFormController, fieldId: string): readonly unknown[] {
  return form.visibleMessages().filter((message) => message.target === fieldId);
}

function readFieldInteraction(form: RuntimeFormController, fieldId: string): unknown | null {
  return form.interaction().fields.find((entry) => entry.field === fieldId) ?? null;
}

function readFieldState<TValue = unknown, TRaw = TValue>(
  form: RuntimeFormController,
  fieldId: string,
): SignalsFormFieldState<TValue, TRaw> {
  const field = form.field(fieldId) as unknown as {
    value(): TValue;
    dirty(): SignalsFormFieldState<TValue, TRaw>["dirty"];
    diagnostics(): SignalsFormFieldState<TValue, TRaw>["diagnostics"];
  };
  const value = field.value();
  const writePosture = form.fieldWritePosture(fieldId);
  const blocked = !Boolean((writePosture as { canWrite?: boolean }).canWrite);
  const state = {
    name: fieldId,
    value,
    disabled: blocked,
    readOnly: blocked,
    field,
    dirty: field.dirty(),
    diagnostics: field.diagnostics(),
    messages: readFieldMessages(form, fieldId),
    interaction: readFieldInteraction(form, fieldId),
    writePosture,
  };
  return Object.freeze(state) as SignalsFormFieldState<TValue, TRaw>;
}

function readFieldBinding<TValue = unknown, TRaw = TValue>(
  form: RuntimeFormController,
  fieldId: string,
  options?: { readonly input?: unknown },
): SignalsFormFieldBinding<TValue, TRaw> {
  const state = readFieldState<TValue, TRaw>(form, fieldId);
  const binding = form.bindInput<TValue, TRaw>(fieldId, options?.input as never);
  return Object.freeze({
    ...state,
    binding,
    onChange(next: unknown) {
      commitTextInput(binding, next);
    },
    onBlur() {
      binding.blur();
    },
    onFocus() {
      binding.focus();
    },
  });
}

function readCheckboxBinding<TValue = boolean>(
  form: RuntimeFormController,
  fieldId: string,
  options?: { readonly input?: unknown },
): SignalsFormCheckboxBinding<TValue> {
  const field = form.field(fieldId) as unknown as {
    value(): TValue;
    dirty(): unknown;
    diagnostics(): unknown;
  };
  const binding = form.bindInput<TValue, boolean>(fieldId, options?.input as never);
  const writePosture = form.fieldWritePosture(fieldId);
  const blocked = !Boolean((writePosture as { canWrite?: boolean }).canWrite);
  const checkboxBinding = {
    name: fieldId,
    checked: Boolean(field.value()),
    disabled: blocked,
    readOnly: blocked,
    dirty: field.dirty(),
    diagnostics: field.diagnostics(),
    messages: readFieldMessages(form, fieldId),
    interaction: readFieldInteraction(form, fieldId),
    writePosture,
    onChange(next: unknown) {
      setCheckboxInput(binding, next);
    },
    onBlur() {
      binding.blur();
    },
    onFocus() {
      binding.focus();
    },
  };
  return Object.freeze(checkboxBinding) as SignalsFormCheckboxBinding<TValue>;
}

function readSelectBinding<TValue = unknown, TRaw = TValue>(
  form: RuntimeFormController,
  fieldId: string,
  fieldOptions: readonly SignalsFormOption<TValue>[],
  options?: { readonly input?: unknown },
): SignalsFormSelectBinding<TValue, TRaw> {
  return Object.freeze({
    ...readFieldBinding<TValue, TRaw>(form, fieldId, options),
    options: fieldOptions,
  });
}

function readMultiSelectBinding<TValue = string>(
  form: RuntimeFormController,
  fieldId: string,
  fieldOptions: readonly SignalsFormOption<TValue>[],
  options?: { readonly input?: unknown },
): SignalsFormMultiSelectBinding<TValue> {
  const field = form.field(fieldId) as unknown as {
    value(): readonly TValue[] | null | undefined;
    dirty(): unknown;
    diagnostics(): unknown;
  };
  const binding = form.bindInput<readonly TValue[], readonly TValue[]>(fieldId, options?.input as never);
  const writePosture = form.fieldWritePosture(fieldId);
  const blocked = !Boolean((writePosture as { canWrite?: boolean }).canWrite);
  const value = field.value() ?? [];
  const multiSelectBinding = {
    name: fieldId,
    value: Array.isArray(value) ? value : [],
    disabled: blocked,
    readOnly: blocked,
    dirty: field.dirty(),
    diagnostics: field.diagnostics(),
    messages: readFieldMessages(form, fieldId),
    interaction: readFieldInteraction(form, fieldId),
    writePosture,
    options: fieldOptions,
    onChange(next: unknown) {
      commitMultiSelectInput(binding, next);
    },
    onBlur() {
      binding.blur();
    },
    onFocus() {
      binding.focus();
    },
  };
  return Object.freeze(multiSelectBinding) as SignalsFormMultiSelectBinding<TValue>;
}

function readActionIds(form: RuntimeFormController): string[] {
  return form.actions().catalog.map((entry) => entry.id);
}

function requireSignalsFormStore<TSignals extends SignalsWithFormLike>(
  explicitStore: ReactSignalsStore<TSignals> | undefined,
  providerStore: ReactSignalsStore | null,
): ReactSignalsStore<TSignals> {
  if (explicitStore) {
    return explicitStore;
  }
  if (providerStore) {
    return providerStore as ReactSignalsStore<TSignals>;
  }
  throw new TypeError(
    "React signals store was not provided. Wrap the tree with <ReactSignalsStoreProvider store={...}> or pass store explicitly.",
  );
}

function requireSignalsFormFactory(signals: SignalsWithFormLike): SignalsWithFormLike["form"] {
  if (typeof signals.form !== "function") {
    throw new TypeError(
      "useSignalsForm(...) requires a worth-signals-wasm signals runtime with signals.form(...) available.",
    );
  }
  return signals.form;
}

export function useSignalsForm<
  TSource = unknown,
  TFields extends Record<string, FormFieldDeclaration> = Record<string, FormFieldDeclaration>,
  TActions extends Record<string, unknown> = Record<string, unknown>,
  TSignals extends SignalsWithFormLike = SignalsWithFormLike,
>(
  declaration: RuntimeFormDeclaration<TSource, TFields> & {
    readonly actions?: TActions | ((...args: never[]) => TActions);
  },
  store?: ReactSignalsStore<TSignals>,
  options?: {
    readonly remountKey?: unknown;
  },
): SignalsFormBinding<
  Extract<keyof TFields, string>,
  Extract<keyof TActions, string>,
  RuntimeFormController<TSource>
> {
  const providerStore = useMaybeReactSignalsStore();
  const resolvedStore = requireSignalsFormStore(store, providerStore);
  const controllerRef = useRef<{
    readonly remountKey: unknown;
    readonly signals: TSignals;
    readonly controller: RuntimeFormController;
  } | null>(null);

  if (
    controllerRef.current === null
    || controllerRef.current.signals !== resolvedStore.signals
    || !Object.is(controllerRef.current.remountKey, options?.remountKey)
  ) {
    controllerRef.current = Object.freeze({
      remountKey: options?.remountKey,
      signals: resolvedStore.signals,
      controller: requireSignalsFormFactory(resolvedStore.signals)(
        declaration as RuntimeFormDeclaration<TSource>,
      ),
    });
  }

  const controller = controllerRef.current.controller as RuntimeFormController<TSource>;
  const summarySnapshot = useSignalValue(controller.summarySignal(), resolvedStore);

  return useMemo(() => {
    const actions = createLazyActionBindings<Extract<keyof TActions, string>>(
      controller,
      readActionIds(controller) as Extract<keyof TActions, string>[],
    );

    return Object.freeze({
      controller,
      source: controller.source(),
      draft: controller.draft(),
      effective: controller.effective(),
      dirty: controller.dirty(),
      patchPlan: controller.patchPlan(),
      readiness: controller.readiness(),
      visibleMessages: controller.visibleMessages(),
      actions,
      fieldState<TValue = unknown, TRaw = TValue>(
        fieldId: Extract<keyof TFields, string>,
      ) {
        return readFieldState<TValue, TRaw>(controller, fieldId);
      },
      field<TValue = unknown, TRaw = TValue>(
        fieldId: Extract<keyof TFields, string>,
        fieldOptions?: { readonly input?: unknown },
      ) {
        return readFieldBinding<TValue, TRaw>(controller, fieldId, fieldOptions);
      },
      checkbox<TValue = boolean>(
        fieldId: Extract<keyof TFields, string>,
        fieldOptions?: { readonly input?: unknown },
      ) {
        return readCheckboxBinding<TValue>(controller, fieldId, fieldOptions);
      },
      select<TValue = unknown, TRaw = TValue>(
        fieldId: Extract<keyof TFields, string>,
        selectOptions: readonly SignalsFormOption<TValue>[],
        fieldOptions?: { readonly input?: unknown },
      ) {
        return readSelectBinding<TValue, TRaw>(controller, fieldId, selectOptions, fieldOptions);
      },
      multiSelect<TValue = string>(
        fieldId: Extract<keyof TFields, string>,
        selectOptions: readonly SignalsFormOption<TValue>[],
        fieldOptions?: { readonly input?: unknown },
      ) {
        return readMultiSelectBinding<TValue>(controller, fieldId, selectOptions, fieldOptions);
      },
      action(actionId: Extract<keyof TActions, string>) {
        return readActionBinding(controller, actionId);
      },
      reset(resetOptions?: { readonly reason?: string }) {
        return controller.reset(resetOptions);
      },
    }) as SignalsFormBinding<
      Extract<keyof TFields, string>,
      Extract<keyof TActions, string>,
      RuntimeFormController<TSource>
    >;
  }, [controller, summarySnapshot]);
}
