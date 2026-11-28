# Durable original operation custody

## Retain an original operation across graceful reopen

On an installed Directory Home, call
`description.retain_durable_operation::<Program, Action>(&request, runtime)`
with the current authenticated request. The original description, captured input,
key, preconditions, action and source DATA join the existing Query obligation
journal. World retains the original effect and aggregate identity; required output
source and demand custody stay with their existing owners.

Use the native owner’s consuming `handoff_for_reopen(&request)` at a quiescent
boundary, then close Home. Handoff releases actual live cleanup and process
interests; it does not complete pending output obligations. Unfinished publication
or held live owners still refuse close. After reopening, authenticate again and
use `WorthQueryOriginalOperation::readmit_native` for a self-contained native
intent, `readmit` for a captured binding, or `readmit_from_query` for the exact
recorded query source. The returned description retains the same identity and
resolves the original World idempotency before any further advancement.

The old process token refuses with `StaleIncarnation`. Missing recorded input,
source, installed action or native ownership material answers
`OwnershipMaterialUnavailable`; a resolving original never executes a fresh
replacement when Native no longer has that original. Durable document-history
owners and branch retirement remain assigned to track C2.

## Related guidance

- [Ordinary application entry](../foundations/ordinary-application-front-door.md)
- [External effects and recovery](application-aftermath-and-recovery.md)
