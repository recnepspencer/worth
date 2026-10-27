mod touch_scope;
use crate::canonical_hash_encoding::CanonicalHashSink as Sink;
use touch_scope::hash_touch_scope;

use crate::canonical_hash_encoding::hash_text_field;

use super::{
    bool_name, conditional_nodes::hash_conditional_nodes, hash_sequence,
    WorthQueryDomainOperationCanonicalSemantics,
};
use crate::domain_operation::*;

pub(super) fn hash_input_and_graph_contracts(
    hasher: &mut impl Sink,
    semantics: &impl WorthQueryDomainOperationCanonicalSemantics,
) {
    hash_parameters(hasher, semantics.parameters());
    hash_sequence(
        hasher,
        "required-domain",
        semantics
            .required_domains()
            .iter()
            .map(|role| role.as_str()),
    );
    hash_native_projection(hasher, semantics.native_projection());
    hash_text_field(hasher, "query-intent", semantics.query_intent_digest());
    hash_text_field(hasher, "result-shape", semantics.result_shape_digest());
    hash_collection(hasher, semantics.collection());
    hash_sequence(
        hasher,
        "required-capability",
        semantics
            .required_capabilities()
            .iter()
            .map(|capability| capability.as_str()),
    );
    hash_conditional_nodes(hasher, semantics.conditional_nodes(), "operation-condition");
    hash_graph_reads(hasher, semantics.graph_reads());
    hash_touches(hasher, semantics.touches());
    hash_effects(hasher, semantics.effects());
    hash_invariants(hasher, semantics.invariants());
    hash_invariant_execution(hasher, semantics.invariant_execution());
}

fn hash_parameters(hasher: &mut impl Sink, contract: &WorthQueryOperationParameterContract) {
    match contract {
        WorthQueryOperationParameterContract::NotRequired => {
            hash_text_field(hasher, "parameters", "not-required");
        }
        WorthQueryOperationParameterContract::Declared { fields } => {
            hash_text_field(hasher, "parameters", "declared");
            for field in fields {
                hash_text_field(hasher, "parameter-name", &field.name);
                hash_text_field(hasher, "parameter-required", bool_name(field.required));
                match &field.value_family {
                    WorthQueryOperationValueFamily::NativeAspect { key, identity } => {
                        hash_text_field(hasher, "parameter-family", "native-aspect");
                        hash_text_field(hasher, "parameter-aspect-key", key.as_str());
                        hash_text_field(
                            hasher,
                            "parameter-aspect-identity",
                            &identity.0.to_string(),
                        );
                    }
                    family => {
                        hash_text_field(hasher, "parameter-family", value_family_name(family));
                    }
                }
            }
        }
    }
}

fn hash_native_projection(
    hasher: &mut impl Sink,
    contract: &WorthQueryOperationNativeProjectionContract,
) {
    hash_text_field(
        hasher,
        "native-contract-canonical-material",
        contract.canonical_contract_material(),
    );
    if contract.mask().is_whole_aspect() {
        hash_text_field(hasher, "native-mask", "whole-aspect");
    } else {
        for path in contract.mask().paths() {
            hash_text_field(hasher, "native-mask-path", "declared");
            for field in path.fields() {
                hash_text_field(hasher, "native-mask-field", field.as_str());
            }
        }
    }
}

fn hash_collection(hasher: &mut impl Sink, contract: &WorthQueryOperationCollectionContract) {
    match contract {
        WorthQueryOperationCollectionContract::NotCollection => {
            hash_text_field(hasher, "collection", "not-collection");
        }
        WorthQueryOperationCollectionContract::Collection {
            row_identity_field,
            ordering_fields,
            grouping,
            window,
            continuation,
        } => {
            hash_text_field(hasher, "collection", "collection");
            hash_collection_field(hasher, "row-identity-field", row_identity_field);
            for field in ordering_fields {
                hash_collection_field(hasher, "ordering-field", field);
            }
            match grouping {
                WorthQueryOperationGroupingContract::Ungrouped => {
                    hash_text_field(hasher, "grouping", "ungrouped");
                }
                WorthQueryOperationGroupingContract::Grouped { grouping_fields } => {
                    hash_text_field(hasher, "grouping", "grouped");
                    for field in grouping_fields {
                        hash_collection_field(hasher, "grouping-field", field);
                    }
                }
            }
            hash_text_field(
                hasher,
                "window-policy",
                match window {
                    WorthQueryOperationWindowPolicy::CompleteCollection => "complete-collection",
                    WorthQueryOperationWindowPolicy::ContinuationBounded => "continuation-bounded",
                },
            );
            hash_text_field(hasher, "continuation", continuation_name(*continuation));
        }
    }
}

