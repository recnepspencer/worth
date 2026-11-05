# Exact D12 boundary and facade entries

No caller pattern or baseline is broadened. certification-construction-callers.txt
adds these six exact factory/caller entries (the first two preceded phase checks):

```text
native-fresh-initialization crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/source_initial_inputs/checkpoint_admission/operational.rs
native-durable-initial-installation crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/source_initial_inputs/checkpoint_admission/operational.rs
native-home-original-source-copy crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/source_initial_inputs/checkpoint_admission/region_memory.rs
native-fresh-initialization crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/source_initial_inputs/checkpoint_admission/required_outputs.rs
native-durable-initial-installation crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/source_initial_inputs/checkpoint_admission/required_outputs.rs
native-home-complete-checkpoint workspaces/worth-store/crates/worth-store-court-application/tests/worth_store_court_application/process/application/native_components/checkpoint.rs

```

control-callers.txt adds one exact receiver/caller row:

```text
workspaces/worth-store/crates/worth-store-application/ workspaces/worth-store/crates/worth-store-court-application/tests/worth_store_court_application/process/application/native_components/admission_refusals.rs
```

One new owning runtime type, CapturedHomeRelationalSection, is exported through
Store Relational's existing facade and its section/checkpoint module exports.
It has private fields and no byte constructor, Clone or selection/ACK method.
No Cargo feature or dependency direction was added. sha2 moved from dev-only to
normal Store Relational dependency; root/Query lockfile changes are mechanical.
The three BC9002 adapter findings remain explicitly deferred; no baseline row,
root set, local adapter or enforcement suppression has changed in this slice.
