//! AST discovery for `#[derive(WorthBinaryValue)]` aliases and `#[cfg_attr(...)]` guards,
//! resolving derive provenance across both inline and out-of-line module parent chains.
use super::macro_origin::BinaryDeriveTarget;
use super::modules::Sources;
use quote::ToTokens;
use std::collections::BTreeSet;
use syn::{Item, Meta, UseTree};

pub(super) fn collect_binary_aliases(
    items: &[Item],
    parent_aliases: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut aliases = BTreeSet::new();
    for item in items {
        if let Item::Use(use_item) = item {
            let mut prefix = Vec::new();
            collect_use_binary_aliases(&use_item.tree, &mut prefix, parent_aliases, &mut aliases);
        }
    }
    aliases
}

fn collect_use_binary_aliases(
    tree: &UseTree,
    prefix: &mut Vec<String>,
    parent_aliases: &BTreeSet<String>,
    out: &mut BTreeSet<String>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_use_binary_aliases(&path.tree, prefix, parent_aliases, out);
            prefix.pop();
        }
        UseTree::Rename(rename) => {
            let src = rename.ident.to_string();
            let from_parent = prefix.first().is_some_and(|p| p == "super");
            if src == "WorthBinaryValue" || (from_parent && parent_aliases.contains(&src)) {
                out.insert(rename.rename.to_string());
            }
        }
        UseTree::Name(name) => {
            let ident = name.ident.to_string();
            if prefix.first().is_some_and(|p| p == "super") && parent_aliases.contains(&ident) {
                out.insert(ident);
            }
        }
        UseTree::Group(group) => {
            for item in &group.items {
                collect_use_binary_aliases(item, prefix, parent_aliases, out);
            }
        }
        UseTree::Glob(_) => {}
    }
}

pub(super) fn inspect_worth_binary_derive(
    attrs: &[syn::Attribute],
    aliases: &BTreeSet<String>,
    sources: &Sources<'_>,
    source: &str,
    modules: &[String],
) -> Result<Option<syn::Path>, String> {
    let mut derives: Vec<(syn::Path, bool)> = Vec::new();
    let mut has_worth_binary = false;
    let mut cfg_worth_binary = false;
    let mut has_worth_query_identity = false;
    for attr in attrs {
        collect_meta_derives(
            &attr.meta,
            false,
            &mut derives,
            &mut has_worth_binary,
            &mut cfg_worth_binary,
            &mut has_worth_query_identity,
        )?;
    }
    if cfg_worth_binary {
        return Err(
            "conditional #[cfg_attr(..., worth_binary(...))] domain binary value claim is unsupported"
                .into(),
        );
    }
    let mut matched: Option<syn::Path> = None;
    let mut contexts_cache: Option<Result<Vec<Vec<String>>, String>> = None;
    for (path, from_cfg) in &derives {
        let direct = path
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "WorthBinaryValue");
        let local_alias = path.leading_colon.is_none()
            && path.segments.len() == 1
            && aliases.contains(&path.segments[0].ident.to_string());
        let mut is_binary_claim = direct || local_alias;
        if !is_binary_claim {
            let contexts = contexts_cache.get_or_insert_with(|| {
                sources.find_sources(source).map(|ctxs| {
                    ctxs.into_iter()
                        .map(|mut ctx| {
                            ctx.extend(modules.iter().cloned());
                            ctx
                        })
                        .collect()
                })
            });
            match contexts {
                Ok(ctxs) => {
                    for ctx in ctxs {
                        match sources.resolve_binary_derive(ctx, path) {
                            BinaryDeriveTarget::Approved | BinaryDeriveTarget::ForgedBinary => {
                                is_binary_claim = true;
                                break;
                            }
                            BinaryDeriveTarget::Unresolvable(err)
                                if has_worth_binary || has_worth_query_identity =>
                            {
                                return Err(err);
                            }
                            BinaryDeriveTarget::Unresolvable(_) | BinaryDeriveTarget::Other => {}
                        }
                    }
                }
                Err(err) if has_worth_binary || has_worth_query_identity => {
                    return Err(err.clone());
                }
                Err(_) => {}
            }
        }
        if is_binary_claim {
            if *from_cfg {
                return Err(
                    "conditional #[cfg_attr(..., derive(...))] domain binary value claim is unsupported"
                        .into(),
                );
            }
            if matched.is_none() {
                matched = Some(path.clone());
            }
        }
    }
    if matched.is_some() {
        return Ok(matched);
    }
    if has_worth_binary {
        for (path, from_cfg) in derives {
            if !is_non_binary_known_derive(&path) {
                if from_cfg {
                    return Err(
                        "conditional #[cfg_attr(..., derive(...))] domain binary value claim is unsupported"
                            .into(),
                    );
                }
                return Ok(Some(path));
            }
        }
        return Err(
            "#[worth_binary(...)] requires an explicit unconditional WorthBinaryValue derive"
                .into(),
        );
    }
    Ok(None)
}

