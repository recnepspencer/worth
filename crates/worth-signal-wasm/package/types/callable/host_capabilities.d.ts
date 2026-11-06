import type { RunSummary, SignalValue } from "../model.js";
import type { HostCapabilityCompatibility } from "../diagnostics.js";

declare const WorthSignalHostCapabilityPlanBrand: unique symbol;

declare const WorthSignalViewportCapabilityRegistrationBrand: unique symbol;

declare const WorthSignalViewportCapabilityHandleBrand: unique symbol;

declare const WorthSignalVisibilityCapabilityRegistrationBrand: unique symbol;

declare const WorthSignalVisibilityCapabilityHandleBrand: unique symbol;

declare const WorthSignalOnlineCapabilityRegistrationBrand: unique symbol;

declare const WorthSignalOnlineCapabilityHandleBrand: unique symbol;

declare const WorthSignalClockCapabilityRegistrationBrand: unique symbol;

declare const WorthSignalClockCapabilityHandleBrand: unique symbol;

declare const WorthSignalPersistenceCapabilityRegistrationBrand: unique symbol;

declare const WorthSignalPersistenceCapabilityValueBrand: unique symbol;

declare const WorthSignalPersistenceCapabilityHandleBrand: unique symbol;

export type HostCapabilitySubscription =
  | void
  | (() => void)
  | { dispose(): void }
  | { free(): void };

export interface ViewportCapabilityState {
  width: number;
  height: number;
}

export interface ViewportCapabilitySource {
  current(): ViewportCapabilityState;
  subscribe(listener: () => void): HostCapabilitySubscription;
}

export interface ViewportCapabilityOptions {
  source: ViewportCapabilitySource;
  compatibility?: HostCapabilityCompatibility;
}

export interface ViewportCapabilityRegistration {
  readonly family: "viewport";
  readonly compatibility: HostCapabilityCompatibility;
  readonly [WorthSignalViewportCapabilityRegistrationBrand]: "viewportCapabilityRegistration";
}

export interface VisibilityCapabilitySource {
  current(): boolean | "visible" | "hidden";
  subscribe(listener: () => void): HostCapabilitySubscription;
}

export interface VisibilityCapabilityOptions {
  source: VisibilityCapabilitySource;
  compatibility?: HostCapabilityCompatibility;
}

export interface VisibilityCapabilityRegistration {
  readonly family: "visibility";
  readonly compatibility: HostCapabilityCompatibility;
  readonly [WorthSignalVisibilityCapabilityRegistrationBrand]: "visibilityCapabilityRegistration";
}

export interface OnlineCapabilitySource {
  current(): boolean | "online" | "offline";
  subscribe(listener: () => void): HostCapabilitySubscription;
}

export interface OnlineCapabilityOptions {
  source: OnlineCapabilitySource;
  compatibility?: HostCapabilityCompatibility;
}

export interface OnlineCapabilityRegistration {
  readonly family: "online";
  readonly compatibility: HostCapabilityCompatibility;
  readonly [WorthSignalOnlineCapabilityRegistrationBrand]: "onlineCapabilityRegistration";
}

export interface ClockCapabilitySource {
  current(): number;
}

export interface ClockCapabilityOptions {
  source: ClockCapabilitySource;
  pollMs?: number;
  compatibility?: HostCapabilityCompatibility;
}

export interface ClockCapabilityRegistration {
  readonly family: "clock";
  readonly compatibility: HostCapabilityCompatibility;
  readonly pollMs: number;
  readonly [WorthSignalClockCapabilityRegistrationBrand]: "clockCapabilityRegistration";
}

export interface PersistenceCapabilitySource<T = SignalValue> {
  current(): T;
}

export interface PersistenceCapabilityOptions<T = SignalValue> {
  source: PersistenceCapabilitySource<T>;
  compatibility?: HostCapabilityCompatibility;
}

export interface PersistenceCapabilityRegistration<T = SignalValue> {
  readonly family: "persistence";
  readonly compatibility: HostCapabilityCompatibility;
  readonly [WorthSignalPersistenceCapabilityRegistrationBrand]: "persistenceCapabilityRegistration";
  readonly [WorthSignalPersistenceCapabilityValueBrand]?: T;
}

