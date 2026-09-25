//! The authored element table and the DSL modules under `app/` describe the
//! same components. An element without a declaration has no graph node, so
//! the product rejects its own copy at start-up; a declaration without an
//! element paints nothing and can never be laid out.
use std::collections::BTreeSet;
use std::path::Path;

use super::{dashboard_containers, dashboard_elements, DashboardScrollPanel};
use crate::product_world::PlatformPulseMosaicRegion;

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

fn dsl_sources() -> Vec<String> {
    let app = Path::new(env!("CARGO_MANIFEST_DIR")).join("app");
    let mut sources = Vec::new();
    for entry in std::fs::read_dir(&app).expect("the app directory holds the DSL modules") {
        let path = entry.expect("readable app directory entry").path();
        if path.extension().is_some_and(|extension| extension == "wui") {
            sources.push(std::fs::read_to_string(&path).expect("readable DSL module"));
        }
    }
    sources
}

fn declared_components() -> BTreeSet<String> {
    let mut declared = BTreeSet::new();
    for source in dsl_sources() {
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

/// A layout container without a declaration has no graph node, so layout
/// refuses the product at start-up.
#[test]
fn every_layout_container_has_a_dsl_component_declaration() {
    let declared = declared_components();
    for container in dashboard_containers() {
        let component = container.component().as_str().to_owned();
        assert!(
            declared.contains(&component),
            "{component} is a layout container but declared in no app/*.wui"
        );
    }
}

/// Every Mosaic region an `app/*.wui` component mounts, paired with that
/// component, nested regions included.
fn mounted_regions() -> BTreeSet<(String, String)> {
    let mut mounted = BTreeSet::new();
    for source in dsl_sources() {
        let mut owner = None;
        for line in source.lines() {
            if let Some(rest) = line.strip_prefix("component ") {
                owner = rest.split_whitespace().next().map(str::to_owned);
            } else if line.starts_with('}') {
                owner = None;
            } else if let (Some(owner), Some(rest)) =
                (&owner, line.trim_start().strip_prefix("region "))
            {
                let region = rest.split_whitespace().next().expect("a mounted region");
                mounted.insert((owner.clone(), region.to_owned()));
            }
        }
    }
    mounted
}

/// Layout places a region only where its owner declares a placement, so each
/// mounted region is exactly one the product places for that owner: the
/// surface regions on the seed, and each list region on its panel owner.
#[test]
fn every_mounted_region_has_a_placement_from_its_owner() {
    let mut placed = BTreeSet::new();
    for region in PlatformPulseMosaicRegion::SURFACE {
        assert!(region.surface_placement().is_some(), "{region:?}");
        placed.insert((
            "platform.pulse.component.seed".to_owned(),
            region.id().to_owned(),
        ));
    }
    for panel in DashboardScrollPanel::ALL {
        placed.insert((
            format!("platform.pulse.component.{}", panel.owner()),
            panel.region().to_owned(),
        ));
    }
    assert_eq!(mounted_regions(), placed);
}
