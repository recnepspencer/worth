use std::collections::BTreeSet;

use super::{
    inventory_file, protected_declarations, rust_files, source_roots, Producer, PROTECTED,
};

fn production_inventory() -> BTreeSet<Producer> {
    let mut actual = BTreeSet::new();
    for (root, label) in source_roots() {
        for path in rust_files(&root) {
            let relative = path.strip_prefix(&root).expect("source under root");
            let source_name = format!("{label}/{}", relative.display()).replace('\\', "/");
            if source_name.contains("/tests/")
                || source_name.starts_with("bank-server/estate_capability_admission/")
            {
                continue;
            }
            let source = std::fs::read_to_string(&path).expect("read producer source");
            actual.extend(inventory_file(&source_name, &source));
        }
    }
    actual
}

fn allowed_inventory() -> BTreeSet<Producer> {
    include_str!("producer_allowlist.tsv")
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let columns: Vec<_> = line.split('\t').collect();
            assert_eq!(columns.len(), 5, "invalid producer allowlist row: {line}");
            Producer {
                source: columns[0].to_owned(),
                owner: columns[1].to_owned(),
                name: columns[2].to_owned(),
                authority: columns[3].to_owned(),
                shape: match columns[4] {
                    "method" => "method",
                    "free-fn" => "free-fn",
                    other => panic!("unsupported producer shape: {other}"),
                },
            }
        })
        .collect()
}

#[test]
fn protected_authority_producers_match_the_exact_owner_allowlist() {
    assert_eq!(
        production_inventory(),
        allowed_inventory(),
        "protected C7 producer inventory drifted"
    );
}

#[test]
fn every_protected_authority_name_resolves_to_a_real_source_declaration() {
    let mut declarations = BTreeSet::new();
    for (root, _) in source_roots() {
        for path in rust_files(&root) {
            let source = std::fs::read_to_string(&path).expect("read producer source");
            declarations.extend(protected_declarations(&source));
        }
    }
    assert_eq!(
        declarations,
        PROTECTED.iter().map(|name| (*name).to_owned()).collect(),
        "protected C7 inventory contains a phantom name or omits a declaration"
    );

    let phantom = protected_declarations("pub struct RenamedRevalidationObservation;");
    assert!(
        phantom.is_empty(),
        "renaming a protected authority must invalidate catalog completeness"
    );
}

#[test]
fn producer_shape_mutants_are_all_detected() {
    let specimens = [
        "pub fn renamed() -> WorthQueryRequestedElevation { todo!() }",
        "pub const MINT: Option<WorthQueryRequestedElevation> = None;",
        "pub static MINT: Option<WorthQueryRequestedElevation> = None;",
        "pub type Mint = WorthQueryRequestedElevation;",
        "struct WorthQueryRequestedElevation; impl Default for WorthQueryRequestedElevation { fn default() -> Self { Self } }",
        "#[derive(Default)] pub struct WorthQueryRequestedElevation;",
        "pub trait Mint { fn mint() -> WorthQueryRequestedElevation; }",
        "struct Factory; impl Factory { pub(crate) fn renamed() -> WorthQueryRequestedElevation { todo!() } }",
    ];
    for (index, specimen) in specimens.into_iter().enumerate() {
        assert!(
            !inventory_file(&format!("mutant-{index}"), specimen).is_empty(),
            "producer shape mutant {index} escaped"
        );
    }

    for authority in [
        "WorthQueryDelegationActivationBinding",
        "WorthQueryProviderSessionAffinity",
        "WorthQueryProviderCommitAuthorization",
        "WorthQueryRegisteredCommitAuthorization",
    ] {
        let specimen = format!("pub(crate) fn renamed() -> {authority} {{ todo!() }}");
        assert_eq!(
            inventory_file("omitted-authority-mutant", &specimen)
                .into_iter()
                .map(|producer| producer.authority)
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([authority.to_owned()]),
            "broad producer for {authority} escaped the protected catalog"
        );
    }
}
