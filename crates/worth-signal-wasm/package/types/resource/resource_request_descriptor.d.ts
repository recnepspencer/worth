import type { ResourceEffectProfile } from "./resource_effect_profiles.js";
import type {
  ResourceAuthPosture,
  ResourceContinuationPosture,
  ResourceParamIdentity,
  ResourceProcessingJobPosture,
  ResourceRequestContext,
  ResourceUploadTransportPosture,
} from "./resource_postures.js";
import type { ResourceLineCompatibilityDigest } from "./resource_verification.js";

export type ResourceFamilyKind = "detail" | "collection" | "paged";

export interface ResourceFamilyIdentity {
  readonly kind: ResourceFamilyKind;
  readonly familyId: string;
}

export interface ResourceLineDescriptor<TParams> {
  readonly family: ResourceFamilyIdentity;
  readonly canonicalParams: ResourceParamIdentity<TParams>;
  readonly runtimeLineId: string;
  readonly scopeId: string;
  readonly compatibility?: Exclude<ResourceLineCompatibilityDigest, { kind: "native" }>;
}

export interface ResourceRequestTarget {
  readonly baseUrl: string | null;
  readonly requestPath: string | null;
  readonly url: string | null;
}

export type ResourceRequestMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";

export interface ResourceRequestDescriptor<TParams> {
  readonly family: ResourceFamilyIdentity;
  readonly canonicalParams: ResourceParamIdentity<TParams>;
  readonly target: ResourceRequestTarget;
  readonly baseUrl: string | null;
  readonly method: ResourceRequestMethod;
  readonly body: unknown | null;
  readonly auth: ResourceAuthPosture;
  readonly context: ResourceRequestContext;
  readonly continuation: ResourceContinuationPosture;
  readonly processingJob: ResourceProcessingJobPosture;
  readonly uploadTransport: ResourceUploadTransportPosture;
  readonly effects: ResourceEffectProfile | null;
}

export interface ResourceRequestContextSummary {
  readonly headerNames: readonly string[];
  readonly correlationId: string | null;
  readonly branchId: string | number | null;
  readonly basisId: string | null;
}

export interface ResourceRequestDiagnostics {
  readonly baseUrl: string | null;
  readonly target: ResourceRequestTarget;
  readonly method: ResourceRequestMethod;
  readonly bodyPresent: boolean;
  readonly auth: ResourceAuthPosture;
  readonly context: ResourceRequestContextSummary;
  readonly continuation: ResourceContinuationPosture;
  readonly processingJob: ResourceProcessingJobPosture;
  readonly uploadTransport: ResourceUploadTransportPosture;
  readonly effects: ResourceEffectProfile | null;
}
