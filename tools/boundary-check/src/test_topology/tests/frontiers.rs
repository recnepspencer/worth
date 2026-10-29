//! Adversarial graph-derived naming and area-owned ceiling exceptions.
use super::super::{frontier_splits, CONFIG};
use super::fixture::Fixture;

fn workspace(courts: &[&str]) -> Fixture {
    let f = Fixture::new();
    f.write(CONFIG, ". | sample\n");
    let members = std::iter::once("product")
        .chain(courts.iter().copied())
        .map(|name| format!("'{name}'"))
        .collect::<Vec<_>>()
        .join(",");
    f.write(
        "Cargo.toml",
        &format!("[workspace]\nmembers=[{members}]\nresolver='3'\n"),
    );
    f.write(
        "product/Cargo.toml",
        "[package]\nname='sample-host'\nversion='0.1.0'\n",
    );
    f.write("product/src/lib.rs", "pub struct Host;\n");
    for name in courts {
        let package = format!("sample-court-{name}");
        let target = package.replace('-', "_");
        f.write(&format!("{name}/Cargo.toml"), &format!("[package]\nname='{package}'\nversion='0.1.0'\nautotests=false\n[dev-dependencies]\nhost={{package='sample-host',path='../product'}}\n[[test]]\nname='{target}'\npath='tests/{target}.rs'\n"));
        f.write(
            &format!("{name}/tests/{target}.rs"),
            "#[test] fn lawful() {}\n",
        );
    }
    f
}

fn split(court: &str, area: &str, owner: &str) -> String {
    format!("[[splits]]\ncourt='sample-court-{court}'\nfrontier=['sample-host']\narea='{area}'\nowners=['{owner}']\n")
}

#[test]
fn equal_product_frontiers_refuse_a_second_court_despite_incidental_dependencies() {
    let f = workspace(&["host", "host-other"]);
    f.append("Cargo.toml", "exclude=['incidental']\n");
    f.append(
        "host-other/Cargo.toml",
        "[dev-dependencies.incidental]\npath='../incidental'\n",
    );
    f.write(
        "incidental/Cargo.toml",
        "[package]\nname='incidental'\nversion='0.1.0'\n[workspace]\n",
    );
    f.write("incidental/src/lib.rs", "");
    f.refuses("equal product frontiers");
}

#[test]
fn a_court_name_cannot_claim_a_different_product_frontier() {
    let f = workspace(&["declaration"]);
    f.refuses("court name does not match product frontier");
}

#[test]
fn a_matching_frontier_needs_no_self_declared_surface_metadata() {
    let f = workspace(&["host"]);
    assert!(f.messages().is_empty(), "{:?}", f.messages());
}

#[test]
fn a_split_registration_cannot_bypass_the_source_ceiling() {
    let f = workspace(&["host-first", "host-second"]);
    f.write(
        frontier_splits::CONFIG,
        &(split("host-first", "first", "first") + &split("host-second", "second", "second")),
    );
    f.refuses("combined 2");
}

#[test]
fn only_required_disjoint_registered_areas_may_share_a_frontier() {
    let f = workspace(&["host-first", "host-second"]);
    f.write(
        frontier_splits::CONFIG,
        &(split("host-first", "first", "first") + &split("host-second", "second", "second")),
    );
    // Mounted area files model owned compiler-unit size; no test is ignored.
    for (court, area) in [("host-first", "first"), ("host-second", "second")] {
        let target = format!("sample-court-{court}").replace('-', "_");
        f.write(
            &format!("{court}/tests/{target}.rs"),
            &format!("#[path=\"{target}/{area}/mod.rs\"] mod {area};\n"),
        );
        let mut declarations = "#[test] fn lawful() {}\n".to_owned();
        for index in 0..20 {
            declarations.push_str(&format!("mod part_{index};\n"));
            f.write(
                &format!("{court}/tests/{target}/{area}/part_{index}.rs"),
                &"// owned source\n".repeat(399),
            );
        }
        f.write(
            &format!("{court}/tests/{target}/{area}/mod.rs"),
            &declarations,
        );
    }
    assert!(f.messages().is_empty(), "{:?}", f.messages());
    f.write(
        frontier_splits::CONFIG,
        &(split("host-first", "first", "same") + &split("host-second", "second", "same")),
    );
    f.refuses("overlapping area owners");
}

#[test]
fn stale_or_falsely_named_area_registrations_are_refused() {
    let f = workspace(&["host"]);
    f.write(
        frontier_splits::CONFIG,
        &split("host-missing", "missing", "missing"),
    );
    f.refuses("absent court");
    f.write(
        frontier_splits::CONFIG,
        &split("host", "root", "root").replace("sample-host", "sample-decl"),
    );
    f.refuses("registers a different product frontier");
}

#[test]
fn an_area_registration_cannot_survive_removing_its_court() {
    let f = workspace(&[]);
    f.write(frontier_splits::CONFIG, &split("host", "root", "root"));
    f.refuses("absent court");
}
