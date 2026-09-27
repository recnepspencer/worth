# Resource Milestone Crosswalk

This page keeps the older resource doc history discoverable alongside the new
task-first tree.

## New Tree To Older Docs

### Start Here

- [Your First Resource](./start-here/your-first-resource.md)
  draws mostly from:
  [Fetch And Write Resources](./fetch-and-write.md) and
  [Resource Family Authoring Reference](../api-reference/resource-family-authoring.md)
- [Choose A Resource Shape](./start-here/choose-a-resource-shape.md)
  draws mostly from:
  [Fetch And Write Resources](./fetch-and-write.md) and
  [Resource Family Authoring Reference](../api-reference/resource-family-authoring.md)

### Fetching

- [Fetch A Single Record](./fetching/fetch-a-single-record.md)
  draws mostly from:
  [Fetch And Write Resources](./fetch-and-write.md)
- [Fetch A Collection](./fetching/fetch-a-collection.md)
  draws mostly from:
  [Fetch And Write Resources](./fetch-and-write.md) and
  [Collections And Delivery](./collections-and-delivery.md)
- [Fetch A Paged List](./fetching/fetch-a-paged-list.md)
  draws mostly from:
  [Fetch And Write Resources](./fetch-and-write.md)
- [Request Auth And Context](./fetching/request-auth-and-context.md)
  draws mostly from:
  [Request Posture And Policy](./request-posture-and-policy.md) and
  [Resource Request And Policy Reference](../api-reference/resource-request-and-policy.md)
- [Request Policies](./fetching/request-policies.md)
  draws mostly from:
  [Request Posture And Policy](./request-posture-and-policy.md)

### Updating

- [Write A Resource](./updating/write-a-resource.md)
  draws mostly from:
  [Fetch And Write Resources](./fetch-and-write.md)
- [Submit Patches And Replacements](./updating/submit-patches-and-replacements.md)
  draws mostly from:
  [Collections And Delivery](./collections-and-delivery.md)
- [Choose An Effect Profile](./updating/choose-an-effect-profile.md)
  draws mostly from:
  [Branch-Native Resource Effects](./branch-native-effects.md),
  [Effect Merge And Rebase](./merge-and-rebase.md), and
  [Effect Closeout Matrix](./effects/README.md)
- [What Happens After A Write](./updating/what-happens-after-a-write.md)
  draws mostly from:
  [Line Inspection](./line-inspection.md),
  [Mutation Response Reconciliation](./mutation-response-reconciliation.md), and
  [History And Restore](./debugging/restore-replay-and-recover.md)

### Handling Server Responses

- [Understand Mutation Responses](./responses/understand-mutation-responses.md)
  draws mostly from:
  [Mutation Response Reconciliation](./mutation-response-reconciliation.md)
- [Handle Partial Canonical Truth](./responses/handle-partial-canonical-truth.md)
  draws mostly from:
  [Mutation Response Reconciliation](./mutation-response-reconciliation.md)
- [Handle Fallback Reconciliation](./responses/handle-fallback-reconciliation.md)
  draws mostly from:
  [Mutation Response Reconciliation](./mutation-response-reconciliation.md) and
  [Mutation Response Closeout Matrix](./responses/README.md)
- [Map Server Truth Back Into Local Truth](./responses/map-server-truth-back-into-local-truth.md)
  draws mostly from:
  [Mutation Response Reconciliation](./mutation-response-reconciliation.md),
  [Collections And Delivery](./collections-and-delivery.md)
  and the remove/create mutation-response proof lanes

### Working With Lists

- [List Shapes And Item Identity](./lists/list-shapes-and-item-identity.md)
  draws mostly from:
  [Collections And Delivery](./collections-and-delivery.md)
- [Visible Selection](./lists/visible-selection.md)
  draws mostly from:
  [Branch-Native Resource Effects](./branch-native-effects.md) and
  [Line Inspection](./line-inspection.md)
