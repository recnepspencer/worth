# Inline atomic values and Native content

`InlineBytes` remains the complete canonical representation of one atomic domain
value. `ContentRefId` is a digest that requires field-bound Native admission and
retained byte custody. The distinction is authority and meaning, never size.
Ordered identity-free numerical data, including splines at every size, uses
Native content. A frame exceeding 16 MiB refuses at admission before effects;
it does not switch to an inline representation.

## Actual semantic consumers

All concrete adopters found in the current tree are generic certification fixtures.
None is evidence of a deployed application.

| Consumer | Meaning and actual use |
| --- | --- |
| `CounterDelegate` / `InlineCounterBinding` | `NumericSchema.Number.Primary` installs a complete nine-byte versioned u64 atom. The correction model's pure conditional predicate decodes its observed `AspectValue` without a Native basis, installed content format or resolver. |
| `RatioDelegate` / `RatioBinding` | The topology entry fixture adopts a pure normalized rational atom; this is one number, not an ordered collection. |
| `StaleEncoder` / `BrokenBinaryBinding` | The correction write-admission fixture deliberately emits an invalid atomic encoding to prove refusal before publication. |

The counter's actual [conditional consumer](../workspaces/worth-query/crates/worth-query-testkit-certification-correction/src/certification/application_graph/correction/model/composite_definition_runtime.rs),
[schema installation](../workspaces/worth-query/crates/worth-query-testkit-certification-correction/src/certification/application_graph/correction/model/schema.rs) and
[binary delegate](../workspaces/worth-query/crates/worth-query-testkit-certification-correction/src/certification/application_graph/correction/model/domain_binding.rs) carry that dataflow. The executable correction
courts live in `worth-query-court-query-decl-host`'s `inline_domain` family.
The ratio's delegate is the topology entry `domain_values/binding.rs`.

The existing Native content door cannot replace these pure atomic contracts:
a content reference contains no value to decode without a resolver. Changing
the pure observation contract to lend Native custody would add a new authority
capability; importing the runtime into the pure owner violates dependency direction.
The complete atom therefore stays in the scalar. Its owner supplies canonical
admission and normalization; it contains no entity identity or graph references.
Native content owns format-bound datasets, integrity, retention and reconstruction.
Neither owner is an alternate blob store.

The 4,294,967,290-byte inline framing ceiling is a wire limit, not an admission
budget or permission to hide datasets. Each installed atomic binding supplies
finite representation bounds; original work and memory admission still apply.

## Audited routes

The pre-repair inventory includes every lexical `InlineBytes` path and every
broad binary binding/route path found during this review. The new owner
documentation and this inventory do not add scalar consumers. Carrier, wire, digest, facade,
projection, archive, renderer and test observation arms preserve an existing
scalar meaning; they are not additional semantic adopters. The two Store arms
are only exhaustive-match compilation fixes after retiring `Bytes`.

The broader search also found unrelated names: server binary surfaces and
`resultMaxInlineBytes` transport limits, physical binary witnesses/reports,
prebuilt executable binary fields, allocator probes and expression binary
operators. These do not consume the inline scalar. Lists are deliberately
separate so those lexical matches do not become capability claims.

### Inline scalar symbol paths (51)

