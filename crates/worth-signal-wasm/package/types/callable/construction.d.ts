import type { SignalValue } from "../model.js";
import type { Signals } from "../raw_surface.js";
import type { HostCapabilityPlan } from "./host_capabilities.js";
import type { CallableSignals } from "../callable_surface.js";

export type SignalsDeployment = "workerFirst" | "mainThreadCompatibility";

export interface CreateSignalsAssets {
  /**
   * Bundler-emitted URL for `worth_signal_wasm_bg.wasm`.
   * Vite: `import wasmUrl from "worth-signals-wasm/wasm?url"`.
   */
  wasmUrl?: string | URL;
  /**
   * Bundler-emitted URL for the worker-first runtime worker.
   * Required together with `wasmUrl` when `deployment` is `workerFirst`.
   * Vite: `import workerUrl from "worth-signals-wasm/worker?worker&url"`.
   */
  workerUrl?: string | URL;
}

export interface CreateSignalsOptions<TPersistence = SignalValue> {
  deployment?: SignalsDeployment;
  hostCapabilities?: HostCapabilityPlan<TPersistence>;
  /**
   * Explicit asset URLs for bundlers that relocate package modules
   * (for example older Vite optimizeDeps hosts). Optional on Vite 8+ when
   * default relative resolution works; recommended as the portable path.
   */
  assets?: CreateSignalsAssets;
}

export interface WrapSignalsOptions<TPersistence = SignalValue> {
  hostCapabilities?: HostCapabilityPlan<TPersistence>;
}

export interface SignalsCompatibilityRecovery {
  deployment: "mainThreadCompatibility";
  message: string;
}

export type SignalsRuntimeCapabilityName =
  | "callableSurface"
  | "scopedAuthoring"
  | "specNamespace"
  | "workerRuntime";

export type SignalsRuntimeSurfaceFamily =
  | "mainThreadCompatibilityCallable"
  | "mainThreadCompatibilityScoped"
  | "workerFirstCallable"
  | "workerFirstScoped";

export interface SignalsRuntimeCapabilities {
  callableSurface: boolean;
  scopedAuthoring: boolean;
  specNamespace: boolean;
  workerRuntime: boolean;
}

export interface SignalsRuntimeContract {
  surfaceFamily: SignalsRuntimeSurfaceFamily;
  surfaceVersion: "1";
  deployment: SignalsDeployment;
  scopeId: string | null;
  capabilities: Readonly<SignalsRuntimeCapabilities>;
}

export interface SignalsCompatibilityAssertionOptions {
  requires?: ReadonlyArray<SignalsRuntimeCapabilityName>;
}

export interface SignalsConstructionArtifact {
  artifactFamily: "workerUnavailableConstruction" | "signalsConstructionDenied";
  requestedDeployment: SignalsDeployment;
  reason: string;
  message: string;
  compatibilityRecovery: SignalsCompatibilityRecovery;
}

export interface SignalsConstructionExplanation {
  requestedDeployment: SignalsDeployment;
  selectedFamily:
    | "workerFirst"
    | "mainThreadCompatibility"
    | "workerUnavailable"
    | "denied";
  selectedDeployment: SignalsDeployment | null;
  reason: string;
  compatibilityRecovery: SignalsCompatibilityRecovery | null;
}

export function explainCreateSignalsConstruction<TPersistence = SignalValue>(
  options?: CreateSignalsOptions<TPersistence>,
): SignalsConstructionExplanation;

export function planCreateSignalsDeployment<TPersistence = SignalValue>(
  options?: CreateSignalsOptions<TPersistence>,
): { explanation: SignalsConstructionExplanation };

export function createSignals<TPersistence = SignalValue>(options?: CreateSignalsOptions<TPersistence>): Promise<CallableSignals<TPersistence>>;

export function createCallableSignals<TPersistence = SignalValue>(options?: CreateSignalsOptions<TPersistence>): Promise<CallableSignals<TPersistence>>;

export function wrapSignals<TPersistence = SignalValue>(signals: Signals, options?: WrapSignalsOptions<TPersistence>): CallableSignals<TPersistence>;
