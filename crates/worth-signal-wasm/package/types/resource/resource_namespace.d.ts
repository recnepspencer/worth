import type { SignalValue } from "../model.js";
import type { CallableSignals, ScopedSignalNamespace } from "../callable_surface.js";
import type { ApiFactory, ApiScopeFactory } from "./api_namespace.js";
import type { FeatureStoreFactory } from "../local/feature_store.js";
import type { LocalNamespace } from "../local/local_namespace.js";
import type {
  DetailResourceFamily,
  CollectionResourceFamily,
  PagedResourceFamily,
} from "./resource_family_surfaces.js";
import type {
  DetailResourceDeclaration,
  ProcessingDetailResourceDeclaration,
  UploadDetailResourceDeclaration,
  ProcessingUploadDetailResourceDeclaration,
  CollectionResourceDeclaration,
  ProcessingCollectionResourceDeclaration,
  UploadCollectionResourceDeclaration,
  ProcessingUploadCollectionResourceDeclaration,
  PagedResourceDeclaration,
  ProcessingPagedResourceDeclaration,
  UploadPagedResourceDeclaration,
  ProcessingUploadPagedResourceDeclaration,
  ExternalDetailResourceDefinition,
  ExternalCollectionResourceDefinition,
  ExternalPagedResourceDefinition,
} from "./resource_declarations.js";
import type {
  DeclaredResourceParams,
  ResourceBinaryDescriptorFactory,
  ResourceBinaryValue,
  ResourceDownload,
  ResourceAuth,
  ResourceContinuation,
  ResourceParamIdentity,
  ResourcePolicyProfiles,
  ResourceProcessingJob,
  ResourceProcessingResult,
  ResourceRequestContext,
  ResourceRequestContextOptions,
  ResourceUploadResult,
  ResourceUploadTransport,
} from "./resource_postures.js";
import type {
  ResourceCollectionShape,
  ResourceDetailReconcile,
  ResourceDeliveryFactory,
  ResourceItemAspectMap,
  ResourceItemAspects,
  ResourcePatchFactory,
  ResourceExternalDeliveryFactory,
  ResourceValueSummaryMap,
  ResourceValueSummaries,
} from "./resource_reconciliation.js";
import type { ResourceResponseFactory } from "./resource_response.js";
import type { ResourceBranchNamespace } from "./resource_branch.js";
import type { ResourceEffects } from "./resource_effect_profiles.js";
import type { ResourceMutationResponses } from "./resource_mutation_response_closeout_matrix.js";

export interface ResourceCompatibilityNamespace {
  readonly delivery: ResourceExternalDeliveryFactory;
  detail<TParams, TValue>(
    definition: ExternalDetailResourceDefinition<TParams, TValue>,
  ): DetailResourceFamily<TParams, TValue>;
  collection<
    TParams,
    TValue,
    TItem = SignalValue,
    TReconcile extends ResourceCollectionShape<
      TValue,
      TItem,
      ResourceItemAspectMap<TItem>,
      ResourceValueSummaryMap<TValue>,
      any
    > | undefined = undefined,
  >(
    definition: ExternalCollectionResourceDefinition<
      TParams,
      TValue,
      TItem,
      TReconcile
    >,
  ): CollectionResourceFamily<TParams, TValue, TItem, TReconcile>;
  paged<
    TParams,
    TValue,
    TItem = SignalValue,
    TReconcile extends ResourceCollectionShape<
      TValue,
      TItem,
      ResourceItemAspectMap<TItem>,
      ResourceValueSummaryMap<TValue>,
      any
    > | undefined = undefined,
  >(
    definition: ExternalPagedResourceDefinition<
      TParams,
      TValue,
      TItem,
      TReconcile
    >,
  ): PagedResourceFamily<TParams, TValue, TItem, TReconcile>;
}