- [docs/build-an-application.md](../docs/build-an-application.md)
- [docs/capability-index.md](../docs/capability-index.md)
- [workspaces/worth-query/crates/worth-query-testkit-certification-correction/src/certification/application_graph/correction/model/causal_inspection/output_equivalence/rich_source.rs](../workspaces/worth-query/crates/worth-query-testkit-certification-correction/src/certification/application_graph/correction/model/causal_inspection/output_equivalence/rich_source.rs)
- [workspaces/worth-query/crates/worth-query-testkit-certification-correction/src/certification/application_graph/correction/model/causal_inspection/output_equivalence/rich_connected.rs](../workspaces/worth-query/crates/worth-query-testkit-certification-correction/src/certification/application_graph/correction/model/causal_inspection/output_equivalence/rich_connected.rs)
- [workspaces/worth-store/crates/worth-store-aspect-native/src/terminal_json_projection.rs](../workspaces/worth-store/crates/worth-store-aspect-native/src/terminal_json_projection.rs)
- [workspaces/worth-store/crates/worth-store-aspect-native/src/contract_fingerprint.rs](../workspaces/worth-store/crates/worth-store-aspect-native/src/contract_fingerprint.rs)
- [crates/worth-foundational/src/value_shape/binary/tests.rs](../crates/worth-foundational/src/value_shape/binary/tests.rs)
- [crates/worth-foundational/src/value_shape/binary/mod.rs](../crates/worth-foundational/src/value_shape/binary/mod.rs)
- [crates/worth-foundational/src/value_shape/binary/encoding.rs](../crates/worth-foundational/src/value_shape/binary/encoding.rs)
- [crates/worth-foundational/src/value_shape/binary/cost.rs](../crates/worth-foundational/src/value_shape/binary/cost.rs)
- [crates/worth-foundational/src/values/scalar_wrappers/mod.rs](../crates/worth-foundational/src/values/scalar_wrappers/mod.rs)
- [crates/worth-foundational/src/values/scalar_wrappers/inline_bytes.rs](../crates/worth-foundational/src/values/scalar_wrappers/inline_bytes.rs)
- [crates/worth-foundational/src/values/scalar_wrappers/inline_bytes/tests.rs](../crates/worth-foundational/src/values/scalar_wrappers/inline_bytes/tests.rs)
- [crates/worth-foundational/src/values/scalar_kind.rs](../crates/worth-foundational/src/values/scalar_kind.rs)
- [crates/worth-foundational/src/values/mod.rs](../crates/worth-foundational/src/values/mod.rs)
- [crates/worth-foundational/src/values/aspect_value.rs](../crates/worth-foundational/src/values/aspect_value.rs)
- [crates/worth-foundational/src/facade.rs](../crates/worth-foundational/src/facade.rs)
- [workspaces/worth-query/crates/worth-query-package-archive/src/record/foundational_value.rs](../workspaces/worth-query/crates/worth-query-package-archive/src/record/foundational_value.rs)
- [workspaces/worth-query/crates/worth-query-package-archive/src/record/foundational_aspect.rs](../workspaces/worth-query/crates/worth-query-package-archive/src/record/foundational_aspect.rs)
- [crates/worth-runtime-bridge/src/canonical_basis.rs](../crates/worth-runtime-bridge/src/canonical_basis.rs)
- [crates/worth-foundational/src/canonicalization/value_lowering.rs](../crates/worth-foundational/src/canonicalization/value_lowering.rs)
- [crates/worth-relational/src/schema/aspect_plans/revision/foundational_contract_basis.rs](../crates/worth-relational/src/schema/aspect_plans/revision/foundational_contract_basis.rs)
- [crates/worth-foundational/src/canonicalization/digest_slots/material/value.rs](../crates/worth-foundational/src/canonicalization/digest_slots/material/value.rs)
- [crates/worth-foundational/src/canonicalization/digest_slots/material/scalar_view.rs](../crates/worth-foundational/src/canonicalization/digest_slots/material/scalar_view.rs)
- [crates/worth-foundational/src/canonicalization/digest_slots/material/borrowed_tests.rs](../crates/worth-foundational/src/canonicalization/digest_slots/material/borrowed_tests.rs)
- [crates/worth-foundational/src/canonicalization/contract_preparation/semantic_tokens.rs](../crates/worth-foundational/src/canonicalization/contract_preparation/semantic_tokens.rs)
- [crates/worth-foundational/src/canonicalization/basis/grammar/value.rs](../crates/worth-foundational/src/canonicalization/basis/grammar/value.rs)
- [crates/worth-foundational/src/canonicalization/basis/construction/entry_measurement.rs](../crates/worth-foundational/src/canonicalization/basis/construction/entry_measurement.rs)
- [crates/worth-relational/src/tests/transactions/core/native_value_matrix.rs](../crates/worth-relational/src/tests/transactions/core/native_value_matrix.rs)
- [crates/worth-relational/src/tests/support/history.rs](../crates/worth-relational/src/tests/support/history.rs)
- [crates/worth-relational/src/diagnostics/data/fields/native_serde/canonical_basis/value.rs](../crates/worth-relational/src/diagnostics/data/fields/native_serde/canonical_basis/value.rs)
- [crates/worth-relational/src/aspect_wire/aspect_value_canonical_codec_tests.rs](../crates/worth-relational/src/aspect_wire/aspect_value_canonical_codec_tests.rs)
- [crates/worth-relational/src/aspect_wire/aspect_value_canonical_codec/writing.rs](../crates/worth-relational/src/aspect_wire/aspect_value_canonical_codec/writing.rs)
- [crates/worth-relational/src/aspect_wire/aspect_value_canonical_codec/reading.rs](../crates/worth-relational/src/aspect_wire/aspect_value_canonical_codec/reading.rs)
- [crates/worth-relational/src/aspect_wire/aspect_value_canonical_codec/inline_bytes_tests.rs](../crates/worth-relational/src/aspect_wire/aspect_value_canonical_codec/inline_bytes_tests.rs)
- [workspaces/worth-query/crates/worth-query-installation/src/application_schema/value_binding/compilation_tests/domain_posture.rs](../workspaces/worth-query/crates/worth-query-installation/src/application_schema/value_binding/compilation_tests/domain_posture.rs)
- [workspaces/worth-query/crates/worth-query-installation/src/application_schema/value_binding/compilation.rs](../workspaces/worth-query/crates/worth-query-installation/src/application_schema/value_binding/compilation.rs)
- [crates/worth-server-client-generation/src/typescript.rs](../crates/worth-server-client-generation/src/typescript.rs)
- [workspaces/worth-query/crates/worth-query-declaration/src/validation/predicates.rs](../workspaces/worth-query/crates/worth-query-declaration/src/validation/predicates.rs)
- [workspaces/worth-query/crates/worth-query-declaration/src/facade.rs](../workspaces/worth-query/crates/worth-query-declaration/src/facade.rs)
- [workspaces/worth-query/crates/worth-query-court-query-decl-host/tests/worth_query_court_query_decl_host/certification/application_graph/correction/inline_domain.rs](../workspaces/worth-query/crates/worth-query-court-query-decl-host/tests/worth_query_court_query_decl_host/certification/application_graph/correction/inline_domain.rs)
- [workspaces/worth-query/crates/worth-query-court-query-decl-host/tests/worth_query_court_query_decl_host/certification/application_graph/correction/inline_domain/predicate.rs](../workspaces/worth-query/crates/worth-query-court-query-decl-host/tests/worth_query_court_query_decl_host/certification/application_graph/correction/inline_domain/predicate.rs)
- [workspaces/worth-query/crates/worth-query-court-query-decl-host/tests/worth_query_court_query_decl_host/certification/application_graph/correction/inline_domain/native.rs](../workspaces/worth-query/crates/worth-query-court-query-decl-host/tests/worth_query_court_query_decl_host/certification/application_graph/correction/inline_domain/native.rs)
- [workspaces/worth-query/crates/worth-query-declaration/src/binary_value_binding_macro.rs](../workspaces/worth-query/crates/worth-query-declaration/src/binary_value_binding_macro.rs)
- [workspaces/worth-query/crates/worth-query-declaration/src/application_schema/value_binding/canonical/mod.rs](../workspaces/worth-query/crates/worth-query-declaration/src/application_schema/value_binding/canonical/mod.rs)
- [workspaces/worth-query/crates/worth-query-declaration/src/application_schema/value_binding/canonical/binary.rs](../workspaces/worth-query/crates/worth-query-declaration/src/application_schema/value_binding/canonical/binary.rs)
- [workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/domain_values/tests.rs](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/domain_values/tests.rs)
- [workspaces/worth-query/crates/worth-query-declaration/src/application_schema/canonical_basis.rs](../workspaces/worth-query/crates/worth-query-declaration/src/application_schema/canonical_basis.rs)
- [workspaces/worth-query/crates/worth-query/src/projection_consumption/consumed/native_refinement.rs](../workspaces/worth-query/crates/worth-query/src/projection_consumption/consumed/native_refinement.rs)
- [workspaces/worth-query/crates/worth-query/src/projection_consumption/certification/oracle/value_terms.rs](../workspaces/worth-query/crates/worth-query/src/projection_consumption/certification/oracle/value_terms.rs)
- [workspaces/worth-query/crates/worth-query/src/runtime/graph_read_access/schema_reference_evidence/admitted_field_kind.rs](../workspaces/worth-query/crates/worth-query/src/runtime/graph_read_access/schema_reference_evidence/admitted_field_kind.rs)