- [Update One Item Without Replacing Everything](./lists/update-one-item-without-replacing-everything.md)
  draws mostly from:
  [Collections And Delivery](./collections-and-delivery.md)
- [Derived Item Views And Summaries](./lists/derived-item-views-and-summaries.md)
  draws mostly from:
  [Collections And Delivery](./collections-and-delivery.md)

### Caching And Refresh

- [How Resource Caching Works](./caching/how-resource-caching-works.md)
  draws mostly from:
  [Line Inspection](./line-inspection.md)
- [Cache Keys And Resource Identity](./caching/cache-keys-and-resource-identity.md)
  draws mostly from:
  [Resource Family Authoring Reference](../api-reference/resource-family-authoring.md)
- [Stale, Pending, And Settled State](./caching/stale-pending-and-settled-state.md)
  draws mostly from:
  [Line Inspection](./line-inspection.md) and
  [Request Posture And Policy](./request-posture-and-policy.md)
- [Invalidation And Refresh](./caching/invalidation-and-refresh.md)
  draws mostly from:
  [Line Inspection](./line-inspection.md),
  [External Delivery And Compatibility](./external-delivery-and-compatibility.md), and
  [History And Restore](./debugging/restore-replay-and-recover.md)
- [Authoritative Vs Derived Resource Truth](./caching/authoritative-vs-derived-resource-truth.md)
  draws mostly from:
  [Collections And Delivery](./collections-and-delivery.md),
  [Line Inspection](./line-inspection.md), and
  [Branch-Native Resource Effects](./branch-native-effects.md)

### Partial Updates And Derived Views

- [How Partial Resource Updates Work](./partial-updates/how-partial-resource-updates-work.md)
  draws mostly from:
  [Collections And Delivery](./collections-and-delivery.md) and
  [JSON Path Effects](./json-effects.md)
- [Automatic Derived Views](./partial-updates/automatic-derived-views.md)
  draws mostly from:
  [Collections And Delivery](./collections-and-delivery.md) and
  [Response Topology Proof](./verification/response-topology-proof.md)
- [Update One Region, Field, Or Item](./partial-updates/update-one-region-field-or-item.md)
  draws mostly from:
  [Collections And Delivery](./collections-and-delivery.md) and
  [JSON Path Effects](./json-effects.md)
- [When To Declare Derived Views Explicitly](./partial-updates/when-to-declare-derived-views-explicitly.md)
  draws mostly from:
  [Collections And Delivery](./collections-and-delivery.md) and
  [JSON Path Effects](./json-effects.md)
- [How Partial Updates Affect Caching And Delivery](./partial-updates/how-partial-updates-affect-caching-and-delivery.md)
  draws mostly from:
  [Collections And Delivery](./collections-and-delivery.md),
  [Line Inspection](./line-inspection.md), and
  [Branch-Native Resource Effects](./branch-native-effects.md)

### Uploads And Transfers

- [Upload Files](./transfers/upload-files.md)
  draws mostly from:
  [Transfers](./transfers.md) and
  [Resource Transfers Reference](../api-reference/resource-transfers.md)
- [Track Processing Jobs](./transfers/track-processing-jobs.md)
  draws mostly from:
  [Transfers](./transfers.md) and
  [Resource Transfers Reference](../api-reference/resource-transfers.md)
- [Understand Transfer Results](./transfers/understand-transfer-results.md)
  draws mostly from:
  [Transfers](./transfers.md) and the transfer runtime proof lanes
- [Compatibility And Readiness](./transfers/compatibility-and-readiness.md)
  draws mostly from:
  the transfer runtime denial and compatibility proof lanes

### Downloads And Binary Data

- [Offer Downloads](./downloads/offer-downloads.md)
  draws mostly from:
  [Downloads](./downloads.md) and
  [Resource Binary And Download Reference](../api-reference/resource-binary-and-download.md)
