# WORTH Query Feature Reference

The Query feature reference moved out of `plans/`. It now lives in the Query
workspace at `workspaces/worth-query/docs/`, organized by the job you are doing
rather than by milestone.

## Where To Read

| Need | Document |
| --- | --- |
| Index of all Query docs and a reading order | [Worth Query Docs](../../../workspaces/worth-query/docs/README.md) |
| Runtime, authority, and facade model for AI agents and contributors | [AI Agent Orientation](../../../workspaces/worth-query/docs/AI_README.md) |
| The supported declare, install, admit, execute, and publish journey | [Ordinary Application Front Door](../../../workspaces/worth-query/docs/foundations/ordinary-application-front-door.md) |
| Programs, workflows, branches, invariants, and the application front door | [`foundations/`](../../../workspaces/worth-query/docs/foundations) |
| Feature capsules, operation bodies, creations, and graph read and touch authoring | [`authoring/`](../../../workspaces/worth-query/docs/authoring) |
| Aftermath, recovery, recorded correction, and storage retention | [`execution/`](../../../workspaces/worth-query/docs/execution) |
| Granular invalidation batches | [`runtime-surfaces/`](../../../workspaces/worth-query/docs/runtime-surfaces) |
| Aspects, authority lanes, relation laws, and schema validation | [`modeling/`](../../../workspaces/worth-query/docs/modeling) |
| Authorization, resources, causal inspection, basis lifecycle, and record history | [`capabilities/`](../../../workspaces/worth-query/docs/capabilities) |
| Installed operations, graph obligations, managed runs, and the certification kit | [`domain-capabilities/`](../../../workspaces/worth-query/docs/domain-capabilities) |

Platform-wide concepts and the public documentation map are in the repository
[`docs/`](../../../docs/README.md) folder.

## Rule

Milestone plans in `plans/WORTH-query/` explain why and how a capability was
built. Usage belongs in the crate docs above. If using a feature correctly
requires reading milestone history, the crate docs are incomplete and should be
fixed there.