fn hash_collection_field(
    hasher: &mut impl Sink,
    label: &str,
    field: &WorthQueryOperationCollectionField,
) {
    hash_text_field(hasher, label, field.aspect_key().as_str());
    for part in field.field_path().fields() {
        hash_text_field(hasher, "collection-field-path-part", part.as_str());
    }
}

fn hash_graph_reads(hasher: &mut impl Sink, contract: &WorthQueryOperationGraphReadContract) {
    match contract {
        WorthQueryOperationGraphReadContract::NotRequired => {
            hash_text_field(hasher, "graph-read", "not-required");
        }
        WorthQueryOperationGraphReadContract::Declared { roles } => {
            hash_text_field(hasher, "graph-read", "declared-application");
            for role in roles {
                hash_text_field(hasher, "graph-read-role", role.role());
                hash_graph_participation(hasher, role.participation());
                hash_text_field(hasher, "graph-access", graph_access_name(role.access()));
                for read in role.read_scopes() {
                    hash_application_read_scope(hasher, read);
                }
            }
        }
        WorthQueryOperationGraphReadContract::DeclaredDomain { roles } => {
            hash_text_field(hasher, "graph-read", "declared-domain");
            for role in roles {
                hash_text_field(hasher, "graph-read-role", &role.role);
                hash_graph_participation(hasher, &role.participation);
                hash_text_field(hasher, "graph-access", graph_access_name(role.access));
                for read in &role.semantic_reads {
                    hash_text_field(hasher, "graph-semantic-read", "native-projection");
                    hash_native_projection(hasher, read);
                }
            }
        }
    }
}

fn hash_graph_participation(
    hasher: &mut impl Sink,
    participation: &WorthQueryOperationGraphParticipation,
) {
    match participation {
        WorthQueryOperationGraphParticipation::PrimaryLogicalGraph => {
            hash_text_field(hasher, "graph-participation", "primary");
        }
        WorthQueryOperationGraphParticipation::SeparateAuthority { role } => {
            hash_text_field(hasher, "graph-participation", "separate");
            hash_text_field(hasher, "graph-participation-role", role);
        }
    }
}

fn hash_application_read_scope(hasher: &mut impl Sink, scope: &WorthQueryOperationGraphReadScope) {
    match scope {
        WorthQueryOperationGraphReadScope::Components(scope) => {
            hash_text_field(hasher, "graph-read-scope", "components");
            hash_binding(hasher, scope.schema());
            hash_text_field(
                hasher,
                "graph-read-component-scope",
                &scope.meaning().canonical_record(),
            );
        }
        WorthQueryOperationGraphReadScope::Entity(entity) => {
            hash_text_field(hasher, "graph-read-scope", "entity");
            hash_binding(hasher, entity.schema());
            hash_text_field(hasher, "graph-read-entity", entity.semantic_key());
        }
        WorthQueryOperationGraphReadScope::NativeProjection(projection) => {
            hash_text_field(hasher, "graph-read-scope", "native-projection");
            hash_binding(hasher, projection.schema());
            hash_text_field(
                hasher,
                "graph-read-entity",
                projection.entity().semantic_key(),
            );
            hash_text_field(hasher, "graph-read-aspect", projection.aspect().as_str());
            hash_native_projection(hasher, projection.projection());
        }
        WorthQueryOperationGraphReadScope::Relation(relation) => {
            hash_text_field(hasher, "graph-read-scope", "relation");
            hash_binding(hasher, relation.schema());
            hash_text_field(hasher, "graph-read-relation", relation.relation());
            hash_text_field(hasher, "graph-read-from", relation.from());
            hash_text_field(hasher, "graph-read-to", relation.to());
        }
    }
}