### Broader binary route paths (27)

- [workspaces/worth-ui/apps/platform-pulse/tests/executable_world/product_process/installation_progression.rs](../workspaces/worth-ui/apps/platform-pulse/tests/executable_world/product_process/installation_progression.rs)
- [workspaces/worth-ui/apps/platform-pulse/tests/executable_world/courtroom/native_phase_f.rs](../workspaces/worth-ui/apps/platform-pulse/tests/executable_world/courtroom/native_phase_f.rs)
- [workspaces/worth-store/crates/worth-store-test-support/src/prebuilt_executable.rs](../workspaces/worth-store/crates/worth-store-test-support/src/prebuilt_executable.rs)
- [workspaces/worth-query/crates/worth-query-testkit-certification-correction/src/certification/application_graph/correction/model/write_admission/mod.rs](../workspaces/worth-query/crates/worth-query-testkit-certification-correction/src/certification/application_graph/correction/model/write_admission/mod.rs)
- [workspaces/worth-query/crates/worth-query-testkit-certification-correction/src/certification/application_graph/correction/model/write_admission/binary.rs](../workspaces/worth-query/crates/worth-query-testkit-certification-correction/src/certification/application_graph/correction/model/write_admission/binary.rs)
- [workspaces/worth-query/crates/worth-query-testkit-certification-correction/src/certification/application_graph/correction/model/domain_binding.rs](../workspaces/worth-query/crates/worth-query-testkit-certification-correction/src/certification/application_graph/correction/model/domain_binding.rs)
- [crates/worth-server/src/surfaces/mod.rs](../crates/worth-server/src/surfaces/mod.rs)
- [crates/worth-server/src/surfaces/facade.rs](../crates/worth-server/src/surfaces/facade.rs)
- [workspaces/worth-store/crates/worth-store-court-physical-format/tests/worth_store_court_physical_format/process/offline_integrity_observer/observation_contracts/report_boundary_and_binary/mod.rs](../workspaces/worth-store/crates/worth-store-court-physical-format/tests/worth_store_court_physical_format/process/offline_integrity_observer/observation_contracts/report_boundary_and_binary/mod.rs)
- [workspaces/worth-store/crates/worth-store-court-physical-format/tests/worth_store_court_physical_format/offline_integrity_observer/observation_contracts/report_boundary_and_binary/mod.rs](../workspaces/worth-store/crates/worth-store-court-physical-format/tests/worth_store_court_physical_format/offline_integrity_observer/observation_contracts/report_boundary_and_binary/mod.rs)
- [workspaces/worth-store/crates/worth-store-physical-format/src/header/authority.rs](../workspaces/worth-store/crates/worth-store-physical-format/src/header/authority.rs)
- [workspaces/worth-query/crates/worth-query-declaration/src/value_binding_macro.rs](../workspaces/worth-query/crates/worth-query-declaration/src/value_binding_macro.rs)
- [workspaces/worth-store/crates/worth-store-physical-backend/src/filesystem_media/tests/allocation_probe.rs](../workspaces/worth-store/crates/worth-store-physical-backend/src/filesystem_media/tests/allocation_probe.rs)
- [workspaces/worth-query/crates/worth-query-declaration/src/binary_value_binding_macro.rs](../workspaces/worth-query/crates/worth-query-declaration/src/binary_value_binding_macro.rs)
- [workspaces/worth-query/crates/worth-query-declaration/src/application_schema/value_binding/mod.rs](../workspaces/worth-query/crates/worth-query-declaration/src/application_schema/value_binding/mod.rs)
- [workspaces/worth-query/crates/worth-query-declaration/src/application_schema/value_binding/canonical/mod.rs](../workspaces/worth-query/crates/worth-query-declaration/src/application_schema/value_binding/canonical/mod.rs)
- [workspaces/worth-query/crates/worth-query-declaration/src/application_schema/value_binding/canonical/binary.rs](../workspaces/worth-query/crates/worth-query-declaration/src/application_schema/value_binding/canonical/binary.rs)
- [workspaces/worth-query/crates/worth-query-declaration/src/application_schema/mod.rs](../workspaces/worth-query/crates/worth-query-declaration/src/application_schema/mod.rs)
- [crates/worth-foundational/src/value_shape/mod.rs](../crates/worth-foundational/src/value_shape/mod.rs)
- [crates/worth-foundational/src/value_shape/binary/tests.rs](../crates/worth-foundational/src/value_shape/binary/tests.rs)
- [crates/worth-foundational/src/value_shape/binary/mod.rs](../crates/worth-foundational/src/value_shape/binary/mod.rs)
- [crates/worth-foundational/src/value_shape/binary/encoding.rs](../crates/worth-foundational/src/value_shape/binary/encoding.rs)
- [crates/worth-foundational/src/value_shape/binary/decoding.rs](../crates/worth-foundational/src/value_shape/binary/decoding.rs)
- [crates/worth-foundational/src/value_shape/binary/cost.rs](../crates/worth-foundational/src/value_shape/binary/cost.rs)
- [crates/worth-foundational/src/facade.rs](../crates/worth-foundational/src/facade.rs)
- [workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/domain_values/binding.rs](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/domain_values/binding.rs)
- [crates/worth-foundational/src/expressions/canonical/basis.rs](../crates/worth-foundational/src/expressions/canonical/basis.rs)
