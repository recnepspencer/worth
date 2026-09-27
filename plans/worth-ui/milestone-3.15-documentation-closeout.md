# Milestone 3.15 Documentation Closeout

## Scope

This record closes only the Milestone 3.15 documentation lane against baseline
`ed958f9fa7`. It does not mark the milestone complete and does not certify work
owned by the runtime, protocol, scale, command-evidence, or native Platform
Pulse lanes.

The durable product guide is
[`runtime-services.md`](../../workspaces/worth-ui/docs/runtime-services.md).
That guide describes only public facade and inspection behavior demonstrable
from the baseline source and its checked-in tests.

## Baseline-Demonstrable Contract

| Claim | Baseline evidence | Documentation posture |
| --- | --- | --- |
| Six sibling service families have distinct policy and ownership meanings | `worth_ui::facade::service`, normalized policy types, and named runtime owner modules | Installed public model |
| Policy defaults alone install no owner; declarations/capabilities create demand | `worth-ui/tests/service_policy_facade.rs` | Installed and compile/runtime checked |
| Portal demand brings its Focus and Motion requirements, while Scroll and Command Routing remain demand-driven | `worth-ui/tests/service_policy_facade.rs` | Installed public behavior |
| Explicit intent-origin service destinations are OpenPortal, ClosePortal, and InvokeCommand | `UiIntentRuntimeServiceDestination` and builder registration tests | Installed intent subset, not a universal service enum |
| WUI service declarations reject invalid clauses at the source boundary | `worth-ui-dsl/src/source/tests/phase8_service_declaration_tests.rs` | Installed language contract |
| Active sessions expose bounded `why_*` summaries and a runtime-service resource census | `active_application_session/service_inspection.rs` and `worth-ui-inspection/src/service` | Installed read-only evidence |
| Query remains a separate audience with separate admission; replay remains certification-only | workspace manifests, UI/Query facades, and boundary enforcement | Constitutional boundary |
| Scheduled scale amplification is reconstructive evidence | ignored `runtime_services/scale_amplification.rs` tests | Not ordinary-lane cost and not closed by docs |

The deterministic parity and protocol-fault fixtures are supporting evidence,
not proof of every family state transition or the full scale claim. The guide
therefore does not use them to declare `RS-07` or `RS-10` complete.

## Placeholder And Compatibility Deletion Inventory

The 3.15 cutover is destructive at the placeholder boundary. Documentation and
examples must not preserve or teach these predecessor surfaces:

- `UiUnsupportedServiceIntentExecutionBinding`;
- `register_unsupported_service` and
  `register_unsupported_intent_definition` builder paths;
- the universal `worth_ui.runtime_service.unsupported` execution posture;
- `CommandDescriptor::default_shortcut_reference` and string shortcut
  identity; or
- an ambiguous host `Focus` observation that collapses window activation,
  semantic focus, and physical semantic-focus placement.

The replacement surfaces are typed runtime-service intent registration, typed
shortcut sequences, explicit `WindowFocus` observation, and solicited semantic
focus placement with acknowledgement/reconciliation. There are no aliases or
fallback lanes for the deleted names.

## Pending Lane Evidence

The following claims remain owned outside this documentation lane even when
supporting source is visible in the shared repository:

- the complete command causal-evidence chain and typed `Unrouted` terminal
  observation;
- adversarial protocol ordering, fault, settlement, and reconciliation closure;
- the scheduled large-scale amplification courtroom and its exact locality
  bounds; and
- native Platform Pulse pixels, 960-by-600 to 1120-by-700 resize journey,
  reference screenshot, and native service-lifecycle success.

Until those owning lanes land their decisive evidence, documentation may name
the required contract and source-level composition, but must not report native
visual success or milestone completion. The application lifecycle guide marks
this boundary explicitly.

## Roadmap Handoff

The existing roadmap already assigns state-driven appearance to Milestone
3.16. That work may consume coherent portal, focus, motion, scroll, and
selection postures as inputs. It must not read mutable owner internals, create
a parallel state lane, or reinterpret service authority. This closeout creates
no new 3.16 requirement.

Later shell, table, canvas, accessibility, native-integration, plugin, and
developer-tool milestones may add consumers or sibling contracts at the public
facade, host-contract, or bounded-inspection insertion points. They may not
replace the six owners, promote inspection into authority, move replay into an
ordinary lane, or make Worth UI a Query authority.
