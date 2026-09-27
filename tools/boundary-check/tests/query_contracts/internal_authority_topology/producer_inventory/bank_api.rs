use std::collections::BTreeSet;

use quote::ToTokens;
use syn::{Fields, ImplItem, Item, TraitItem, UseTree, Visibility};

use super::{rust_files, source_roots};

const RAW_QUERY_ELEVATION_TYPES: &[&str] = &[
    "WorthQueryRequestedElevation",
    "WorthQueryApprovedElevation",
    "WorthQueryMandatoryReview",
    "WorthQueryReviewedElevation",
    "WorthQueryElevationRequestOutcome",
    "WorthQueryElevationApprovalOutcome",
    "WorthQueryElevationCloseOutcome",
    "WorthQueryMandatoryReviewOutcome",
    // The runtimes that can redeem a commit receipt for recovery or outbox
    // authority. Pinning them here is what keeps the receipt itself ordinary.
    "WorthQueryPrimaryGraphApplicationRuntime",
    "WorthQueryProgramApplicationRuntime",
    "WorthQueryWorkflowApplicationRuntime",
];
// The commit receipt is not on this list: since 9.17.4 it is the ordinary
// Query mutation outcome Bank hands every caller, and only the runtimes above
// can turn it into authority. No public Bank item names a runtime (9.17.6
// slice 5.6 moved Bank's callers onto typed lanes), so no caller outside Bank
// can redeem a receipt.

fn names_raw_query_elevation(tokens: impl ToTokens, aliases: &BTreeSet<String>) -> bool {
    let source = tokens.to_token_stream().to_string();
    RAW_QUERY_ELEVATION_TYPES
        .iter()
        .any(|protected| source.contains(protected))
        || source
            .split(|character: char| !(character.is_alphanumeric() || character == '_'))
            .any(|word| aliases.contains(word))
}

/// Local names Bank gives a protected type, through `use ... as` or a type
/// alias of any visibility, so a public item cannot hide one behind a rename.
/// A name for a name counts too, so aliases are followed until none is new.
fn raw_query_elevation_aliases(sources: &[&str]) -> BTreeSet<String> {
    fn renames(tree: &UseTree, aliases: &mut BTreeSet<String>) {
        match tree {
            UseTree::Path(path) => renames(&path.tree, aliases),
            UseTree::Group(group) => group.items.iter().for_each(|tree| renames(tree, aliases)),
            UseTree::Rename(rename) => {
                let target = rename.ident.to_string();
                if RAW_QUERY_ELEVATION_TYPES.contains(&target.as_str()) || aliases.contains(&target)
                {
                    aliases.insert(rename.rename.to_string());
                }
            }
            _ => {}
        }
    }
    fn visit(items: &[Item], aliases: &mut BTreeSet<String>) {
        for item in items {
            match item {
                Item::Use(item) => renames(&item.tree, aliases),
                Item::Type(item) if names_raw_query_elevation(&item.ty, aliases) => {
                    aliases.insert(item.ident.to_string());
                }
                Item::Mod(module) => {
                    if let Some((_, content)) = &module.content {
                        visit(content, aliases);
                    }
                }
                _ => {}
            }
        }
    }
    let files = sources
        .iter()
        .map(|source| syn::parse_file(source).expect("Bank source must parse"))
        .collect::<Vec<_>>();
    let mut aliases = BTreeSet::new();
    loop {
        let known = aliases.len();
        for file in &files {
            visit(&file.items, &mut aliases);
        }
        if aliases.len() == known {
            return aliases;
        }
    }
}

fn public_bank_items_naming_raw_query_elevation(
    source: &str,
    aliases: &BTreeSet<String>,
) -> Vec<String> {
    let syntax = syn::parse_file(source).expect("Bank source must parse");
    let mut escaped = Vec::new();
    public_items_naming_raw_query_elevation(&syntax.items, aliases, &mut escaped);
    escaped
}

