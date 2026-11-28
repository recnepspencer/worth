# Advanced Resource Capabilities

This guide is the map for capabilities that extend an existing resource line.
They do not replace the family-and-line model. Reach for the section that owns
the new problem and keep the original resource identity intact.

## Direct Family Authoring

Use the raw lane when route-derived identity or ordinary API finalizers are not
enough:

```ts
import {
  resourceParamIdentity,
  resourceParams,
} from "worth-signals-wasm";

const documentDetail = signals.resource.detail({
  params: resourceParams<{ tenantId: string; documentId: string }>(),
  normalizeParams: ({ tenantId, documentId }) =>
    resourceParamIdentity(
      { tenantId, documentId },
      `${tenantId}:${documentId}`,
    ),
  load: ({ documentId }) => client.loadDocument(documentId),
});
```

`normalizeParams(...)` is authoritative for canonical line identity in this
lane. The resulting line uses the same value, lifecycle, diagnostics, effects,
and history surfaces as a route-built line.

Do not drop to raw authoring because it sounds more powerful. It moves identity
and request-shape responsibility into your declaration.

## Capability Map

| Problem | Capability | Start here |
| --- | --- | --- |
| Signed upload, multipart upload, deferred processing | Transfers | [Uploads And Transfers](../transfers/README.md) |
| File, media, export, or multipart download | Binary descriptors and downloads | [Downloads And Binary Data](../downloads/README.md) |
| Server push or externally authored resource definitions | External delivery and compatibility | [External Delivery And Compatibility](../external-delivery-and-compatibility.md) |
| Use resource truth as a form source | Resource-backed forms | [Using Resources In Forms](../forms/README.md) |
| Prefetch and expose resource posture through navigation | Route resources | [Using Resources In Routes](../router/README.md) |
| Manual canonical identity and topology declarations | Raw resource families | [Raw Resource Lines](./raw-resource-lines.md) |
| Fields, regions, JSON paths, aspects, and summaries | Advanced narrow modeling | [Detail Fields, Regions, And JSON Paths](./detail-fields-regions-and-json-paths.md) |
| Support, compatibility, and performance evidence | Verification packages | [Verification And Proof](../verification/README.md) |

## Transfers And Downloads

Transfers add upload preparation and processing lifecycle to a line. Downloads
attach typed binary descriptors and availability to ordinary resource value.
Worth coordinates and describes these states; browser APIs, network transport,
and remote storage remain external boundaries.

A declared download can be ready, unavailable, or incompatible. A processing
job can be accepted without being complete. Preserve those distinctions in UI
and workflow code.

## Forms And Routes

A form can use a resource line as source authority while retaining its own
runtime-owned draft. A route can declare, prefetch, warm, and project resource
capabilities. Neither integration creates a second resource cache or promotes a
draft, route projection, or component value into server truth.

## External Delivery And Compatibility

External resource definitions and pushed packets carry compatibility and basis
evidence. A **basis** is the server or branch version the update expects to
extend. When that proof is stale or incompatible, the line can deny the update
or require refresh instead of accepting a plausible-looking payload.

## Replay And Restore

These are proof-bearing capabilities, not generic object utilities:

- replay requires retained executable history;
- exact restore requires an exact retained snapshot;
- compact inverse rollback is not the same guarantee as exact restore;
- an unavailable result is part of the supported contract.

## Common Mistakes

- Rebuilding line identity in a transfer, form, or route adapter.
- Treating raw authoring as an invitation to bypass family validation.
- Accepting external delivery without checking basis and compatibility.
- Assuming every retained history entry can be replayed.

## Reference

- [Resource Family Identity](./resource-family-identity.md)
- [Request Targets And Identity](./request-targets-and-identity.md)
- [Item Aspects And Value Summaries](./item-aspects-and-value-summaries.md)
- [Raw Resource Lines](./raw-resource-lines.md)
- [Resource API Reference](../../api-reference/resources.md)
