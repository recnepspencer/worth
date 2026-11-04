# Standalone Embedded original-frame exports

Search evidence: funding-embedded-export-callers.txt, complete untruncated
`rg -n "\.native_checkpoint\(|\.native_checkpoint_with_policy\(" crates workspaces -g "*.rs"`.

Real production callers of the default Embedded export:

- Query primary_graph/application_checkpoint/capture.rs,
  `capture_application_checkpoint_with_sections`: explicit interim application
  checkpoint demand, with the caller phase request and capture policy.
- Query primary_graph/bootstrap/checkpoint_transition/publication.rs: captures
  the explicit interim transition result under its supplied capture policy.
- Query primary_graph/bootstrap/checkpoint_transition/repair.rs: captures the
  explicit transition repair result under its supplied capture policy.

The Query application_invariant/typed_view/derived_relation/native_snapshot.rs
references are behind its parent's cfg(test). Native prepared_close.rs is a
placement-parameter forwarding door, not an unconditional Embedded selection.
Store Relational section/checkpoint.rs passes ExternalMembers with the actual
Store member roster. Native checkpointing.rs is the default-policy wrapper.
The remaining references are Native unit tests, external courts or testkit
recipes. The hidden Query capture_native_truth_checkpoint_for_test is a
certification door, not an ordinary caller.

These ordinary Native exports produce RelationalNativeCheckpoint, and Query
wraps that in WorthQueryApplicationCheckpoint. Home Complete capture uses the
distinct complete service and actual external membership. The selected
Complete service now denies Embedded immediately after metadata freeze, before
original read protection, index demand or canonical body demand. It does not
call the standalone export. No export API, format or version changed.
