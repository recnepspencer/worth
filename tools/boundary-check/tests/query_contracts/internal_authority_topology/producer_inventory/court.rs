use std::collections::BTreeSet;

use quote::ToTokens;
use syn::{Fields, ImplItem, Item, TraitItem, Visibility};

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

const RAW_QUERY_ELEVATION_TYPES: &[&str] = &[
    "WorthQueryRequestedElevation",
    "WorthQueryApprovedElevation",
    "WorthQueryMandatoryReview",
    "WorthQueryReviewedElevation",
    "WorthQueryElevationRequestOutcome",
    "WorthQueryElevationApprovalOutcome",
    "WorthQueryElevationCloseOutcome",
    "WorthQueryMandatoryReviewOutcome",
];
// The commit receipt is not on this list: since 9.17.4 it is the ordinary
// Query mutation outcome Bank hands every caller, and the Query runtime that
// could turn it into recovery or outbox authority stays crate-private.

fn names_raw_query_elevation(tokens: impl ToTokens) -> bool {
    let source = tokens.to_token_stream().to_string();
    RAW_QUERY_ELEVATION_TYPES
        .iter()
        .any(|protected| source.contains(protected))
}

fn public_bank_items_naming_raw_query_elevation(source: &str) -> Vec<String> {
    let syntax = syn::parse_file(source).expect("Bank source must parse");
    let mut escaped = Vec::new();
    for item in syntax.items {
        match item {
            Item::Fn(function) if matches!(function.vis, Visibility::Public(_)) => {
                if names_raw_query_elevation(&function.sig) {
                    escaped.push(function.sig.ident.to_string());
                }
            }
            Item::Impl(implementation) => {
                for member in implementation.items {
                    let ImplItem::Fn(function) = member else {
                        continue;
                    };
                    if matches!(function.vis, Visibility::Public(_))
                        && names_raw_query_elevation(&function.sig)
                    {
                        escaped.push(function.sig.ident.to_string());
                    }
                }
            }
            Item::Struct(item) if matches!(item.vis, Visibility::Public(_)) => {
                let fields = match item.fields {
                    Fields::Named(fields) => fields.named.into_iter().collect::<Vec<_>>(),
                    Fields::Unnamed(fields) => fields.unnamed.into_iter().collect::<Vec<_>>(),
                    Fields::Unit => Vec::new(),
                };
                if fields.into_iter().any(|field| {
                    matches!(field.vis, Visibility::Public(_))
                        && names_raw_query_elevation(&field.ty)
                }) {
                    escaped.push(item.ident.to_string());
                }
            }
            Item::Enum(item) if matches!(item.vis, Visibility::Public(_)) => {
                if item.variants.iter().any(|variant| {
                    variant.fields.iter().any(|field| names_raw_query_elevation(&field.ty))
                }) {
                    escaped.push(item.ident.to_string());
                }
            }
            Item::Type(item) if matches!(item.vis, Visibility::Public(_)) => {
                if names_raw_query_elevation(&item.ty) {
                    escaped.push(item.ident.to_string());
                }
            }
            Item::Trait(item) if matches!(item.vis, Visibility::Public(_)) => {
                if item.items.iter().any(|member| {
                    matches!(member, TraitItem::Fn(function) if names_raw_query_elevation(&function.sig))
                }) {
                    escaped.push(item.ident.to_string());
                }
            }
            _ => {}
        }
    }
    escaped
}

#[test]
fn public_bank_api_never_exposes_raw_query_elevation_authority() {
    let (bank_root, _) = source_roots().into_iter().nth(2).expect("Bank root");
    let mut escaped = Vec::new();
    for path in rust_files(&bank_root) {
        let relative = path.strip_prefix(&bank_root).expect("source under root");
        let relative = relative.to_string_lossy().replace('\\', "/");
        if relative.starts_with("estate_capability_admission/") {
            continue;
        }
        let source = std::fs::read_to_string(&path).expect("read Bank source");
        for signature in public_bank_items_naming_raw_query_elevation(&source) {
            escaped.push(format!("{relative}::{signature}"));
        }
    }
    assert!(
        escaped.is_empty(),
        "raw Query elevation escaped Bank: {escaped:?}"
    );

    let mutants = [
        "struct Runtime; impl Runtime { pub fn renamed(value: WorthQueryApprovedElevation) {} }",
        "pub enum Outcome { Leaked(WorthQueryRequestedElevation) }",
        "pub struct Output { pub leaked: WorthQueryMandatoryReview }",
        "pub type Receipt = WorthQueryReviewedElevation;",
        "pub trait Route { fn leak() -> WorthQueryElevationCloseOutcome; }",
    ];
    for (index, mutant) in mutants.into_iter().enumerate() {
        assert_eq!(
            public_bank_items_naming_raw_query_elevation(mutant).len(),
            1,
            "Bank API mutant {index} escaped"
        );
    }
}
