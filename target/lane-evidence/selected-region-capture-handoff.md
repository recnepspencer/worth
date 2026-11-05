# Narrow selected-region DATA handoff

Evidence: funding-finish-native-selected-courts.log3/5/0. Five cold recaptures
fail Region(0):SourceUnavailable. Existing original Complete operation in
storage/residency/checkpoint_capture/complete.rs uses lease_resident_full;
cell/resident_leases.rs excludes every checkpoint-backed cell, even cached Full.
Search: rg -n 'capture_complete_for_request|lease_resident_full|fn capture_checkpoint'
under Native storage/residency, plus full owner reads. Ordinary capture supports
cold references but creates unfunded/external payload aliases, so cannot replace
the funded Complete path. No ordinary-capture fallback is proposed.

Requested exact owner file only:
crates/worth-relational/src/storage/residency/checkpoint_capture/complete.rs.
Extract its existing funded emitter into a private same-cell Full-lease operation;
expose DATA emission on the existing counted RelationalResidencyLease<Full>.
Keep original-Full admission unchanged. No new type, root, basis, currentness,
wire/version, or trust source. Proposal: proposed-selected-region-capture.patch.
Owner file remains untouched pending scope ruling.

Existing checkpoint_complete consumer first carries a matching authenticated
selected root/region by reference (and retains the actual cell). Only changed
selected regions demand their actual region Full contents under the original
capture request and existing owned limit, then use the new DATA operation.
No canonical body demand/replay. All actual frame bytes retain intrinsic bills.
The Complete producer still requires its unforgeable selected capture base and
actual held roots before it can mint the composite checkpoint witness.

Bound: selected-directory joins O(S); unchanged covered roots consume metadata
only. Changed regions consume retained region DATA S plus bounded uncovered
contributions T. Flat Full emission resets their physical history layering;
never-checked/fresh path keeps the original Full-only emitter. Growth proof
continues to compare the same captured state with discarded canonical bytes.