fn hash_touches(hasher: &mut impl Sink, contract: &WorthQueryOperationTouchContract) {
    match contract {
        WorthQueryOperationTouchContract::NotRequired => {
            hash_text_field(hasher, "touch", "not-required");
        }
        WorthQueryOperationTouchContract::Declared {
            graph_roles,
            scopes,
        } => {
            hash_text_field(hasher, "touch", "declared");
            hash_sequence(
                hasher,
                "touch-graph",
                graph_roles.iter().map(String::as_str),
            );
            for scope in scopes {
                hash_touch_scope(hasher, scope);
            }
        }
    }
}

fn hash_binding(
    hasher: &mut impl Sink,
    binding: &worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity,
) {
    hash_text_field(
        hasher,
        "scope-runtime-ordinal",
        &binding.runtime_ordinal().to_string(),
    );
    hash_text_field(
        hasher,
        "scope-generation",
        &binding.generation().to_string(),
    );
    hash_text_field(
        hasher,
        "scope-package",
        &binding.package_identity().render_hex(),
    );
    hash_text_field(
        hasher,
        "scope-schema",
        &binding.schema_identity().render_hex(),
    );
}

fn hash_effects(hasher: &mut impl Sink, contract: &WorthQueryOperationEffectContract) {
    match contract {
        WorthQueryOperationEffectContract::NotRequired => {
            hash_text_field(hasher, "effect", "not-required");
        }
        WorthQueryOperationEffectContract::Declared { effect_families } => {
            hash_text_field(hasher, "effect", "declared");
            hash_sequence(
                hasher,
                "effect-family",
                effect_families.iter().map(|family| family.as_str()),
            );
        }
    }
}

fn hash_invariants(hasher: &mut impl Sink, contract: &WorthQueryOperationInvariantContract) {
    match contract {
        WorthQueryOperationInvariantContract::NotRequired => {
            hash_text_field(hasher, "invariant", "not-required");
        }
        WorthQueryOperationInvariantContract::Declared { invariant_slots } => hash_sequence(
            hasher,
            "invariant-slot",
            invariant_slots.iter().map(String::as_str),
        ),
    }
}

fn hash_invariant_execution(
    hasher: &mut impl Sink,
    contract: &WorthQueryInvariantExecutionContract,
) {
    match contract {
        WorthQueryInvariantExecutionContract::NotRequired => {
            hash_text_field(hasher, "invariant-execution", "not-required");
        }
        WorthQueryInvariantExecutionContract::Declared { requirements } => {
            hash_text_field(hasher, "invariant-execution", "declared");
            for requirement in requirements {
                let parts = requirement.canonical_parts();
                hash_sequence(
                    hasher,
                    "invariant-execution-requirement",
                    parts.iter().map(String::as_str),
                );
            }
        }
    }
}

fn value_family_name(family: &WorthQueryOperationValueFamily) -> &'static str {
    match family {
        WorthQueryOperationValueFamily::Bool => "bool",
        WorthQueryOperationValueFamily::I64 => "i64",
        WorthQueryOperationValueFamily::U64 => "u64",
        WorthQueryOperationValueFamily::Text => "text",
        WorthQueryOperationValueFamily::EntityIdentity => "entity-identity",
        WorthQueryOperationValueFamily::NativeAspect { .. } => "native-aspect",
    }
}

fn continuation_name(posture: WorthQueryOperationContinuationPosture) -> &'static str {
    match posture {
        WorthQueryOperationContinuationPosture::NotRequired => "not-required",
        WorthQueryOperationContinuationPosture::SnapshotCursor => "snapshot-cursor",
        WorthQueryOperationContinuationPosture::LiveCursor => "live-cursor",
    }
}

fn graph_access_name(access: WorthQueryOperationGraphAccess) -> &'static str {
    match access {
        WorthQueryOperationGraphAccess::Observe => "observe",
        WorthQueryOperationGraphAccess::Project => "project",
    }
}