export interface HostCapabilityPlanInput<TPersistence = SignalValue> {
  viewport?: ViewportCapabilityRegistration;
  visibility?: VisibilityCapabilityRegistration;
  online?: OnlineCapabilityRegistration;
  clock?: ClockCapabilityRegistration;
  persistence?: PersistenceCapabilityRegistration<TPersistence>;
}

export interface HostCapabilityPlan<TPersistence = SignalValue> {
  readonly viewport?: ViewportCapabilityRegistration;
  readonly visibility?: VisibilityCapabilityRegistration;
  readonly online?: OnlineCapabilityRegistration;
  readonly clock?: ClockCapabilityRegistration;
  readonly persistence?: PersistenceCapabilityRegistration<TPersistence>;
  readonly [WorthSignalHostCapabilityPlanBrand]: "hostCapabilityPlan";
}

export interface ViewportCapabilityDescriptor {
  family: "viewport";
  compatibility: HostCapabilityCompatibility;
  registrationId: "viewport";
}

export interface HostViewportCapability {
  size(): ViewportCapabilityState;
  width(): number;
  height(): number;
  descriptor(): ViewportCapabilityDescriptor;
  readonly [WorthSignalViewportCapabilityHandleBrand]: "viewportCapabilityHandle";
}

export interface VisibilityCapabilityDescriptor {
  family: "visibility";
  compatibility: HostCapabilityCompatibility;
  registrationId: "visibility";
}

export interface HostVisibilityCapability {
  state(): "visible" | "hidden";
  isVisible(): boolean;
  descriptor(): VisibilityCapabilityDescriptor;
  readonly [WorthSignalVisibilityCapabilityHandleBrand]: "visibilityCapabilityHandle";
}

export interface OnlineCapabilityDescriptor {
  family: "online";
  compatibility: HostCapabilityCompatibility;
  registrationId: "online";
}

export interface HostOnlineCapability {
  state(): "online" | "offline";
  isOnline(): boolean;
  descriptor(): OnlineCapabilityDescriptor;
  readonly [WorthSignalOnlineCapabilityHandleBrand]: "onlineCapabilityHandle";
}

export interface ClockCapabilityDescriptor {
  family: "clock";
  compatibility: HostCapabilityCompatibility;
  registrationId: "clock";
}

export interface HostClockCapability {
  now(): number;
  descriptor(): ClockCapabilityDescriptor;
  readonly [WorthSignalClockCapabilityHandleBrand]: "clockCapabilityHandle";
}

export interface PersistenceCapabilityDescriptor {
  family: "persistence";
  compatibility: HostCapabilityCompatibility;
  registrationId: "persistence";
}

export interface HostPersistenceCapability<T = SignalValue> {
  value(): T;
  commit(): RunSummary | Promise<RunSummary>;
  descriptor(): PersistenceCapabilityDescriptor;
  readonly [WorthSignalPersistenceCapabilityHandleBrand]: "persistenceCapabilityHandle";
}

export interface CallableSignalsHost<TPersistence = SignalValue> {
  readonly viewport?: HostViewportCapability;
  readonly visibility?: HostVisibilityCapability;
  readonly online?: HostOnlineCapability;
  readonly clock?: HostClockCapability;
  readonly persistence?: HostPersistenceCapability<TPersistence>;
}

export function viewportCapability(options: ViewportCapabilityOptions): ViewportCapabilityRegistration;

export function visibilityCapability(options: VisibilityCapabilityOptions): VisibilityCapabilityRegistration;

export function onlineCapability(options: OnlineCapabilityOptions): OnlineCapabilityRegistration;

export function clockCapability(options: ClockCapabilityOptions): ClockCapabilityRegistration;

export function persistenceCapability<T = SignalValue>(options: PersistenceCapabilityOptions<T>): PersistenceCapabilityRegistration<T>;

export function hostCapabilityPlan<TPersistence = SignalValue>(input?: HostCapabilityPlanInput<TPersistence>): HostCapabilityPlan<TPersistence>;
