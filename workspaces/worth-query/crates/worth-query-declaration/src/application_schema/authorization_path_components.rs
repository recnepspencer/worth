use worth_foundational::facade::{
    canonical_basis_value_for_aspect_value, CanonicalBasisValue, CanonicalIntegerWidth,
};

use super::{
    ApplicationAuthorizationComparison, ApplicationAuthorizationPath,
    ApplicationAuthorizationPathEffect, ApplicationAuthorizationTraversalDirection,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationAuthorizationPathCanonicalComponent {
    locus: String,
    value: CanonicalBasisValue,
}

impl ApplicationAuthorizationPathCanonicalComponent {
    pub fn locus(&self) -> &str {
        &self.locus
    }

    pub const fn value(&self) -> &CanonicalBasisValue {
        &self.value
    }
}

pub fn application_authorization_path_canonical_components(
    path: &ApplicationAuthorizationPath,
) -> Vec<ApplicationAuthorizationPathCanonicalComponent> {
    let mut components = Vec::with_capacity(
        5 + path.traversals().len().saturating_mul(4) + path.predicates().len().saturating_mul(5),
    );
    text(&mut components, "effect", effect_name(path.effect()));
    text(&mut components, "principal", path.principal_entity());
    text(&mut components, "scope", path.scope_entity());
    unsigned(&mut components, "traversal-count", path.traversals().len());
    for (ordinal, traversal) in path.traversals().iter().enumerate() {
        let prefix = format!("traversal.{ordinal}");
        text(
            &mut components,
            format!("{prefix}.relation"),
            traversal.relation(),
        );
        text(&mut components, format!("{prefix}.from"), traversal.from());
        text(&mut components, format!("{prefix}.to"), traversal.to());
        text(
            &mut components,
            format!("{prefix}.direction"),
            direction_name(traversal.direction()),
        );
    }
    unsigned(&mut components, "predicate-count", path.predicates().len());
    for (ordinal, predicate) in path.predicates().iter().enumerate() {
        let prefix = format!("predicate.{ordinal}");
        unsigned(
            &mut components,
            format!("{prefix}.traversal-ordinal"),
            predicate.traversal_ordinal(),
        );
        text(
            &mut components,
            format!("{prefix}.entity"),
            predicate.entity(),
        );
        text(
            &mut components,
            format!("{prefix}.aspect"),
            predicate.aspect(),
        );
        text(
            &mut components,
            format!("{prefix}.field"),
            predicate.field(),
        );
        components.push(component(
            format!("{prefix}.value"),
            canonical_basis_value_for_aspect_value(predicate.value()),
        ));
        // Equality is the legacy meaning; only another comparison adds a component.
        if predicate.comparison() != ApplicationAuthorizationComparison::Equal {
            text(
                &mut components,
                format!("{prefix}.comparison"),
                predicate.comparison().canonical_name(),
            );
        }
    }
    // Paths without principal-field constraints keep their legacy identity.
    if !path.field_constraints().is_empty() {
        unsigned(
            &mut components,
            "field-constraint-count",
            path.field_constraints().len(),
        );
    }
    for (ordinal, constraint) in path.field_constraints().iter().enumerate() {
        let prefix = format!("field-constraint.{ordinal}");
        unsigned(
            &mut components,
            format!("{prefix}.traversal-ordinal"),
            constraint.traversal_ordinal(),
        );
        for (name, value) in [
            ("entity", constraint.entity()),
            ("aspect", constraint.aspect()),
            ("field", constraint.field()),
            ("comparison", constraint.comparison().canonical_name()),
            ("principal-aspect", constraint.principal_aspect()),
            ("principal-field", constraint.principal_field()),
        ] {
            text(&mut components, format!("{prefix}.{name}"), value);
        }
    }
    components
}

fn text(
    components: &mut Vec<ApplicationAuthorizationPathCanonicalComponent>,
    locus: impl Into<String>,
    value: impl AsRef<str>,
) {
    components.push(component(
        locus,
        CanonicalBasisValue::ExactText(value.as_ref().to_owned().into()),
    ));
}

fn unsigned(
    components: &mut Vec<ApplicationAuthorizationPathCanonicalComponent>,
    locus: impl Into<String>,
    value: usize,
) {
    components.push(component(
        locus,
        CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: u64::try_from(value)
                .expect("authorization path structural counts fit in u64")
                .into(),
        },
    ));
}

fn component(
    locus: impl Into<String>,
    value: CanonicalBasisValue,
) -> ApplicationAuthorizationPathCanonicalComponent {
    ApplicationAuthorizationPathCanonicalComponent {
        locus: locus.into(),
        value,
    }
}

const fn effect_name(effect: ApplicationAuthorizationPathEffect) -> &'static str {
    match effect {
        ApplicationAuthorizationPathEffect::Allow => "allow",
        ApplicationAuthorizationPathEffect::Deny => "deny",
    }
}

const fn direction_name(direction: ApplicationAuthorizationTraversalDirection) -> &'static str {
    match direction {
        ApplicationAuthorizationTraversalDirection::Forward => "forward",
        ApplicationAuthorizationTraversalDirection::Reverse => "reverse",
    }
}