export interface ResourceNamespace {
  readonly branch: ResourceBranchNamespace;
  readonly compatibility: ResourceCompatibilityNamespace;
  /**
   * Marks every materialized line of every family in this namespace stale
   * (freshness `manualRuntimeInvalidateAll`, scope `runtimeAll`). Visible
   * values stay. Returns the number of lines marked. Lines materialized after
   * the call are unaffected.
   */
  invalidateAll(): number;
  /**
   * Starts a refresh on every materialized line of every family in this
   * namespace, with `line.refresh()` semantics per line (a pending reload is
   * superseded). Returns the number of lines refreshed.
   */
  refreshAll(): number;
  readonly detailFields: typeof resourceDetailFields;
  readonly detailRegions: typeof resourceDetailRegions;
  readonly detailJsonPaths: typeof resourceDetailJsonPaths;
  readonly effects: ResourceEffects;
  readonly mutationResponses: ResourceMutationResponses;
  readonly response: ResourceResponseFactory;
  detail<TParams, TValue, TReconcile extends ResourceDetailReconcile<TValue> | undefined = undefined>(
    declaration: ProcessingUploadDetailResourceDeclaration<TParams, TValue, TReconcile>,
  ): DetailResourceFamily<TParams, TValue | null, TReconcile>;
  detail<TParams, TValue, TReconcile extends ResourceDetailReconcile<TValue> | undefined = undefined>(
    declaration: ProcessingDetailResourceDeclaration<TParams, TValue, TReconcile>,
  ): DetailResourceFamily<TParams, TValue | null, TReconcile>;
  detail<TParams, TValue, TReconcile extends ResourceDetailReconcile<TValue> | undefined = undefined>(
    declaration: UploadDetailResourceDeclaration<TParams, TValue, TReconcile>,
  ): DetailResourceFamily<TParams, TValue | null, TReconcile>;
  detail<TParams, TValue, TReconcile extends ResourceDetailReconcile<TValue> | undefined = undefined>(
    declaration: DetailResourceDeclaration<TParams, TValue, TReconcile>,
  ): DetailResourceFamily<TParams, TValue, TReconcile>;
  collection<
    TParams,
    TValue,
    TItem = SignalValue,
    TReconcile extends ResourceCollectionShape<
      TValue,
      TItem,
      ResourceItemAspectMap<TItem>,
      ResourceValueSummaryMap<TValue>,
      any
    > | undefined = undefined,
  >(
    declaration: ProcessingUploadCollectionResourceDeclaration<
      TParams,
      TValue,
      TItem,
      TReconcile
    >,
  ): CollectionResourceFamily<TParams, TValue | null, TItem, TReconcile>;
  collection<
    TParams,
    TValue,
    TItem = SignalValue,
    TReconcile extends ResourceCollectionShape<
      TValue,
      TItem,
      ResourceItemAspectMap<TItem>,
      ResourceValueSummaryMap<TValue>,
      any
    > | undefined = undefined,
  >(
    declaration: ProcessingCollectionResourceDeclaration<
      TParams,
      TValue,
      TItem,
      TReconcile
    >,
  ): CollectionResourceFamily<TParams, TValue | null, TItem, TReconcile>;
  collection<
    TParams,
    TValue,
    TItem = SignalValue,
    TReconcile extends ResourceCollectionShape<
      TValue,
      TItem,
      ResourceItemAspectMap<TItem>,
      ResourceValueSummaryMap<TValue>,
      any
    > | undefined = undefined,
  >(
    declaration: UploadCollectionResourceDeclaration<
      TParams,
      TValue,
      TItem,
      TReconcile
    >,
  ): CollectionResourceFamily<TParams, TValue | null, TItem, TReconcile>;
  collection<
    TParams,
    TValue,
    TItem = SignalValue,
    TReconcile extends ResourceCollectionShape<
      TValue,
      TItem,
      ResourceItemAspectMap<TItem>,
      ResourceValueSummaryMap<TValue>,
      any
    > | undefined = undefined,
  >(
    declaration: CollectionResourceDeclaration<TParams, TValue, TItem, TReconcile>,
  ): CollectionResourceFamily<TParams, TValue, TItem, TReconcile>;
  paged<
    TParams,
    TValue,
    TItem = SignalValue,
    TReconcile extends ResourceCollectionShape<
      TValue,
      TItem,
      ResourceItemAspectMap<TItem>,
      ResourceValueSummaryMap<TValue>,
      any
    > | undefined = undefined,
  >(
    declaration: ProcessingUploadPagedResourceDeclaration<
      TParams,
      TValue,
      TItem,
      TReconcile
    >,
  ): PagedResourceFamily<TParams, TValue | null, TItem, TReconcile>;
  paged<
    TParams,
    TValue,
    TItem = SignalValue,
    TReconcile extends ResourceCollectionShape<
      TValue,
      TItem,
      ResourceItemAspectMap<TItem>,
      ResourceValueSummaryMap<TValue>,
      any
    > | undefined = undefined,
  >(
    declaration: ProcessingPagedResourceDeclaration<
      TParams,
      TValue,
      TItem,
      TReconcile
    >,
  ): PagedResourceFamily<TParams, TValue | null, TItem, TReconcile>;
  paged<
    TParams,
    TValue,
    TItem = SignalValue,
    TReconcile extends ResourceCollectionShape<
      TValue,
      TItem,
      ResourceItemAspectMap<TItem>,
      ResourceValueSummaryMap<TValue>,
      any
    > | undefined = undefined,
  >(
    declaration: UploadPagedResourceDeclaration<
      TParams,
      TValue,
      TItem,
      TReconcile
    >,
  ): PagedResourceFamily<TParams, TValue | null, TItem, TReconcile>;
  paged<
    TParams,
    TValue,
    TItem = SignalValue,
    TReconcile extends ResourceCollectionShape<
      TValue,
      TItem,
      ResourceItemAspectMap<TItem>,
      ResourceValueSummaryMap<TValue>,
      any
    > | undefined = undefined,
  >(
    declaration: PagedResourceDeclaration<TParams, TValue, TItem, TReconcile>,
  ): PagedResourceFamily<TParams, TValue, TItem, TReconcile>;
}

