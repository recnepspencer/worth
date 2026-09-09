# worth-ui-platform-pulse

Generated from the machine-owned Road 1 boundary model. Do not edit by hand.
Canonical machine constitution: `tools/boundary-check/config/road1.toml`

- Constitutional class: `worth/ui`
- Domain noun: `platform-pulse`
- Crate root: `workspaces/worth-ui/apps/platform-pulse`
- Road 1 exemplar role: WORTH UI workspace-owned implementation surface.
- Deferred next homes:

- Public surface: workspace-owned; package targets remain the explicit export or composition owners
- Facade exports: `none`
- Owned internal modules: `application, application_readiness, bin, intent, launch_configuration, lifecycle_observation_publication, main, native_application, native_close_evidence, native_gate_d_application, native_phase2_evidence, native_phase3_application, native_phase6_evidence, native_phase7_evidence, native_phase8_evidence, native_phase8_world, native_phase_f_application, native_phase_f_cancellation_world, native_phase_f_deferred_completion_world, native_phase_f_evidence, native_phase_f_reconstruction_world, native_phase_f_world, native_phase_f_world_evidence, native_seed_application, observation_contract, product_process, product_world, query_source, source_watch, visual_identity_adjudication, visual_identity_execution, visual_identity_pulse, visual_observation_publication`
- Allowed in-tree dependency bands: `WORTH UI manifest-declared dependencies`

Machine fences:
- Must not depend on worthy-* crates.
- Replay dependencies are admitted only for configured certification packages: worth-ui-certification.
- Production dependencies on the direct Query engine remain confined by the configured Worth UI Query edge; certification-only test dependencies are outside that production fence.

Skeleton fence:
- No Road 1 seed skeleton applies; WORTH UI topology is workspace-owned and mechanically discovered.
