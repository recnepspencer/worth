# WORTH Query Feature Reference

The Query feature reference moved out of `plans/`. It now lives with the crate
at `workspaces/worth-query/crates/worth-query/docs/`, organized by the job you
are doing rather than by milestone.

## Where To Read

| Need | Document |
| --- | --- |
| Index of all Query docs and a reading order | [Worth Query Docs](../../../workspaces/worth-query/crates/worth-query/docs/README.md) |
| Runtime, authority, and facade model for AI agents and contributors | [AI Agent Orientation](../../../workspaces/worth-query/crates/worth-query/docs/AI_README.md) |
| The supported declare, install, admit, execute, and publish journey | [Ordinary Application Front Door](../../../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md) |
| Workspace posture, support matrix, operating modes, branches, and previews | [`foundations/`](../../../workspaces/worth-query/crates/worth-query/docs/foundations/) |
| Query expressions, result shapes, collections, and graph authoring | [`authoring/`](../../../workspaces/worth-query/crates/worth-query/docs/authoring/) |
| Effects, writes and intents, admission, and recovery | [`execution/`](../../../workspaces/worth-query/crates/worth-query/docs/execution/) |
| Live views, computed values, reads, and invalidation | [`runtime-surfaces/`](../../../workspaces/worth-query/crates/worth-query/docs/runtime-surfaces/) |
| Aspects, authority lanes, and schema validation | [`modeling/`](../../../workspaces/worth-query/crates/worth-query/docs/modeling/) |
| Inspection, history, lineage, projection consumption, and subscriptions | [`capabilities/`](../../../workspaces/worth-query/crates/worth-query/docs/capabilities/) |
| Domain-owned declarations, orchestration, continuation, and certification | [Domain Capabilities](../../../workspaces/worth-query/crates/worth-query/docs/domain-capabilities/README.md) |

Platform-wide concepts and the public documentation map are in the repository
[`docs/`](../../../docs/README.md) folder.

## Rule

Milestone plans in `plans/WORTH-query/` explain why and how a capability was
built. Usage belongs in the crate docs above. If using a feature correctly
requires reading milestone history, the crate docs are incomplete and should be
fixed there.