export function resourceParams<TParams>(): DeclaredResourceParams<TParams>;

export function resourceParamIdentity<TParams>(
  params: TParams,
  canonicalKey: string,
): ResourceParamIdentity<TParams>;

export function resourceBinaryValue<TValue>(options: {
  value: TValue;
  descriptors?: readonly import("./resource_postures.js").ResourceBinaryDescriptor[];
}): ResourceBinaryValue<TValue>;

export const resourceBinaryDescriptor: ResourceBinaryDescriptorFactory;
export const resourceDownload: ResourceDownload;
export const resourceAuth: ResourceAuth;
export const resourceContinuation: ResourceContinuation;
export const resourceDetailFields: typeof import("./resource_reconciliation.js").resourceDetailFields;
export const resourceDetailRegions: typeof import("./resource_reconciliation.js").resourceDetailRegions;
export const resourceDetailJsonPaths: typeof import("./resource_reconciliation.js").resourceDetailJsonPaths;
export const resourceDelivery: ResourceDeliveryFactory;
export const resourceEffects: ResourceEffects;
export const resourceMutationResponses: ResourceMutationResponses;
export const resourcePatch: ResourcePatchFactory;
export const resourcePolicyProfiles: ResourcePolicyProfiles;
export const resourceProcessingJob: ResourceProcessingJob;
export const resourceProcessingResult: ResourceProcessingResult;
export const resourceResponse: ResourceResponseFactory;
export const resourceUploadTransport: ResourceUploadTransport;
export const resourceUploadResult: ResourceUploadResult;

export function resourceItemAspects<
  TItem,
  TAspectMap extends ResourceItemAspectMap<TItem>,
>(definitions: TAspectMap): ResourceItemAspects<TItem, TAspectMap>;

export function resourceValueSummaries<
  TValue,
  TSummaryMap extends ResourceValueSummaryMap<TValue>,
>(definitions: TSummaryMap): ResourceValueSummaries<TValue, TSummaryMap, "line">;

export namespace resourceValueSummaries {
  function pageWindow<
    TValue,
    TSummaryMap extends ResourceValueSummaryMap<TValue>,
  >(definitions: TSummaryMap): ResourceValueSummaries<TValue, TSummaryMap, "pageWindow">;
}

export function resourceCollectionShape<
  TValue,
  TItem,
  TAspectMap extends ResourceItemAspectMap<TItem> = {},
  TSummaryMap extends ResourceValueSummaryMap<TValue> = {},
  TSummaryPatchScope extends "line" | "pageWindow" = "line",
>(options: {
  items(value: TValue): readonly TItem[];
  replaceItems(value: TValue, nextItems: readonly TItem[]): TValue;
  aspects?: ResourceItemAspects<TItem, TAspectMap>;
  summaries?: ResourceValueSummaries<TValue, TSummaryMap, TSummaryPatchScope>;
}): ResourceCollectionShape<TValue, TItem, TAspectMap, TSummaryMap, TSummaryPatchScope>;

export function resourceRequestContext(
  options?: ResourceRequestContextOptions,
): ResourceRequestContext;

declare module "../callable_surface.js" {
  interface CallableSignals<TPersistence = SignalValue> {
    readonly resource: ResourceNamespace;
    readonly api: ApiFactory;
    readonly apiScope: ApiScopeFactory;
    readonly local: LocalNamespace;
    readonly featureStore: FeatureStoreFactory;
  }

  interface ScopedSignalNamespace<TPersistence = SignalValue> {
    readonly resource: ResourceNamespace;
    readonly api: ApiFactory;
    readonly apiScope: ApiScopeFactory;
    readonly local: LocalNamespace;
    readonly featureStore: FeatureStoreFactory;
  }
}