fn public_items_naming_raw_query_elevation(
    items: &[Item],
    aliases: &BTreeSet<String>,
    escaped: &mut Vec<String>,
) {
    let names =
        |tokens: &dyn ToTokens| names_raw_query_elevation(tokens.to_token_stream(), aliases);
    for item in items {
        match item {
            Item::Fn(function) if matches!(function.vis, Visibility::Public(_)) => {
                if names(&function.sig) {
                    escaped.push(function.sig.ident.to_string());
                }
            }
            // A trait impl's members are as visible as the trait, and its
            // header or an associated type (`Deref::Target`) can hand out a
            // runtime a Bank value holds.
            Item::Impl(implementation) if implementation.trait_.is_some() => {
                let (_, path, _) = implementation.trait_.as_ref().expect("trait impl");
                if names(path)
                    || implementation.items.iter().any(|member| match member {
                        ImplItem::Type(item) => names(&item.ty),
                        ImplItem::Fn(function) => names(&function.sig),
                        ImplItem::Const(item) => names(&item.ty),
                        _ => false,
                    })
                {
                    escaped.push(format!("impl {}", path.to_token_stream()));
                }
            }
            Item::Impl(implementation) => {
                for member in &implementation.items {
                    let ImplItem::Fn(function) = member else {
                        continue;
                    };
                    if matches!(function.vis, Visibility::Public(_)) && names(&function.sig) {
                        escaped.push(function.sig.ident.to_string());
                    }
                }
            }
            Item::Static(item) if matches!(item.vis, Visibility::Public(_)) => {
                if names(&item.ty) {
                    escaped.push(item.ident.to_string());
                }
            }
            Item::Const(item) if matches!(item.vis, Visibility::Public(_)) => {
                if names(&item.ty) {
                    escaped.push(item.ident.to_string());
                }
            }
            Item::Struct(item) if matches!(item.vis, Visibility::Public(_)) => {
                let fields = match &item.fields {
                    Fields::Named(fields) => fields.named.iter().collect::<Vec<_>>(),
                    Fields::Unnamed(fields) => fields.unnamed.iter().collect::<Vec<_>>(),
                    Fields::Unit => Vec::new(),
                };
                if fields
                    .into_iter()
                    .any(|field| matches!(field.vis, Visibility::Public(_)) && names(&field.ty))
                {
                    escaped.push(item.ident.to_string());
                }
            }
            Item::Enum(item) if matches!(item.vis, Visibility::Public(_)) => {
                if item
                    .variants
                    .iter()
                    .any(|variant| variant.fields.iter().any(|field| names(&field.ty)))
                {
                    escaped.push(item.ident.to_string());
                }
            }
            Item::Type(item) if matches!(item.vis, Visibility::Public(_)) => {
                if names(&item.ty) {
                    escaped.push(item.ident.to_string());
                }
            }
            Item::Trait(item) if matches!(item.vis, Visibility::Public(_)) => {
                if item
                    .items
                    .iter()
                    .any(|member| matches!(member, TraitItem::Fn(function) if names(&function.sig)))
                {
                    escaped.push(item.ident.to_string());
                }
            }
            Item::Use(item) if matches!(item.vis, Visibility::Public(_)) => {
                if names(&item.tree) {
                    escaped.push(format!("use {}", item.tree.to_token_stream()));
                }
            }
            Item::Mod(module) => {
                if let Some((_, content)) = &module.content {
                    public_items_naming_raw_query_elevation(content, aliases, escaped);
                }
            }
            _ => {}
        }
    }
}

#[test]
fn public_bank_api_never_exposes_raw_query_elevation_authority() {
    let (bank_root, _) = source_roots().into_iter().nth(2).expect("Bank root");
    let sources = rust_files(&bank_root)
        .into_iter()
        .filter_map(|path| {
            let relative = path.strip_prefix(&bank_root).expect("source under root");
            let relative = relative.to_string_lossy().replace('\\', "/");
            (!relative.starts_with("estate_capability_admission/")).then(|| {
                (
                    relative,
                    std::fs::read_to_string(&path).expect("read Bank source"),
                )
            })
        })
        .collect::<Vec<_>>();
    // A crate-level alias can be named from any Bank file.
    let aliases = raw_query_elevation_aliases(
        &sources
            .iter()
            .map(|(_, source)| source.as_str())
            .collect::<Vec<_>>(),
    );
    let escaped = sources
        .iter()
        .flat_map(|(relative, source)| {
            public_bank_items_naming_raw_query_elevation(source, &aliases)
                .into_iter()
                .map(move |signature| format!("{relative}::{signature}"))
        })
        .collect::<Vec<_>>();
    assert!(
        escaped.is_empty(),
        "raw Query elevation escaped the public Bank API: {escaped:?}"
    );

    let mutants = [
        "struct Runtime; impl Runtime { pub fn renamed(value: WorthQueryApprovedElevation) {} }",
        "pub enum Outcome { Leaked(WorthQueryRequestedElevation) }",
        "pub struct Output { pub leaked: WorthQueryMandatoryReview }",
        "pub type Receipt = WorthQueryReviewedElevation;",
        "pub trait Route { fn leak() -> WorthQueryElevationCloseOutcome; }",
        "pub use host::WorthQueryProgramApplicationRuntime as BankProgramRuntime;",
        "pub mod leak { pub fn runtime() -> WorthQueryProgramApplicationRuntime { todo!() } }",
        "use host::WorthQueryWorkflowApplicationRuntime as Hidden; pub fn leak() -> Hidden { todo!() }",
        "type Hidden = WorthQueryPrimaryGraphApplicationRuntime; pub fn leak() -> Hidden { todo!() }",
        "impl Deref for Bank { type Target = WorthQueryProgramApplicationRuntime; fn deref(&self) -> &Self::Target { todo!() } }",
        "impl AsRef<WorthQueryWorkflowApplicationRuntime> for Bank { fn as_ref(&self) -> &Self { self } }",
        "use host::WorthQueryProgramApplicationRuntime as First; type Second = First; pub fn leak() -> Second { todo!() }",
        "type First = WorthQueryProgramApplicationRuntime; use self::First as Second; pub fn leak() -> Second { todo!() }",
        "pub static LEAKED: OnceLock<WorthQueryPrimaryGraphApplicationRuntime> = OnceLock::new();",
        "pub const LEAKED: Option<WorthQueryRequestedElevation> = None;",
    ];
    for (index, mutant) in mutants.into_iter().enumerate() {
        assert_eq!(
            public_bank_items_naming_raw_query_elevation(
                mutant,
                &raw_query_elevation_aliases(&[mutant])
            )
            .len(),
            1,
            "Bank API mutant {index} escaped"
        );
    }
}
