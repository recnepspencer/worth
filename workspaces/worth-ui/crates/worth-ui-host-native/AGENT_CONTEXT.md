# worth-ui-host-native

Generated from the machine-owned Road 1 boundary model. Do not edit by hand.
Canonical machine constitution: `tools/boundary-check/config/road1.toml`

- Constitutional class: `worth/ui`
- Domain noun: `host-native`
- Crate root: `workspaces/worth-ui/crates/worth-ui-host-native`
- Road 1 exemplar role: WORTH UI workspace-owned implementation surface.
- Deferred next homes:

- Public surface: workspace-owned; package targets remain the explicit export or composition owners
- Facade exports: `none`
- Owned internal modules: `native, native_profile, prepared_host, prepared_host_tests, qualification, qualification_tests, text_profile`
- Allowed in-tree dependency bands: `WORTH UI manifest-declared dependencies`

Machine fences:
- Must not depend on worthy-* crates.
- Replay dependencies are admitted only for configured certification packages: worth-ui-certification.
- Production dependencies on the direct Query engine remain confined by the configured Worth UI Query edge; certification-only test dependencies are outside that production fence.

Skeleton fence:
- No Road 1 seed skeleton applies; WORTH UI topology is workspace-owned and mechanically discovered.