fn collect_meta_derives(
    meta: &Meta,
    in_cfg_attr: bool,
    derives: &mut Vec<(syn::Path, bool)>,
    has_worth_binary: &mut bool,
    cfg_worth_binary: &mut bool,
    has_worth_query_identity: &mut bool,
) -> Result<(), String> {
    let Meta::List(list) = meta else {
        return Ok(());
    };
    if list.path.is_ident("worth_binary") {
        *has_worth_binary = true;
        if in_cfg_attr {
            *cfg_worth_binary = true;
        }
        return Ok(());
    }
    if list.path.is_ident("worth_query") && list.tokens.to_string().contains("identity") {
        *has_worth_query_identity = true;
        return Ok(());
    }
    if list.path.is_ident("derive") {
        let paths = list
            .parse_args_with(
                syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
            )
            .map_err(|error| format!("unparseable derive attribute: {error}"))?;
        for path in paths {
            derives.push((path, in_cfg_attr));
        }
        return Ok(());
    }
    if list.path.is_ident("cfg_attr") {
        let args = list
            .parse_args_with(syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated)
            .map_err(|error| format!("unparseable cfg_attr attribute: {error}"))?;
        for inner in args.iter().skip(1) {
            collect_meta_derives(
                inner,
                true,
                derives,
                has_worth_binary,
                cfg_worth_binary,
                has_worth_query_identity,
            )?;
        }
    }
    Ok(())
}

fn is_non_binary_known_derive(path: &syn::Path) -> bool {
    let text = path.to_token_stream().to_string().replace(' ', "");
    let name = text.trim_start_matches("::");
    matches!(
        name,
        "Clone"
            | "Copy"
            | "Debug"
            | "Eq"
            | "PartialEq"
            | "Ord"
            | "PartialOrd"
            | "Hash"
            | "Default"
            | "serde::Serialize"
            | "serde::Deserialize"
            | "Serialize"
            | "Deserialize"
            | "WorthValue"
            | "worth_foundational::facade::WorthValue"
            | "worth_query_decl::facade::application_schema::WorthValue"
            | "WorthQuery"
            | "WorthQuerySchema"
            | "WorthQueryRecord"
            | "WorthQueryProjection"
            | "WorthManagedIndex"
            | "WorthQueryAction"
            | "WorthQueryCorrection"
            | "WorthNameDiscriminator"
    )
}

pub(super) fn derived_binary_binding_name(item: &syn::ItemStruct) -> Result<String, String> {
    let mut explicit: Option<String> = None;
    for attr in &item.attrs {
        if !attr.path().is_ident("worth_binary") && !attr.path().is_ident("worth_query") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("binding") {
                let value = meta.value()?;
                let ident: syn::Ident = value.parse()?;
                if explicit.replace(ident.to_string()).is_some() {
                    return Err(meta.error("duplicate `binding` attribute on WorthBinaryValue"));
                }
            } else if meta.input.peek(syn::Token![=]) {
                let value = meta.value()?;
                let _: syn::Expr = value.parse()?;
            } else if meta.input.peek(syn::token::Paren) {
                let _content;
                syn::parenthesized!(_content in meta.input);
            }
            Ok(())
        })
        .map_err(|error| {
            format!(
                "invalid WorthBinaryValue attribute on `{}`: {error}",
                item.ident
            )
        })?;
    }
    Ok(explicit.unwrap_or_else(|| format!("{}Binding", item.ident)))
}
