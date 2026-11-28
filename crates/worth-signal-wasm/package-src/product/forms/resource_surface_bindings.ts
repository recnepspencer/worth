import { stableValueDigest } from "./values/value_paths.js";
import { readAttachmentTransfersReport } from "./attachment_transfers/report.js";
import { readResourceDriftReport, noteResourceDrift } from "./resource_drift/report.js";
import { readResourceSourceReport } from "./sources/resource_source_report.js";

export function createResourceSurfaceBindings(context) {
  return Object.freeze({
    resourceSource() {
      return readResourceSourceReport(context.source);
    },
    resourceDrift() {
      syncResourceDrift(context);
      return readResourceDriftReport(context.resourceDrifts, {
        currentSourceDigest: stableValueDigest(context.authoritativeSource()),
        latestCanonicalSourceDigest: context.latestCanonicalSourceDigest(),
      });
    },
    attachmentTransfers() {
      return readAttachmentTransfersReport(
        context.formRef(),
        context.fieldDeclarations,
        context.source,
      );
    },
  });
}

function syncResourceDrift(context) {
  const currentSource = context.authoritativeSource();
  const sourceCompatibility = context.syncSourceCompatibility(currentSource);
  noteResourceDrift(context.resourceDrifts, {
    currentSourceDigest: stableValueDigest(currentSource),
    draft: context.draft(),
    effective: context.effective(),
    resourceSource: readResourceSourceReport(context.source),
    sourceCompatibility,
  });
}
