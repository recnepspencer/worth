use crate::application_schema::{
    application_authorization_path_canonical_components, ApplicationAuthorizationTraversalDirection,
};
use worth_foundational::facade::canonical_basis_value_for_aspect_value;

use super::{
    canonical_components::{append_field, append_relation, structural_count, text, unsigned},
    ApplicationCapabilityCanonicalComponent, ApplicationCapabilityConflictRule,
    ApplicationCapabilityDelegationRule, ApplicationCapabilityDenyRule,
    ApplicationCapabilityDisclosureRule, ApplicationCapabilityDistinctActorRule,
    ApplicationCapabilityGraphRule, ApplicationCapabilityPathContextAnchor,
    ApplicationCapabilityScopeGuard, ApplicationCapabilitySeparationOfDutyRule,
    ErasedApplicationCapabilityContract,
};

pub(super) fn append_composition(
    components: &mut Vec<ApplicationCapabilityCanonicalComponent>,
    contract: &ErasedApplicationCapabilityContract,
) {
    let composition = contract.composition();
    append_graph_rule(
        components,
        "composition.allow",
        composition.decision().allow().graph(),
    );
    append_optional_graph_rule(
        components,
        "composition.deny",
        optional_deny_rule(composition.decision().deny()),
    );
    append_optional_graph_rule(
        components,
        "composition.conflict",
        optional_conflict_rule(composition.decision().conflict()),
    );
    append_optional_graph_rule(
        components,
        "composition.separation-of-duty",
        optional_separation_rule(composition.actors().separation_of_duty()),
    );
    append_optional_graph_rule(
        components,
        "composition.distinct-actor",
        optional_distinct_actor_rule(composition.actors().distinct_actor()),
    );
    text(
        components,
        "composition.recorded-actor".to_owned(),
        composition
            .actors()
            .recorded_actor()
            .requirement()
            .map_or("not-applicable", |r| r.persistence_identity()),
    );
    append_delegation(
        components,
        "composition.delegation",
        composition.propagation().delegation(),
    );
    append_disclosure(
        components,
        "composition.disclosure",
        composition.propagation().disclosure(),
    );
}

fn append_optional_graph_rule(
    components: &mut Vec<ApplicationCapabilityCanonicalComponent>,
    prefix: &str,
    rule: Option<&ApplicationCapabilityGraphRule>,
) {
    match rule {
        None => text(components, format!("{prefix}.posture"), "not-applicable"),
        Some(rule) => {
            text(components, format!("{prefix}.posture"), "graph-rule");
            append_graph_rule(components, prefix, rule);
        }
    }
}

fn append_graph_rule(
    components: &mut Vec<ApplicationCapabilityCanonicalComponent>,
    prefix: &str,
    rule: &ApplicationCapabilityGraphRule,
) {
    structural_count(
        components,
        format!("{prefix}.requirement-count"),
        rule.requirements().len(),
    );
    for (requirement_ordinal, requirement) in rule.requirements().iter().enumerate() {
        let requirement_prefix = format!("{prefix}.requirement[{requirement_ordinal}]");
        structural_count(
            components,
            format!("{requirement_prefix}.alternative-count"),
            requirement.clauses().len(),
        );
        for (clause_ordinal, clause) in requirement.clauses().iter().enumerate() {
            append_graph_clause(
                components,
                &format!("{requirement_prefix}.alternative[{clause_ordinal}]"),
                clause,
            );
        }
    }
}

fn append_graph_clause(
    components: &mut Vec<ApplicationCapabilityCanonicalComponent>,
    clause_prefix: &str,
    clause: &super::ApplicationCapabilityGraphClause,
) {
    for component in application_authorization_path_canonical_components(clause.path()) {
        components.push(ApplicationCapabilityCanonicalComponent::new(
            format!("{clause_prefix}.path.{}", component.locus()),
            component.value().clone(),
        ));
    }
    append_guard(
        components,
        &format!("{clause_prefix}.guard"),
        clause.guard(),
    );
    structural_count(
        components,
        format!("{clause_prefix}.context-anchor-count"),
        clause.context_anchors().len(),
    );
    for (anchor_ordinal, anchor) in clause.context_anchors().iter().enumerate() {
        append_context_anchor(
            components,
            &format!("{clause_prefix}.context-anchor[{anchor_ordinal}]"),
            anchor,
        );
    }
}

