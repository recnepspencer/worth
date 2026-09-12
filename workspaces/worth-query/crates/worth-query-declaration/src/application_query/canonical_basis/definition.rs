use worth_foundational::facade::CanonicalBasisEntry;

use super::{
    entry::{boolean, null, text, unsigned_u64, unsigned_usize},
    prepare_artifact,
    result_shape::{append_result_shape, cardinality_name, traversal_direction_name},
    ApplicationQueryCanonicalArtifact,
};
use crate::application_query::{
    ApplicationQueryAuthorizationRequirement, ApplicationQueryDisclosurePosture,
    ApplicationQueryOrderingDirection, WorthQueryPortableApplicationQueryParts,
};

mod disclosure;
use disclosure::append_disclosure;

pub(in crate::application_query) fn prepare_definition_basis(
    definition: &WorthQueryPortableApplicationQueryParts,
) -> ApplicationQueryCanonicalArtifact {
    let mut entries = definition_header(definition);
    append_parameters(&mut entries, definition);
    append_root_paths(&mut entries, definition);
    append_result_shape(&mut entries, definition.result_shape(), "result");
    append_predicates(&mut entries, definition);
    append_ordering(&mut entries, definition);
    append_continuation(&mut entries, definition);
    append_authorization(&mut entries, definition);
    append_disclosure(&mut entries, definition);
    append_live_cause(&mut entries, definition);
    append_controls(&mut entries, definition);
    prepare_artifact(entries)
}

fn append_root_paths(
    entries: &mut Vec<CanonicalBasisEntry>,
    definition: &WorthQueryPortableApplicationQueryParts,
) {
    for (path_index, root_path) in definition.root_paths().iter().enumerate() {
        let path = format!("root-path[{path_index}]");
        entries.extend([
            text(format!("{path}.start"), root_path.start_entity()),
            text(format!("{path}.terminal"), root_path.terminal_entity()),
        ]);
        for (step_index, step) in root_path.steps().iter().enumerate() {
            let step_path = format!("{path}.step[{step_index}]");
            entries.extend([
                text(format!("{step_path}.relation"), step.relation()),
                text(format!("{step_path}.from"), step.from()),
                text(format!("{step_path}.to"), step.to()),
                text(
                    format!("{step_path}.direction"),
                    match step.direction() {
                        crate::application_query::ApplicationQueryRootPathDirection::Forward => {
                            "forward"
                        }
                        crate::application_query::ApplicationQueryRootPathDirection::Reverse => {
                            "reverse"
                        }
                    },
                ),
            ]);
        }
        for (guard_index, guard) in root_path.guards().iter().enumerate() {
            let guard_path = format!("{path}.guard[{guard_index}]");
            entries.extend([
                unsigned_usize(format!("{guard_path}.after-step"), guard.after_step()),
                text(format!("{guard_path}.entity"), guard.entity()),
                text(format!("{guard_path}.aspect"), guard.aspect()),
                text(format!("{guard_path}.field"), guard.field()),
                text(
                    format!("{guard_path}.scalar"),
                    guard.scalar_family().canonical_name(),
                ),
                text(format!("{guard_path}.value-type"), guard.value_type()),
                text(
                    format!("{guard_path}.expected"),
                    worth_foundational::facade::prepare_aspect_value_identity_basis(
                        guard.expected(),
                    )
                    .as_str(),
                ),
            ]);
        }
    }
}

fn definition_header(
    definition: &WorthQueryPortableApplicationQueryParts,
) -> Vec<CanonicalBasisEntry> {
    vec![
        text("definition.name", definition.name()),
        text("definition.query-type", definition.query_type()),
        text("definition.parameter-type", definition.parameter_type()),
        text("definition.result-type", definition.result_type()),
        text("definition.scope-type", definition.scope_type()),
        text("definition.root-entity", definition.root_entity()),
        text("definition.scope-entity", definition.scope_entity()),
        text(
            "definition.cardinality",
            cardinality_name(definition.cardinality()),
        ),
    ]
}

fn append_parameters(
    entries: &mut Vec<CanonicalBasisEntry>,
    definition: &WorthQueryPortableApplicationQueryParts,
) {
    for (index, parameter) in definition.parameters().iter().enumerate() {
        let path = format!("parameter[{index}]");
        entries.extend([
            text(format!("{path}.name"), parameter.name()),
            text(
                format!("{path}.scalar"),
                parameter.scalar_family().canonical_name(),
            ),
            text(format!("{path}.value-type"), parameter.value_type()),
            parameter.unit().map_or_else(
                || null(format!("{path}.unit")),
                |unit| text(format!("{path}.unit"), unit),
            ),
            parameter.frame().map_or_else(
                || null(format!("{path}.frame")),
                |frame| text(format!("{path}.frame"), frame),
            ),
        ]);
    }
}

fn append_predicates(
    entries: &mut Vec<CanonicalBasisEntry>,
    definition: &WorthQueryPortableApplicationQueryParts,
) {
    for (index, predicate) in definition.predicates().iter().enumerate() {
        let path = format!("predicate[{index}]");
        let (entity, aspect, field) = predicate.field();
        entries.extend([
            text(format!("{path}.entity"), entity),
            text(format!("{path}.aspect"), aspect),
            text(format!("{path}.field"), field),
            text(format!("{path}.parameter"), predicate.parameter()),
            text(
                format!("{path}.scalar"),
                predicate.scalar_family().canonical_name(),
            ),
        ]);
    }
}

