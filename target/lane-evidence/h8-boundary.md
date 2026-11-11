# H8 boundary brief

Base5d41be2990. Governing contracts: milestone-3.md Delivery tracks/H8, Phase8, R13, D15, D5/D8/D9/D11/D17; roadmap shared owner publication/readmission. Adversarial constraint: a completed operation followed by graceful close or a quiescent kill must preserve exact dispatch/inbound/claim meaning across fresh runtimes. Source installation is fresh; original custody and terminal/no-effect meaning must never be guessed or replaced. True in-flight kills/R14/boundary oracle are C3/C4, not H8.

## Observed owners and gaps

- Query external_effect/outbox.rs::restore installs inbound None; provider/committed_dispatch_outbox/original_row.rs reads exact original Native created rows. Current source receipt pairs outbox outcome with the same original idempotency row.
- Query identity_derivation.rs includes query-runtime and World owner.get(); source_identity/runtime_commitment.rs includes runtime authority and owner.get(). These are process-local, not Home meaning. Removing them must retain separate live-affinity fencing.
- Query OutstandingDispatchOwner is in-memory and preflight_home_retirement refuses outstanding custody. Inbound messages/expiry/terminal-ack/source posture and recovery claims are in-memory. PayloadRejected currently becomes Unresolved.
- Store registry.rs marks query.dispatch RequireAbsent. No Query dispatch medium is attached/readmitted; deleting that check alone is invalid D8 admission.
- Original operation definition is already durable: idempotency/admitted_intent + encoding put it in v2 :operation=. InstalledOperationView::definition_identity_bytes derives from package/schema/slot/input, not generation/runtime/seal. Schema identity includes inbound source and all limits. A current-slot lookup alone cannot prove original meaning; compare the authentic original operation digest first.
- World public observation.readmitted_native_publication accepts owner-readmitted Native effect; Query OriginalPublication::Home uses it. WorldReadmittedPublication owns original Native data/bases + history/catalog retention. Keep a distinct recorded Query original. It cannot turn into live Consumed/Performed. performed_publication_page skips readmitted entries and protect_performed_publication refuses them. Rebuild recorded terminal/correlation indexes from actual original effects and keep readmitted retention explicitly.
- New inbound completion can publish a NEW Native/World occurrence at the current head of the original occurrence. The old original remains recorded; no handler/effect replay.

## Composition and fences

No World change currently needed. A separate Store dispatch adapter/accessor on WorldReadmittedApplicationHome can use home.context.serving without editing held attachment.rs or readmission/query.rs. Host fresh.finish_world/existing.attach_world already own Home and World ports. Dedicated Query preparation methods can place the SAME dispatch owner in bootstrap.graph; unheld installation/home_attachment::preflight_home_application admits it and finish_home_application transfers it before existing held orchestrator mints QueryReadmitted. Do not transport dispatch authority through HomeMaintenance.

P-C holds home_preparation/world.rs, world/custody.rs, existing.rs, Host opening/service.rs and execution_runtime/product_world/home_attachment.rs. Probe every newly touched Query path exactly; previous probes do not grant other files. H5's explicit Query/Store holds stay intact. World/Native crate-wide grant belongs H5; do not edit either crate. The two retired World medium variants are for a later BOTH-owners merge, not this H8 slice.

## Remaining boundary work

Design owned query.dispatch append/continuation funding and D8 admission; define recorded-original dispatch/inbound/terminal retention and durable claims/R13 postures; skip close-checkpoint while external obligations exist but retire live shares honestly; lift D15 only after all paths exist. Old-schema obligations need genuine original installed meaning or an explicit transition disposition/refusal; current successor-slot lookup is insufficient. No silent policy choice yet.
