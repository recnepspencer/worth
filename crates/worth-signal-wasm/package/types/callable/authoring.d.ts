import type {
  AspectId, InputOptions, ComputedSpec, OutputSpec, RunSummary, SignalValue,
} from "../model.js";

declare const WorthSignalBrand: unique symbol;

declare const WorthSignalInputBrand: unique symbol;

declare const WorthSignalComputedBrand: unique symbol;

declare const WorthSignalOutputBrand: unique symbol;

export interface Signal<T = SignalValue> {
  (): T;
  get(): T;
  value(): T;
  free(): void;
  [Symbol.dispose](): void;
  readonly id: string;
  readonly debugName: string | null;
  readonly [WorthSignalBrand]: "signal";
}

export interface NamedComputedCallbackDefinition<T = SignalValue> {
  compute: () => T;
}

export interface NamedOutputCallbackDefinition<T = SignalValue> {
  compute: () => T;
}

type PatchSignalValue<T> = T extends ReadonlyArray<infer U>
  ? ReadonlyArray<U>
  : T extends Array<infer U>
    ? Array<U>
    : T extends object
      ? Partial<T>
      : never;

type AssignSignalValue<T> = T extends ReadonlyArray<unknown>
  ? never
  : T extends Array<unknown>
    ? never
    : T extends object
      ? Partial<T>
      : never;

export interface InputSignalHandle<T = SignalValue> extends Signal<T> {
  set(value: T): RunSummary | Promise<RunSummary>;
  reset(): RunSummary | Promise<RunSummary>;
  patch(value: PatchSignalValue<T>): RunSummary | Promise<RunSummary>;
  assign(fields: AssignSignalValue<T>): RunSummary | Promise<RunSummary>;
  readonly [WorthSignalInputBrand]: "input";
}

export interface LinkedSignalPrevious<TValue = SignalValue, TSource = TValue> {
  readonly value: TValue;
  readonly source: TSource;
}

export interface LinkedSignalOptions {
  debugName?: string;
}

export interface LinkedIdentitySignalDefinition<TSource = SignalValue> {
  source: () => TSource;
  debugName?: string;
}

export interface LinkedComputedSignalDefinition<TSource = SignalValue, TValue = SignalValue> {
  source: () => TSource;
  computation: (
    source: TSource,
    previous: LinkedSignalPrevious<TValue, TSource> | null,
  ) => TValue;
  debugName?: string;
}

export type LinkedSignalDefinition<TSource = SignalValue, TValue = SignalValue> =
  | LinkedIdentitySignalDefinition<TSource>
  | LinkedComputedSignalDefinition<TSource, TValue>;

export interface LinkedSignalHandle<TValue = SignalValue, TSource = TValue>
  extends InputSignalHandle<TValue> {
  relink(): RunSummary | Promise<RunSummary>;
}

export interface ComputedSignalHandle<T = SignalValue> extends Signal<T> {
  readonly [WorthSignalComputedBrand]: "computed";
}

export interface OutputSignalHandle<T = SignalValue> extends Signal<T> {
  readonly [WorthSignalOutputBrand]: "output";
}

export interface InputAuthoringOptions extends InputOptions {
  debugName?: string;
}

export interface SignalAuthoringOptions {
  debugName?: string;
}

export interface CallbackSignalAuthoringOptions {
  debugName?: string;
}

export interface ExplicitSignalSpecNamespace {
  input<T = SignalValue>(id: string, initial: T, options?: InputAuthoringOptions): InputSignalHandle<T>;
  computed<T = SignalValue>(
    id: string,
    spec: ComputedSpec | NamedComputedCallbackDefinition<T>,
    options?: SignalAuthoringOptions,
  ): ComputedSignalHandle<T>;
  computedCallback<T = SignalValue>(
    id: string,
    compute: () => T,
    options?: CallbackSignalAuthoringOptions,
  ): ComputedSignalHandle<T>;
  output<T = SignalValue>(
    id: string,
    spec: OutputSpec | NamedOutputCallbackDefinition<T>,
    options?: SignalAuthoringOptions,
  ): OutputSignalHandle<T>;
  outputCallback<T = SignalValue>(
    id: string,
    compute: () => T,
    options?: CallbackSignalAuthoringOptions,
  ): OutputSignalHandle<T>;
}

export interface CallableSignalsTransaction {
  set<T = SignalValue>(input: InputSignalHandle<T>, value: T): void;
  patch<T = SignalValue>(input: InputSignalHandle<T>, value: PatchSignalValue<T>): void;
  setWithAspects<T = SignalValue>(input: InputSignalHandle<T>, value: T, aspects: ReadonlyArray<AspectId>): void;
  setWithRegions<T = SignalValue>(input: InputSignalHandle<T>, value: T, changedRegions: unknown): void;
  setWithRegionsAndAspects(
    input: InputSignalHandle,
    value: SignalValue,
    changedRegions: unknown,
    aspects: ReadonlyArray<AspectId>,
  ): void;
  free(): void;
  [Symbol.dispose](): void;
}

export interface AsyncCallableSignalsTransaction {
  set<T = SignalValue>(input: InputSignalHandle<T>, value: T): void;
  patch<T = SignalValue>(input: InputSignalHandle<T>, value: PatchSignalValue<T>): void;
  setWithAspects<T = SignalValue>(input: InputSignalHandle<T>, value: T, aspects: ReadonlyArray<AspectId>): void;
  setWithRegions<T = SignalValue>(input: InputSignalHandle<T>, value: T, changedRegions: unknown): void;
  setWithRegionsAndAspects(
    input: InputSignalHandle,
    value: SignalValue,
    changedRegions: unknown,
    aspects: ReadonlyArray<AspectId>,
  ): void;
  free(): void;
  [Symbol.dispose](): void;
}
