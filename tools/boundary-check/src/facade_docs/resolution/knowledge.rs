//! Closed negative knowledge is deliberately narrower than rustdoc attribution.
use super::{CrateIndex, Definition, Resolver};
use std::path::Path;
use syn::Item;

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Knowledge {
    Present,
    ProvenAbsent,
    Unsupported(String),
}
#[derive(Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SelectorKind {
    ModuleExport,
    EnumVariant,
    AssociatedMember,
}

fn benign(attribute: &syn::Attribute) -> bool {
    // This exact crate metadata does not remove or transform source items.
    if !attribute.path().is_ident("cfg_attr") {
        return false;
    }
    let Ok(arguments) = attribute.parse_args_with(
        syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
    ) else {
        return false;
    };
    let values = arguments.iter().collect::<Vec<_>>();
    if let [syn::Meta::Path(condition), syn::Meta::NameValue(value)] = values.as_slice() {
        return condition.is_ident("doc")
            && value.path.is_ident("recursion_limit")
            && matches!(&value.value, syn::Expr::Lit(lit) if matches!(&lit.lit, syn::Lit::Str(s) if s.value() == "256"));
    }
    // The condition is deliberately not evaluated: every possible expansion
    // contains lint metadata only, so neither branch changes source shape.
    let Some((_, metadata)) = values.split_first() else {
        return false;
    };
    !metadata.is_empty()
        && metadata.iter().all(|meta| {
            let syn::Meta::List(lint) = meta else {
                return false;
            };
            ["allow", "warn", "deny", "forbid"]
                .iter()
                .any(|name| lint.path.is_ident(name))
                && lint
                    .parse_args_with(
                        syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                    )
                    .is_ok_and(|paths| !paths.is_empty())
        })
}
fn conditional(attrs: &[syn::Attribute]) -> bool {
    attrs
        .iter()
        .any(|a| a.path().is_ident("cfg") || (a.path().is_ident("cfg_attr") && !benign(a)))
}
fn module_attributes(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().all(|a| {
        benign(a)
            || (a.path().is_ident("macro_use") && matches!(a.meta, syn::Meta::Path(_)))
            || [
                "doc",
                "allow",
                "warn",
                "deny",
                "forbid",
                "path",
                "recursion_limit",
            ]
            .iter()
            .any(|name| a.path().is_ident(name))
    })
}

pub(crate) fn enum_definition(
    root: &Path,
    manifest: &str,
    path: &str,
    crates: &CrateIndex,
) -> Result<syn::ItemEnum, String> {
    if !path.starts_with("facade::") {
        return Err("facts must enter through an audience facade".into());
    }
    let resolver = Resolver::new(crates);
    let id = crates.load(root.join(manifest).parent().ok_or("missing crate root")?)?;
    let segments = path.split("::").map(str::to_owned).collect::<Vec<_>>();
    require_fact_route(&crates, &resolver, id, &segments)?;
    let definitions = resolver.resolve_public_path(id, &segments)?;
    let [Definition::Item {
        krate,
        module,
        index,
    }] = definitions.as_slice()
    else {
        return Err("fact source must resolve to exactly one real enum definition".into());
    };
    require_definition_source(crates, &definitions[0])?;
    let owner = crates.crate_at(*krate);
    let Some(Item::Enum(item)) = owner
        .graph
        .modules
        .get(module)
        .and_then(|n| n.items.get(*index))
    else {
        return Err("fact source is not an enum".into());
    };
    if !matches!(&item.vis, syn::Visibility::Public(_)) {
        return Err("fact enum is not public".into());
    }
    if !fact_attributes(&item.attrs)
        || item.variants.iter().any(|v| {
            !fact_attributes(&v.attrs)
                || v.fields.iter().any(|field| !fact_attributes(&field.attrs))
        })
    {
        return Err("conditional enum facts require an owning emitter".into());
    }
    Ok(item.clone())
}

/// Public route attribution alone does not authorize transformed physical source.
pub(super) fn require_definition_source(
    crates: &CrateIndex,
    definition: &Definition,
) -> Result<(), String> {
    let Definition::Item {
        krate,
        module,
        index,
    } = definition
    else {
        return Err("source item knowledge is unsupported".into());
    };
    let owner = crates.crate_at(*krate);
    for length in 0..=module.len() {
        let node = owner
            .graph
            .modules
            .get(&module[..length])
            .ok_or("missing owner module")?;
        if !module_attributes(&node.attributes)
            || module.get(length).is_some_and(|child| {
                node.items.iter().any(|item| {
                    matches!(item, Item::Mod(declaration)
                if declaration.ident == child.as_str() && !module_attributes(&declaration.attrs))
                })
            })
        {
            return Err("conditional fact owner is unsupported".into());
        }
    }
    let item = owner
        .graph
        .modules
        .get(module)
        .and_then(|n| n.items.get(*index))
        .ok_or("missing source item")?;
    let (_, visibility) =
        super::item_shape::defined_name(item).ok_or("unsupported source item kind")?;
    if !matches!(visibility, syn::Visibility::Public(_))
        || !fact_attributes(super::item_shape::attributes(item))
    {
        return Err("source item is private or transformed".into());
    }
    Ok(())
}
fn fact_attributes(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().all(|a| {
        [
            "doc",
            "derive",
            "repr",
            "non_exhaustive",
            "must_use",
            "deprecated",
            "allow",
            "warn",
            "deny",
            "forbid",
        ]
        .iter()
        .any(|name| a.path().is_ident(name))
    })
}

