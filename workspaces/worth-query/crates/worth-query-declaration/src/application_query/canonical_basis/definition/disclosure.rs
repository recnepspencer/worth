use worth_foundational::facade::{
    prepare_aspect_value_identity_basis, AspectMask, CanonicalBasisEntry,
};

use super::super::{
    entry::{boolean, null, text},
    result_shape::{cardinality_name, traversal_direction_name},
};
use crate::application_query::{
    ApplicationQueryDisclosurePosture, ApplicationQueryObservableInfluence,
    WorthQueryPortableApplicationQueryParts,
};

pub(super) fn append_disclosure(
    entries: &mut Vec<CanonicalBasisEntry>,
    definition: &WorthQueryPortableApplicationQueryParts,
) {
    let disclosure = definition.disclosure();
    entries.extend([
        text("disclosure.posture", disclosure_name(disclosure.posture())),
        text("disclosure.classification", disclosure.classification()),
    ]);
    match (disclosure.capability_name(), disclosure.capability_type()) {
        (Some(name), Some(marker_type)) => entries.extend([
            text("disclosure.capability-name", name),
            text("disclosure.capability-type", marker_type),
        ]),
        (None, None) => entries.push(null("disclosure.capability")),
        (Some(name), None) => entries.extend([
            text("disclosure.capability-name", name),
            null("disclosure.capability-type"),
        ]),
        (None, Some(marker_type)) => entries.extend([
            null("disclosure.capability-name"),
            text("disclosure.capability-type", marker_type),
        ]),
    }
    for (index, rule) in disclosure.rules().iter().enumerate() {
        append_rule(entries, index, rule);
    }
}

fn append_rule(
    entries: &mut Vec<CanonicalBasisEntry>,
    index: usize,
    rule: &crate::application_query::ApplicationQueryDisclosureRule,
) {
    let path = format!("disclosure.rule[{index}]");
    let selector = rule.selector();
    entries.extend([
        text(format!("{path}.query-type"), selector.query_type()),
        text(format!("{path}.slot-type"), selector.slot_type()),
        text(format!("{path}.output"), selector.output_name()),
        text(
            format!("{path}.value"),
            prepare_aspect_value_identity_basis(rule.disclosure_value()).as_str(),
        ),
    ]);
    if let Some((entity, aspect, field)) = selector.field_contract() {
        append_field_selector(entries, &path, selector, entity, aspect, field);
    }
    if let Some((relation, from, to, direction, cardinality)) = selector.relation_contract() {
        entries.extend([
            text(format!("{path}.kind"), "relation"),
            text(format!("{path}.relation"), relation),
            text(format!("{path}.from"), from),
            text(format!("{path}.to"), to),
            text(
                format!("{path}.direction"),
                traversal_direction_name(direction),
            ),
            text(format!("{path}.cardinality"), cardinality_name(cardinality)),
        ]);
    }
    if let crate::application_query::ApplicationQueryDisclosureSelector::Orbit(orbit) = selector {
        entries.extend([
            text(format!("{path}.kind"), "orbit"),
            text(format!("{path}.relation"), orbit.relation()),
            text(format!("{path}.from"), orbit.from()),
            text(format!("{path}.to"), orbit.to()),
            text(format!("{path}.order"), "anchor-forward-v1"),
        ]);
    }
    for (index, surface) in rule.influence().permitted().enumerate() {
        entries.push(text(
            format!("{path}.influence[{index}]"),
            influence_name(surface),
        ));
    }
}

fn append_field_selector(
    entries: &mut Vec<CanonicalBasisEntry>,
    path: &str,
    selector: &crate::application_query::ApplicationQueryDisclosureSelector,
    entity: &str,
    aspect: &str,
    field: &str,
) {
    entries.extend([
        text(
            format!("{path}.kind"),
            if selector.is_internal_computation() {
                "internal-field"
            } else {
                "result-field"
            },
        ),
        text(format!("{path}.entity"), entity),
        text(format!("{path}.aspect"), aspect),
        text(format!("{path}.field"), field),
    ]);
    append_mask(entries, path, "projection", selector.projection_mask());
    append_mask(entries, path, "diagnostic", selector.diagnostic_mask());
}

fn append_mask<Mode>(
    entries: &mut Vec<CanonicalBasisEntry>,
    rule_path: &str,
    name: &str,
    mask: Option<&AspectMask<Mode>>,
) {
    let Some(mask) = mask else {
        return;
    };
    entries.push(boolean(
        format!("{rule_path}.{name}.whole-aspect"),
        mask.is_whole_aspect(),
    ));
    for (path_index, field_path) in mask.paths().iter().enumerate() {
        for (field_index, field) in field_path.fields().iter().enumerate() {
            entries.push(text(
                format!("{rule_path}.{name}.path[{path_index}].field[{field_index}]"),
                field.as_str(),
            ));
        }
    }
}

const fn disclosure_name(value: ApplicationQueryDisclosurePosture) -> &'static str {
    match value {
        ApplicationQueryDisclosurePosture::Public => "public",
        ApplicationQueryDisclosurePosture::InstalledPolicyRequired => "installed-policy",
        ApplicationQueryDisclosurePosture::Governed => "governed",
    }
}

const fn influence_name(value: ApplicationQueryObservableInfluence) -> &'static str {
    match value {
        ApplicationQueryObservableInfluence::RowPresence => "row-presence",
        ApplicationQueryObservableInfluence::Ordering => "ordering",
        ApplicationQueryObservableInfluence::Pagination => "pagination",
        ApplicationQueryObservableInfluence::Count => "count",
        ApplicationQueryObservableInfluence::Aggregate => "aggregate",
        ApplicationQueryObservableInfluence::Explanation => "explanation",
        ApplicationQueryObservableInfluence::HistoricalMembership => "historical-membership",
        ApplicationQueryObservableInfluence::Preview => "preview",
        ApplicationQueryObservableInfluence::LiveMembership => "live-membership",
    }
}
