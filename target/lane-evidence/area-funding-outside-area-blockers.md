# Funding qualification outside-area stop

The latest area grant permits storage/, durability/checkpoints/ and durability/data/, with ten explicit Lifecycle exclusions, and directs an outside-area requirement to be reported rather than edited. The earlier Complete capture grant remains active until funding lands. No listed exclusion was changed. No Host/Query/Store production file was broadened.

The ten exclusions, all relative to `crates/worth-relational/src/`, are:

- durability/checkpoints/region_directory.rs
- durability/checkpoints/symbol_pages.rs
- durability/data/checkpoint_directory.rs
- durability/data/checkpoint_directory/admission.rs
- durability/data/checkpoint_directory/retirement.rs
- durability/data/native_checkpoint_capture_denial.rs
- durability/data/recovery_stop.rs
- storage/residency/cell.rs
- storage/residency/cell/selected_readmission.rs
- storage/residency/kind_authority.rs

Independent review also verified `git diff --quiet 7d0e5189bc -- <all ten paths>` returns0 across the whole reviewed range and dirty source, not only this turn.

Full Root Clippy (`area-funding-root-clippy.log`, exit101) found eleven lint errors. Six are corrected in the granted funding owners: Complete destination-members nested if; Complete root-directory lazy fallback; HistoryFolder's two manually saturating additions; and current-image's two lazy constant refusals. These are equivalent expression changes, formatted in `area-funding-clippy-correction-format.log`. No runtime budget, authority, charging order or wire format changed by those expression corrections. The final Root repetition (`area-funding-final-root-clippy.log`) verifies that only the five outside-area sites remain.

The other five sites are outside the area, in earlier Native admission work. They are product lint failures and must be fixed before the full Root lane is green. They are not dismissed as unrelated debt. A sixth outside-area site stops full Store Clippy.

| Exact file under crates/worth-relational/src/ unless absolute repo path shown | Line in failing log | Required mechanical change | Last changing commit |
|---|---:|---|---|
| durability/artifact_payload_emission/request.rs | 140 | Combine nested if/let while preserving conditional work charging |4215bd9518|
| durability/checkpoint_admission/wire/emission.rs |13| Dereference Copy cut instead of clone |7b6d17eba7|
| durability/medium/recovery_source/home_preflight.rs |138| Constant MissingArtifact refusal through ok_or |7b6d17eba7|
| history/data/required_output_positions/readmission/node.rs |55| Combine nested if/let, preserving fallible family comparison |d61b52fe28|
| runtime/state/subsystems/durability/checkpoint_selection/source_retirement.rs |40| Combine nested if/let, preserving original retirement loan outcome |7b6d17eba7|
| workspaces/worth-store/crates/worth-store-application/src/world_section/close.rs |54| Remove needless borrow of an already borrowed conditional assembly |c39a85ce67|

The Native source-retirement file's last change is this branch's D12 admission commit; calling its lint somebody else's defect would be wrong. The Store close edit is outside the granted checkpoint-capture call site. The six proposed expression changes are in `proposed-outside-area-clippy.patch`, generated without writing any source file by `prepare_area_funding_clippy_handoff.py`. No suppression or allow attribute is proposed.

The final boundary run also finds two BC7013 references in the new public wide-contract court. The exact required additions to `tools/boundary-check/config/certification-construction-callers.txt` are:

```text
native-fresh-initialization crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/source_initial_inputs/checkpoint_admission/contract_work.rs
native-durable-initial-installation crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/source_initial_inputs/checkpoint_admission/contract_work.rs
```

These are a single exact court path and the existing two factory roles, with no pattern or new factory. The proposed addition is separate in `proposed-contract-court-callers.patch`; config remains untouched pending its outside-area ruling. The nearest existing allowed analogues are operational.rs and required_outputs.rs, registered in the same list. `rg --no-ignore -n 'begin_fresh_home_initialization|prepare_fresh_home_schema_installation' crates/worth-testkit-relational/src -g '*.rs'` finds only initial_input_source.rs and initial_input_source/schema_history.rs; their existing recipes do not take arbitrary contract declarations. No unrelated test relocation or new trust door is proposed to avoid enforcement. Checking the registry before adding this new court would have exposed the need earlier.

FUNDING LANDED cannot be claimed while full qualification remains red. Integration merging and the ordered decoder/D13/digest/BC9002 queue remain pending the funding gate. The focused runtime proofs, corrected strict cold-tail observation, unchanged <=40 control and instrumented sample remain available in `area-funding-lanes.md`; they do not stand in for Clippy or independent landing approval. Independent review also identifies the unrun InlineBytes and dependency/peer-history funding controls; the current Text lifetime cases do not establish them.