pub(crate) fn export_knowledge(
    root: &Path,
    manifest: &str,
    container: &str,
    name: &str,
    kind: SelectorKind,
) -> Knowledge {
    match inspect(root, manifest, container, name, kind) {
        Ok(knowledge) => knowledge,
        Err(reason) => Knowledge::Unsupported(reason),
    }
}

fn inspect(
    root: &Path,
    manifest: &str,
    container: &str,
    name: &str,
    kind: SelectorKind,
) -> Result<Knowledge, String> {
    if container != "facade" && !container.starts_with("facade::") {
        return Err("absence must name an audience facade container".into());
    }
    // Associated methods can be in another module, trait impl or opaque macro.
    // This source resolver does not claim coherence/impl coverage.
    if matches!(kind, SelectorKind::AssociatedMember) {
        return Err("associated-member coverage is not established".into());
    }
    if name.starts_with("r#") || syn::parse_str::<syn::Ident>(name).is_err() {
        return Err("invalid terminal selector".into());
    }
    let crates = CrateIndex::new(root);
    let resolver = Resolver::new(&crates);
    let id = crates.load(root.join(manifest).parent().ok_or("missing crate root")?)?;
    let path = container.split("::").map(str::to_owned).collect::<Vec<_>>();
    require_fact_route(&crates, &resolver, id, &path)?;
    require_closed_route(&crates, &resolver, id, &path)?;
    if matches!(kind, SelectorKind::EnumVariant) {
        let item = enum_definition(root, manifest, container, &crates)?;
        return Ok(if item.variants.iter().any(|v| v.ident == name) {
            Knowledge::Present
        } else {
            Knowledge::ProvenAbsent
        });
    }
    let definitions = resolver.resolve_path(id, &[], &path)?;
    let [Definition::Module { krate, path }] = definitions.as_slice() else {
        return Err("containing audience module does not resolve uniquely".into());
    };
    let owner = crates.crate_at(*krate);
    let node = owner
        .graph
        .modules
        .get(path)
        .ok_or("missing module source")?;
    let present = node.items.iter().any(|item| {
        if let Item::Mod(module) = item {
            return module.ident == name && matches!(&module.vis, syn::Visibility::Public(_));
        }
        super::item_shape::defined_name(item).is_some_and(|(defined, visibility)| {
            defined == name && matches!(visibility, syn::Visibility::Public(_))
        })
    });
    Ok(if present {
        Knowledge::Present
    } else {
        Knowledge::ProvenAbsent
    })
}

pub(super) fn require_fact_route(
    crates: &CrateIndex,
    resolver: &Resolver<'_>,
    id: super::CrateId,
    segments: &[String],
) -> Result<(), String> {
    for length in 0..=segments.len() {
        let prefix = resolver.resolve_path(id, &[], &segments[..length])?;
        for definition in prefix {
            if let Definition::Module { krate, path } = definition {
                let indexed = crates.crate_at(krate);
                let node = indexed
                    .graph
                    .modules
                    .get(&path)
                    .ok_or("missing route module")?;
                if !module_attributes(&node.attributes)
                    || node.items.iter().any(|item| {
                        let attrs = super::item_shape::attributes(item);
                        !super::item_shape::is_test_only(attrs)
                            && (conditional(attrs)
                                || matches!(item, Item::Mod(_)) && !module_attributes(attrs)
                                || matches!(item, Item::Use(_)) && !fact_attributes(attrs))
                    })
                {
                    return Err("conditional audience-route coverage is unsupported".into());
                }
            }
        }
    }
    Ok(())
}

// Closed negatives currently support local public module paths only. Public
// reexport/macro graphs remain Unsupported rather than borrowing attribution
// heuristics as evidence of complete visibility or expansion.
fn require_closed_route(
    crates: &CrateIndex,
    resolver: &Resolver<'_>,
    id: super::CrateId,
    segments: &[String],
) -> Result<(), String> {
    for length in 0..=segments.len() {
        for definition in resolver.resolve_path(id, &[], &segments[..length])? {
            let Definition::Module { krate, path } = definition else {
                continue;
            };
            let owner = crates.crate_at(krate);
            let node = owner
                .graph
                .modules
                .get(&path)
                .ok_or("missing closed route")?;
            for item in &node.items {
                if matches!(
                    item,
                    Item::Use(_)
                        | Item::Macro(_)
                        | Item::Verbatim(_)
                        | Item::ExternCrate(_)
                        | Item::ForeignMod(_)
                ) || super::item_shape::attributes(item)
                    .iter()
                    .any(|attr| !attr.path().is_ident("doc"))
                {
                    return Err("route contains unsupported export/attribute knowledge".into());
                }
                if let (Some(next), Item::Mod(module)) = (segments.get(length), item) {
                    if module.ident == next.as_str()
                        && !matches!(&module.vis, syn::Visibility::Public(_))
                    {
                        return Err("route traverses a non-public module".into());
                    }
                }
            }
        }
    }
    Ok(())
}
