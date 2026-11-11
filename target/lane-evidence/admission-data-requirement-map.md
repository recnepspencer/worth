# Admission DATA requirement map

Source range3d7ccf16b0..d61b52fe28. These are prerequisite observations, not
substitutes for the operational filesystem acceptance courts.

| Requirement | Implemented boundary / named observation | Status |
|---|---|---|
| Coordinator digest ruling; D12 projected meanings | fixed_digest_encoding_has_value_independent_width_and_round_trips; checkpoint_image_seal_work_is_independent_of_digest_octet_values | Green at recorded source states |
| D12 meaning preservation; D17 type distinctions | fixed_byte_encoding_preserves_nonbyte_tuples_and_variable_byte_sequences; fixed_byte_probe_obeys_the_underlying_serializer_mode | Green; value codecs only |
| D12 original-request cost | fixed_byte_probe_does_not_format_nonbyte_tuple_values; original_checkpoint_capture_seals_each_projected_role_independently | Green; no extra probe rendering, role/copy/hash work observed |
| D12 independent authentication of roles | original_checkpoint_capture_seals_each_projected_role_independently | Green; each role/root payload and directory digest mutated independently |
| D12 source/cut binding DATA | checkpoint_image_sealing_refuses_a_different_original_source_or_cut | Green; does not prove protected Store selection |
| D12 version/request preflight | checkpoint_image_framing_refuses_unknown_versions_and_stopped_requests | Green; selected-Home predecessor package is still unimplemented |
| D12 logical durable axes | checkpoint_image_records_logical_reference_coordinates_across_runtime_instances | Green; runtime-allocation coordinates excluded |
| D12 original publication membership | checkpoint_image_records_original_required_output_membership_with_logical_basis | Green; actual installed publisher, actual publication, original assertions retained |
| D12 selector prefixes | checkpoint_image_refuses_unresolved_routes_outside_its_canonical_roster | Green for DATA refusal; actual recovered unresolved-prefix operation remains unrun |
| D12 selector meaning, balanced original image | checkpoint_image_preserves_balanced_selectors_across_distinct_original_publications | Green; three genuine original publications, authenticated DATA parsing only |
| D12 bounded polling and invoking controls | long_selector_families_preserve_utf8_and_bound_invoking_request_stops | Green; genuine long UTF-8 families, two tight partial-decode work stops with <=64KiB slack, cancelled request before allocation |
| D12 image-backed basis, pins, publication constructor cutover | root.rs, root_readmission, checkpoint_images, associated_effects, recorded_component_readmission | NOT IMPLEMENTED |
| D12 selected-Home protected package / missing and predecessor cutoff | Store Complete section preparation and source readmission | NOT IMPLEMENTED |
| D12 <=40 canonical bodies / zero-member ACK | unchanged filesystem bounded-tail and interrupted-ACK courts | UNRUN this resume; historical red observations remain unresolved |
| D13 / grant11b | captured-origin registry and receiver restoration | NOT IMPLEMENTED |
| Grant12 | remaining provider-dependent courts, adapter deletion, baseline shrink and zero BC9002 after integration merge | NOT IMPLEMENTED |

No new facade type, public operational door, Cargo feature or boundary-config
entry was added in this range. Native's missing-image guard was not edited.