fn append_context_anchor(
    components: &mut Vec<ApplicationCapabilityCanonicalComponent>,
    prefix: &str,
    anchor: &ApplicationCapabilityPathContextAnchor,
) {
    append_relation(components, &format!("{prefix}.relation"), anchor.relation());
    text(
        components,
        format!("{prefix}.direction"),
        match anchor.direction() {
            ApplicationAuthorizationTraversalDirection::Forward => "forward",
            ApplicationAuthorizationTraversalDirection::Reverse => "reverse",
        },
    );
    text(
        components,
        format!("{prefix}.slot.context"),
        anchor.slot().context(),
    );
    text(
        components,
        format!("{prefix}.slot.context-type"),
        anchor.slot().context_identity().as_str(),
    );
    text(
        components,
        format!("{prefix}.slot.name"),
        anchor.slot().slot(),
    );
    text(
        components,
        format!("{prefix}.slot.type"),
        anchor.slot().slot_identity().as_str(),
    );
    text(
        components,
        format!("{prefix}.slot.entity"),
        anchor.slot().entity(),
    );
}

fn append_guard(
    components: &mut Vec<ApplicationCapabilityCanonicalComponent>,
    prefix: &str,
    guard: &ApplicationCapabilityScopeGuard,
) {
    structural_count(
        components,
        format!("{prefix}.requirement-count"),
        guard.requirements().len(),
    );
    for (requirement_ordinal, requirement) in guard.requirements().iter().enumerate() {
        let requirement_prefix = format!("{prefix}.requirement[{requirement_ordinal}]");
        append_field(
            components,
            &format!("{requirement_prefix}.field-binding"),
            requirement.field(),
        );
        structural_count(
            components,
            format!("{requirement_prefix}.accepted-value-count"),
            requirement.values().len(),
        );
        for (value_ordinal, value) in requirement.values().iter().enumerate() {
            components.push(ApplicationCapabilityCanonicalComponent::new(
                format!("{requirement_prefix}.accepted-value[{value_ordinal}]"),
                canonical_basis_value_for_aspect_value(value),
            ));
        }
    }
}

fn append_delegation(
    components: &mut Vec<ApplicationCapabilityCanonicalComponent>,
    prefix: &str,
    rule: ApplicationCapabilityDelegationRule,
) {
    match rule {
        ApplicationCapabilityDelegationRule::Forbidden => {
            text(components, format!("{prefix}.posture"), "forbidden");
        }
        ApplicationCapabilityDelegationRule::NarrowAllDimensions { maximum_depth } => {
            text(
                components,
                format!("{prefix}.posture"),
                "narrow-all-dimensions",
            );
            unsigned(
                components,
                format!("{prefix}.maximum-depth"),
                maximum_depth.maximum(),
            );
        }
    }
}

fn append_disclosure(
    components: &mut Vec<ApplicationCapabilityCanonicalComponent>,
    prefix: &str,
    rule: &ApplicationCapabilityDisclosureRule,
) {
    match rule {
        ApplicationCapabilityDisclosureRule::NotApplicable => {
            text(components, format!("{prefix}.posture"), "not-applicable");
        }
        ApplicationCapabilityDisclosureRule::Permit(guards) => {
            text(components, format!("{prefix}.posture"), "permit");
            structural_count(components, format!("{prefix}.guard-count"), guards.len());
            for (guard_ordinal, guard) in guards.iter().enumerate() {
                append_guard(
                    components,
                    &format!("{prefix}.guard[{guard_ordinal}]"),
                    guard,
                );
            }
        }
    }
}

const fn optional_deny_rule(
    rule: &ApplicationCapabilityDenyRule,
) -> Option<&ApplicationCapabilityGraphRule> {
    rule.graph()
}

const fn optional_conflict_rule(
    rule: &ApplicationCapabilityConflictRule,
) -> Option<&ApplicationCapabilityGraphRule> {
    rule.graph()
}

const fn optional_separation_rule(
    rule: &ApplicationCapabilitySeparationOfDutyRule,
) -> Option<&ApplicationCapabilityGraphRule> {
    rule.graph()
}

const fn optional_distinct_actor_rule(
    rule: &ApplicationCapabilityDistinctActorRule,
) -> Option<&ApplicationCapabilityGraphRule> {
    rule.graph()
}
