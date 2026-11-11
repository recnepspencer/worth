//! Only source-preserving attributes may participate in a source type proof.
use quote::ToTokens;
use syn::{Attribute, Item, UseTree};

pub(super) fn check(attributes: &[Attribute], items: &[Item]) -> Result<(), String> {
    check_with_helpers(attributes, items, &[])
}

fn check_with_helpers(
    attributes: &[Attribute],
    items: &[Item],
    inherited: &[String],
) -> Result<(), String> {
    let mut helpers = inherited.to_vec();
    helpers.extend(helper_names(attributes));
    for attribute in attributes {
        let path = attribute
            .path()
            .to_token_stream()
            .to_string()
            .replace(' ', "");
        match path.as_str() {
            "derive" => {
                let derives = attribute
                    .parse_args_with(
                        syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                    )
                    .map_err(|error| error.to_string())?;
                for derive in derives {
                    let full_name = derive.to_token_stream().to_string().replace(' ', "");
                    let name = full_name.trim_start_matches("::");
                    let namespace = derive
                        .segments
                        .first()
                        .ok_or("empty derive path")?
                        .ident
                        .to_string();
                    if derive.leading_colon.is_none() && derive.segments.len() > 1
                        && items.iter().any(|item| matches!(item, Item::Mod(value) if value.ident == namespace)
                            || matches!(item, Item::Use(value) if imported(&value.tree, &namespace, false))) {
                        return Err(format!("shadowed derive namespace: {name}"));
                    }
                    let builtins = [
                        "Clone",
                        "Copy",
                        "Debug",
                        "Eq",
                        "PartialEq",
                        "Ord",
                        "PartialOrd",
                        "Hash",
                        "Default",
                    ];
                    let known = [
                        "serde::Serialize",
                        "serde::Deserialize",
                        "worth_foundational::facade::WorthValue",
                        "worth_query_decl::facade::application_schema::WorthValue",
                    ];
                    if !(known.contains(&name) || (builtins.contains(&name) && !items.iter().any(|item| matches!(item, Item::Use(value) if imported(&value.tree, name, false))))) {
                        return Err(format!("unknown/shadowed derive may change namespace resolution: {name}"));
                    }
                }
            }
            "serde" | "worth_value" | "charged_bytes" => {
                if !helpers.contains(&path)
                    || items.iter().any(
                        |item| matches!(item, Item::Use(value) if imported(&value.tree, &path, true)),
                    )
                {
                    return Err(format!("unowned/shadowed helper attribute: {path}"));
                }
            }
            "cfg_attr" if conditional_lints(attribute) => (),
            "doc" | "cfg" | "path" | "allow" | "warn" | "deny" | "forbid" | "expect"
            | "deprecated" | "repr" | "must_use" | "non_exhaustive" | "test" | "ignore"
            | "should_panic" | "inline" | "cold" | "track_caller" | "rustfmt::skip"
            | "recursion_limit" | "macro_use" | "macro_export" => (),
            _ => {
                return Err(format!(
                    "unsupported transforming attribute in domain source: {path}"
                ))
            }
        }
    }
    Ok(())
}

fn conditional_lints(attribute: &Attribute) -> bool {
    let Ok(arguments) = attribute.parse_args_with(
        syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
    ) else {
        return false;
    };
    arguments.len() > 1
        && arguments.iter().skip(1).all(|meta| {
            ["allow", "warn", "deny", "forbid", "expect"]
                .iter()
                .any(|name| meta.path().is_ident(name))
        })
}

fn imported(tree: &UseTree, name: &str, glob: bool) -> bool {
    match tree {
        UseTree::Name(value) => value.ident == name,
        UseTree::Rename(value) => value.rename == name,
        UseTree::Path(value) => imported(&value.tree, name, glob),
        UseTree::Group(group) => group.items.iter().any(|tree| imported(tree, name, glob)),
        UseTree::Glob(_) => glob,
    }
}

pub(super) fn module(items: &[Item]) -> Result<(), String> {
    for item in items {
        match item {
            Item::Struct(value) => {
                check(&value.attrs, items)?;
                for field in &value.fields {
                    check_with_helpers(&field.attrs, items, &helper_names(&value.attrs))?;
                }
            }
            Item::Enum(value) => {
                check(&value.attrs, items)?;
                for variant in &value.variants {
                    check_with_helpers(&variant.attrs, items, &helper_names(&value.attrs))?;
                    for field in &variant.fields {
                        check_with_helpers(&field.attrs, items, &helper_names(&value.attrs))?;
                    }
                }
            }
            Item::Type(value) => check(&value.attrs, items)?,
            Item::Use(value) => check(&value.attrs, items)?,
            Item::Mod(value) => check(&value.attrs, items)?,
            Item::Fn(value) => check(&value.attrs, items)?,
            Item::Impl(value) => {
                check(&value.attrs, items)?;
                for member in &value.items {
                    match member {
                        syn::ImplItem::Type(value) => check(&value.attrs, items)?,
                        syn::ImplItem::Const(value) => check(&value.attrs, items)?,
                        syn::ImplItem::Fn(value) => check(&value.attrs, items)?,
                        syn::ImplItem::Macro(value) => check(&value.attrs, items)?,
                        _ => {
                            return Err("unknown generated associated item in domain source".into())
                        }
                    }
                }
            }
            Item::Const(value) => check(&value.attrs, items)?,
            Item::Static(value) => check(&value.attrs, items)?,
            Item::Macro(value) => check(&value.attrs, items)?,
            Item::ExternCrate(value) => check(&value.attrs, items)?,
            Item::Trait(value) => check(&value.attrs, items)?,
            Item::Union(_) | Item::Verbatim(_) => {
                return Err("unsupported generated/union namespace in domain source".into())
            }
            _ => (),
        }
    }
    Ok(())
}

fn helper_names(attributes: &[Attribute]) -> Vec<String> {
    let mut names = Vec::new();
    for attribute in attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("derive"))
    {
        if let Ok(paths) = attribute.parse_args_with(
            syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
        ) {
            for path in paths {
                let name = path.to_token_stream().to_string().replace(' ', "");
                match name.trim_start_matches("::") {
                    "serde::Serialize" | "serde::Deserialize" => names.push("serde".into()),
                    "worth_foundational::facade::WorthValue"
                    | "worth_query_decl::facade::application_schema::WorthValue" => {
                        names.push("serde".into());
                        names.push("worth_value".into());
                    }
                    _ => (),
                }
            }
        }
    }
    names
}
