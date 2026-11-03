# Authored retirement completeness — #45, #46, #47

Root D1082 and D1085 govern this batch. A delete must not strand a current
owned publication, and candidate execution must not become a sequence of
visible commits or a fabricated current-read proof.

## Boundary review

Native canonical history owns current publication membership, provenance,
route selection and closure. Reuse source_scope_closure for all anchors and
select_owned_scope_dependent for custody and external-endpoint dependencies.
Authored identities are authenticated by authored_birth; relations confer no
ownership on external endpoints. Query owns typed source intents, program
connections and producer dispatch, and cannot reconstruct intents from Native
owner IDs. Existing provisional facts are symbolic; existing document history
groups publish separate movements and cannot provide atomic coupling.

D1085 resolves source inputs: retirement/correction declares coupled roots
using existing program output connections. The operation input and pre-image
supply typed intent values. Producers read the unpublished candidate; Native
independently requires exactly the canonical dependent owners. Restoration
uses fresh authored IDs with the same declared coupling. An undeclared required
coupling refuses typed before effect. No replayable input contract is added.

## Implementation order

1. Extend the existing owned-dependent selector with record subjects. At commit
   admission, fence actual deletes/unlinks unless a completed authentic successor
   or admitted cascade covers the publication. Keep source-only callers unchanged.
   Prove external member deletion and owned-edge unlink through real Query.
2. Extend authored retirement to an explicit anchor set and per-anchor births.
   Authenticate each anchor and exclude current publication members. Reuse the
   existing closure and exact roster admission; preserve independent origins.
   Extend restoration to inverse Create correspondence for every anchor.
3. Extend Query authored correction declarations with typed co-retired roles,
   including zero-or-one cardinality, and consume the declaration through original
   and inverse correspondence. Court first-stage undo, multi-stage whole-owner
   retirement/restoration, extra roles and corruption.
4. Add unpublished candidate read custody and aggregate producer candidates prior
   to final invariant validation. Bind declared coupled roots through program
   output dispatch. Native validates exact required successor participants from
   canonical history, including no coupling, missing, extra, foreign and corrupt
   cases. Each producer retains writer custody and individual provenance.
5. Court tail/middle stage deletion and restoration, append undo, atomic refusals,
   original single-publication and independent paths, cold history where affected.
   Disable each new admission check and require a real Query court to fail; restore
   exact source and rebuild before accepting evidence.

## Verification and delivery

Run focused Query decl-host and Native owner courts, affected history/recovery
lanes, formatting, dirty Rust line caps, boundary-check and agent-context.
Verify an identical patch atop pin 190312b082 (7a8569b3b8 plus #44), with cargo
PATH excluding orchestration/scripts/shims. Preserve known unrelated master/pin
debt. Record commands, mutation failures and restored passes in the orchestration
report. Commit both locally; do not push.

## QA considerations

Candidate reads must carry actual candidate authority, never committed-current
or synthetic row evidence. Exact participant validation must reject missing and
extra owners before effect. Co-anchor restore must remap both endpoints without
adopting external entities. Coverage exceptions require native completion,
never a caller-supplied list. All new scans remain bounded and charged under the
existing request budgets; non-opted-in paths retain their existing semantics.

Co-anchor canonical history uses artifact format 4 and Native checkpoint magic
WORTH-REL-CHK16. This follows the existing development-format policy: no migration
promise; older framed formats refuse. Absent co-anchor fields preserve the
previous single-anchor canonical payload shape. Protocol framing is versioned.
