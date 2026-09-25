//! The authored element table and the DSL modules under `app/` describe the
//! same components. An element without a declaration has no graph node, so
//! the product rejects its own copy at start-up; a declaration without an
//! element paints nothing and can never be laid out.
use std::collections::BTreeSet;
use std::path::Path;

use super::{dashboard_containers, dashboard_elements, DashboardScrollPanel};

/// Components declared outside the element table on purpose: the status band,
/// the two scroll owners, and the layout containers, which are registered by
/// the structure owners rather than authored as painted elements.
fn declared_without_an_element() -> BTreeSet<String> {
    let mut allowed = BTreeSet::from(["platform.pulse.component.status_band".to_owned()]);
    for panel in DashboardScrollPanel::ALL {
        allowed.insert(format!("platform.pulse.component.{}", panel.owner()));
    }
    for container in dashboard_containers() {
        allowed.insert(container.component().as_str().to_owned());
    }
    allowed
}

fn declared_components() -> BTreeSet<String> {
    let app = Path::new(env!("CARGO_MANIFEST_DIR")).join("app");
    let mut declared = BTreeSet::new();
    for entry in std::fs::read_dir(&app).expect("the app directory holds the DSL modules") {
        let path = entry.expect("readable app directory entry").path();
        if path.extension().is_none_or(|extension| extension != "wui") {
            continue;
        }
        let source = std::fs::read_to_string(&path).expect("readable DSL module");
        for line in source.lines() {
            let Some(rest) = line.strip_prefix("component ") else {
                continue;
            };
            let identity = rest.split_whitespace().next().expect("a declared identity");
            assert!(
                declared.insert(identity.to_owned()),
                "{identity} is declared twice under app/"
            );
        }
    }
    declared
}

#[test]
fn every_authored_element_has_exactly_one_dsl_component_declaration() {
    let declared = declared_components();
    let mut authored = BTreeSet::new();
    for element in dashboard_elements() {
        let component = element.component_id();
        assert!(
            declared.contains(&component),
            "{component} is authored in the element table but declared in no app/*.wui"
        );
        assert!(
            authored.insert(component.clone()),
            "{component} is authored twice in the element table"
        );
    }
    let allowed = declared_without_an_element();
    for component in declared.difference(&authored) {
        assert!(
            allowed.contains(component),
            "{component} is declared under app/ but no authored element paints it"
        );
    }
}