- [Describe Binary Values](./downloads/describe-binary-values.md)
  draws mostly from:
  [Downloads](./downloads.md) and
  [Line Inspection](./line-inspection.md)
- [File, Media, And Export Downloads](./downloads/file-media-and-export-downloads.md)
  draws mostly from:
  [Downloads](./downloads.md) and the multipart/binary download proof lanes
- [Why A Download Is Unavailable](./downloads/why-a-download-is-unavailable.md)
  draws mostly from:
  [Downloads](./downloads.md) and the binary download boundary proof lanes

### Inspecting And Debugging Resources

- [Inspect A Resource Line](./debugging/inspect-a-resource-line.md)
  draws mostly from:
  [Line Inspection](./line-inspection.md)
- [Check Status, Freshness, And History](./debugging/check-status-settlement-and-history.md)
  draws mostly from:
  [Line Inspection](./line-inspection.md) and
  [History And Restore](./debugging/restore-replay-and-recover.md)
- [Why Did This View Update?](./debugging/why-did-this-view-update.md)
  draws mostly from:
  [Line Inspection](./line-inspection.md),
  [Collections And Delivery](./collections-and-delivery.md), and
  [Branch-Native Resource Effects](./branch-native-effects.md)
- [Why Didn't This View Update?](./debugging/why-didnt-this-view-update.md)
  draws mostly from:
  [Line Inspection](./line-inspection.md),
  [Request Posture And Policy](./request-posture-and-policy.md), and
  [History And Restore](./debugging/restore-replay-and-recover.md)
- [Read Delivery And Compatibility](./debugging/read-delivery-and-compatibility.md)
  draws mostly from:
  [External Delivery And Compatibility](./external-delivery-and-compatibility.md) and
  [Line Inspection](./line-inspection.md)
- [Restore, Replay, And Recover](./debugging/restore-replay-and-recover.md)
  draws mostly from:
  [History And Restore](./debugging/restore-replay-and-recover.md) and
  [Line Inspection](./line-inspection.md)

### Effects And Recovery

- [Branch-Native Effects](./effects/branch-native-effects.md)
  draws mostly from:
  [Branch-Native Resource Effects](./branch-native-effects.md)
- [Effect Envelopes And Closeout](./effects/effect-envelopes-and-closeout.md)
  draws mostly from:
  [Branch-Native Resource Effects](./branch-native-effects.md),
  [Effect Envelope Contract](./effects/effect-envelopes-and-closeout.md), and
  [Effect Closeout Matrix](./effects/README.md)
- [Merge And Rebase](./effects/merge-and-rebase.md)
  draws mostly from:
  [Effect Merge And Rebase](./merge-and-rebase.md)
- [Rollback And Recovery](./effects/rollback-and-recovery.md)
  draws mostly from:
  [Branch-Native Resource Effects](./branch-native-effects.md) and
  [History And Restore](./debugging/restore-replay-and-recover.md)

### Using Resources In Forms

- [Use A Resource As Form Source](./forms/use-a-resource-as-form-source.md)
  draws mostly from:
  [Resource-Backed Forms](../forms/resource-backed/README.md) and
  [Line Inspection](./line-inspection.md)
- [Reflect Resource Settlement In A Form](./forms/reflect-resource-settlement-in-a-form.md)
  draws mostly from:
  [Resource Settlement](../forms/resource-backed/resource-settlement.md) and
  [Line Inspection](./line-inspection.md)
- [Handle Resource Drift And Merge](./forms/handle-resource-drift-and-merge.md)
  draws mostly from:
  [Resource Drift](../forms/resource-backed/resource-drift.md),
  [Resource Merge](../forms/resource-backed/resource-merge.md), and
  [Effect Merge And Rebase](./merge-and-rebase.md)
- [Read Mutation Responses In Forms](./forms/read-mutation-responses-in-forms.md)
  draws mostly from:
  [Mutation Response Readback](../forms/resource-backed/mutation-response-readback.md) and
  [Mutation Response Reconciliation](./mutation-response-reconciliation.md)
- [Replay, Restore, And Reset Resource-Backed Forms](./forms/replay-restore-and-reset-resource-backed-forms.md)
  draws mostly from:
  [Replay And Restore](../forms/resource-backed/replay-and-restore.md) and
  [History And Restore](./debugging/restore-replay-and-recover.md)

### Using Resources In Routes

- [Declare Route Resources](./router/declare-route-resources.md)
  draws mostly from:
  [Router Route Resources](../router/resources/route_resource_declarations.md)
- [Prefetch And Warmup Route Resources](./router/prefetch-and-warmup-route-resources.md)
  draws mostly from:
  [Resource Prefetch](../router/resources/resource_prefetch.md) and
  [Resource Warmup](../router/resources/resource_warmup.md)
- [Read Projected And Admitted Resource Capabilities](./router/read-projected-and-admitted-resource-capabilities.md)
  draws mostly from:
  [Projected Resource Capabilities](../router/resources/projected_resource_capabilities.md) and
  [Admitted Resource Capabilities](../router/resources/admitted_resource_capabilities.md)

### Advanced Resource Modeling

- [Resource Family Identity](./advanced/resource-family-identity.md)
  draws mostly from:
  [Raw Escape Hatch](./raw-escape-hatch.md) and
  the family identity runtime proof lanes
- [Request Targets And Identity](./advanced/request-targets-and-identity.md)
  draws mostly from:
  [Raw Escape Hatch](./raw-escape-hatch.md),
  [Fetch And Write Resources](./fetch-and-write.md), and
  [Request Posture And Policy](./request-posture-and-policy.md)
- [Detail Fields, Regions, And Json Paths](./advanced/detail-fields-regions-and-json-paths.md)
  draws mostly from:
  [Reconciliation Contract](./responses/README.md) and
  [JSON Path Effects](./json-effects.md)
- [Item Aspects And Value Summaries](./advanced/item-aspects-and-value-summaries.md)
  draws mostly from:
  [Collections And Delivery](./collections-and-delivery.md) and
  [Reconciliation Contract](./responses/README.md)
- [Raw Resource Lines](./advanced/raw-resource-lines.md)
  draws mostly from:
  [Raw Escape Hatch](./raw-escape-hatch.md),
  [Line Inspection](./line-inspection.md), and
  the api-route DX parity closeout proofs

### Verification And Proof

- [Verification Packages](./verification/verification-packages.md)
  draws mostly from:
  [Inspection And History Contract](./debugging/README.md) and
  the line facade stability proof lane
- [Response Topology Proof](./verification/response-topology-proof.md)
  draws mostly from:
  [Response Topology Proof](./verification/response-topology-proof.md)
- [Mutation-Response Closeout Matrix](./verification/mutation-response-closeout-matrix.md)
  draws mostly from:
  [Mutation Response Closeout Matrix](./responses/README.md)
- [Delivery And Compatibility Digests](./verification/delivery-and-compatibility-digests.md)
  draws mostly from:
  [Delivery And Compatibility Contract](./debugging/read-delivery-and-compatibility.md) and
  the full resource hostile convergence proof lane

## What Has Not Moved Yet

These older pages are still worth keeping nearby for their deeper
milestone-era treatment of the same feature areas:

- [Mutation Response Reconciliation](./mutation-response-reconciliation.md)
- [Transfers](./transfers.md)
- [Downloads](./downloads.md)
- [Line Inspection](./line-inspection.md)
- [Branch-Native Resource Effects](./branch-native-effects.md)
- [Effect Merge And Rebase](./merge-and-rebase.md)
- [JSON Path Effects](./json-effects.md)
- [External Delivery And Compatibility](./external-delivery-and-compatibility.md)
- [Raw Escape Hatch](./raw-escape-hatch.md)
