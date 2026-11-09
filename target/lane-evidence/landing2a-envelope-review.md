# Envelope/invariant public consumer review

Base1403ddb275. Reviewers read code/evidence only; no rebuild/test:
- /root/envelope_invariant_design_review: REJECT omissions in initial design.
- /root/envelope_invariant_corrected_design_review: APPROVE corrected design.
- /root/envelope_invariant_source_review: candidate nonempty-result P2 plus
  diagnostic equality correction; verified corrected hunks, no open code issue.
- /root/envelope_invariant_final_review: fresh final APPROVE; no open findings.

Resolved findings:
1. Preserve exact external I/O assertions (<32/zero-tail, zero/positive/warm and
   sticky no-reload, History noninterference, candidate<24), not C2 proxies.
2. Preserve exact original outgoing relation-ID Vec equality and partition4
   >=2 relation count with per-ID effective equality through exact root reads.
3. Candidate positive must observe both changed_entities()[0] IDs and actual
   created records; otherwise a successful no-op could pass. Corrected with
   actual candidate-value record reads. Private floor/slot accounting staysC2.
4. Existing public diagnostics_snapshot().diagnostics preserves exact original
   before/after diagnostic Vec equality; this claim is not movedC2.
5. Initial compile stopped on private touched_record_refs plus one unused
   import. For this actual unequal-name update fixture, public canonical commit
   identity and nonempty authoritative patches preserve the successful-body
   observation; no new getter/counter/bridge. Fixed both diagnostics together.

Memory image envelope list remains private. The court consumes the exact
captured eager image through the existing retained recovery plan, asserts public
whole-image equality, and uses real recovery coverage checkpoint_commits. Actual
owner computes this from restored image.envelopes.len; no metadata proxy.
Public reclamation and <4 count remain; only exact Weak/resident internals C2.

Qualification: first prebuild fail,0tests; corrected selected packet12/0/0.
FullRoot/testkit/court Clippy exits0,same32library/23lib-testwarnings,no newones.
Caps19filespass; functions9advisories/0errors, reviewed cohesive dispatch/scenario
orchestration. Mandatory boundary14unchanged(3ownedremainingadapters+11otherowners),noneonchangedfiles;context/fmtpass. OverallLanding2agatestillopen. Baseline-7,0adds;
6oldfiles/3testfunctions removed after12/0/0.29originalcoldtests remain.
Merge seam: facade/runtime_validation_exports.rs exports existingRecordKindTag.
No new public type/authority/constructor/control/wire/profile/feature.