fn append_ordering(
    entries: &mut Vec<CanonicalBasisEntry>,
    definition: &WorthQueryPortableApplicationQueryParts,
) {
    for (index, ordering) in definition.ordering().iter().enumerate() {
        let path = format!("ordering[{index}]");
        let (entity, aspect, field) = ordering.field();
        entries.extend([
            text(format!("{path}.query-type"), ordering.query_type()),
            text(format!("{path}.slot-type"), ordering.slot_type()),
            text(format!("{path}.entity"), entity),
            text(format!("{path}.aspect"), aspect),
            text(format!("{path}.field"), field),
            text(format!("{path}.output"), ordering.output_name()),
            text(format!("{path}.value-type"), ordering.value_type()),
            text(
                format!("{path}.direction"),
                ordering_direction_name(ordering.direction()),
            ),
            text(
                format!("{path}.scalar"),
                ordering.scalar_family().canonical_name(),
            ),
        ]);
    }
}

fn append_continuation(
    entries: &mut Vec<CanonicalBasisEntry>,
    definition: &WorthQueryPortableApplicationQueryParts,
) {
    let Some(continuation) = definition.continuation() else {
        entries.push(null("continuation"));
        return;
    };
    entries.extend([
        text("continuation.query-type", continuation.query_type()),
        text("continuation.slot-type", continuation.slot_type()),
        text("continuation.relation", continuation.relation()),
        text("continuation.parent-entity", continuation.parent_entity()),
        text("continuation.child-entity", continuation.child_entity()),
        text(
            "continuation.direction",
            traversal_direction_name(continuation.direction()),
        ),
    ]);
}

fn append_authorization(
    entries: &mut Vec<CanonicalBasisEntry>,
    definition: &WorthQueryPortableApplicationQueryParts,
) {
    match definition.authorization() {
        ApplicationQueryAuthorizationRequirement::Public => {
            entries.push(text("authorization.posture", "public"));
        }
        ApplicationQueryAuthorizationRequirement::Ability {
            ability,
            scope_entity,
        } => {
            entries.extend([
                text("authorization.posture", "ability"),
                text("authorization.ability", ability),
                text("authorization.scope-entity", scope_entity),
            ]);
        }
    }
}

fn append_live_cause(
    entries: &mut Vec<CanonicalBasisEntry>,
    definition: &WorthQueryPortableApplicationQueryParts,
) {
    let Some(live) = definition.live_cause() else {
        entries.push(null("live-cause"));
        return;
    };
    entries.extend([
        text("live-cause.binding-type", live.binding_type()),
        text("live-cause.effect", live.effect()),
        text("live-cause.payload-type", live.payload_type()),
        text("live-cause.scope-slot-type", live.scope_slot_type()),
        text("live-cause.scope-entity", live.scope_field().0),
        text("live-cause.scope-aspect", live.scope_field().1),
        text("live-cause.scope-field", live.scope_field().2),
        text("live-cause.scope-value-type", live.scope_value_type()),
        text("live-cause.target-slot-type", live.target_slot_type()),
        text("live-cause.target-entity", live.target_field().0),
        text("live-cause.target-aspect", live.target_field().1),
        text("live-cause.target-field", live.target_field().2),
        text("live-cause.target-value-type", live.target_value_type()),
        unsigned_u64(
            "live-cause.maximum-buffered-causes",
            live.resources().maximum_buffered_causes(),
        ),
        unsigned_u64(
            "live-cause.maximum-work-per-delivery",
            live.resources().maximum_work_per_delivery(),
        ),
        unsigned_u64(
            "live-cause.maximum-retained-payload-bytes",
            live.resources().maximum_retained_payload_bytes(),
        ),
    ]);
}

fn append_controls(
    entries: &mut Vec<CanonicalBasisEntry>,
    definition: &WorthQueryPortableApplicationQueryParts,
) {
    let ceiling = definition.dependency_ceiling();
    entries.extend([
        unsigned_usize(
            "controls.maximum-traversal-depth",
            ceiling.maximum_traversal_depth(),
        ),
        unsigned_usize(
            "controls.maximum-relation-count",
            ceiling.maximum_relation_count(),
        ),
        unsigned_usize(
            "controls.maximum-projected-field-count",
            ceiling.maximum_projected_field_count(),
        ),
        text(
            "controls.disclosure-posture",
            disclosure_name(definition.disclosure().posture()),
        ),
        text(
            "controls.disclosure-classification",
            definition.disclosure().classification(),
        ),
        boolean(
            "controls.basis-current",
            definition.basis_support().current(),
        ),
        boolean("controls.basis-pinned", definition.basis_support().pinned()),
        boolean(
            "controls.basis-preview",
            definition.basis_support().preview(),
        ),
        boolean(
            "controls.lane-one-shot",
            definition.lanes().one_shot_enabled(),
        ),
        boolean(
            "controls.lane-historical",
            definition.lanes().historical_enabled(),
        ),
        boolean("controls.lane-live", definition.lanes().live_enabled()),
        boolean(
            "controls.lane-preview",
            definition.lanes().preview_enabled(),
        ),
    ]);
}

const fn ordering_direction_name(value: ApplicationQueryOrderingDirection) -> &'static str {
    match value {
        ApplicationQueryOrderingDirection::Ascending => "ascending",
        ApplicationQueryOrderingDirection::Descending => "descending",
    }
}

const fn disclosure_name(value: ApplicationQueryDisclosurePosture) -> &'static str {
    match value {
        ApplicationQueryDisclosurePosture::Public => "public",
        ApplicationQueryDisclosurePosture::InstalledPolicyRequired => "installed-policy",
        ApplicationQueryDisclosurePosture::Governed => "governed",
    }
}
