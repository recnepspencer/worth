# Proposal 02 — inbound processing leaves the runtime

Status: implemented under the proposal-01 rulings (facade on the runtime, module-owned host
traits, `&dyn` by default). Nothing here needs a new ruling.

## The cluster

`application_runtime::inbound_occurrence` (21 methods, 26 owned-field accesses) and
`application_runtime::inbound_publication` (10 methods, 18 accesses) are the two inbound
clusters of the runtime map. They already call each other through 12 named methods and
together hold 4,642 lines. Their state is four fields of `WorthQueryApplicationRuntimeCore`:

| field | type | reached from |
|---|---|---|
| `inbound_verifiers` | `Mutex<BTreeMap<String, Arc<WorthQueryInstalledInboundVerifier>>>` | the two clusters, installation assembly, teardown |
| `inbound_custody` | `Arc<Mutex<WorthQueryInboundCustody>>` | the two clusters, external dispatch, assembly, teardown |
| `transport_completion_custody` | `Mutex<InstalledTransportCompletionCustody>` | inbound publication, assembly, teardown |
| `inbound_maintenance_turn` | `AtomicU64` | inbound occurrence maintenance, assembly |

No code outside these clusters, external dispatch, assembly and teardown reaches the fields.

## The move

1. A new primary-graph module, `application_inbound`, takes both clusters whole
   (`application_inbound::inbound_occurrence`, `application_inbound::inbound_publication`,
   by `module_move.py`, so every path is rewritten and no shim is left).
2. `application_inbound::processing::WorthQueryInboundProcessing` owns the four fields. The
   core holds one `inbound: WorthQueryInboundProcessing` where the four sat, and the
   post-conditional teardown takes the verifiers, custody and completions from it exactly as
   it took them from the core, so drop order is unchanged.
3. `application_inbound::host::ApplicationInboundHost` lists what the clusters need from
   above (the inbound processing state, plus what the compile names); the runtime implements
   it in `application_runtime/host/application_inbound.rs`.
4. The clusters' runtime impl blocks become extension traits over that host, one per original
   visibility (`runtime_extension.py`); public non-test methods keep facades on the runtime
   in `application_runtime/facade/application_inbound.rs`.
5. External dispatch (still in `application_runtime`) reaches the custody through the state.

## Placement

`application_inbound` sits below `application_attempt`, whose external dispatch resumes
installed transport completions and progresses retained inbound occurrences; the attempt host
extends the inbound host when those bridges are removed. Its upward needs (advancement
opening, product recovery release) are host methods.
