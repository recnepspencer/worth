//! A sealed test door may implement another door; neighboring production code may not.
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{ImplItem, Item};

pub(super) fn test_only(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("cfg")
            && attr
                .parse_args::<syn::Path>()
                .is_ok_and(|path| path.is_ident("test"))
    })
}

pub(super) fn ordinary_tokens(source: &str) -> Result<TokenStream, String> {
    let mut file = syn::parse_file(source).map_err(|error| error.to_string())?;
    ordinary_items(&mut file.items);
    Ok(file.into_token_stream())
}

fn ordinary_items(items: &mut Vec<Item>) {
    items.retain_mut(|item| match item {
        Item::Fn(function) => {
            !(test_only(&function.attrs)
                || exported(&function.vis)
                    && super::public_items::test_name(&function.sig.ident.to_string()))
        }
        Item::Struct(item) => {
            !test_only(&item.attrs) && !super::public_items::test_name(&item.ident.to_string())
        }
        Item::Enum(item) => {
            !test_only(&item.attrs) && !super::public_items::test_name(&item.ident.to_string())
        }
        Item::Type(item) => {
            !test_only(&item.attrs) && !super::public_items::test_name(&item.ident.to_string())
        }
        Item::Mod(module) => {
            if test_only(&module.attrs) || super::public_items::test_name(&module.ident.to_string())
            {
                return false;
            }
            if let Some((_, items)) = &mut module.content {
                ordinary_items(items);
            }
            true
        }
        Item::Impl(implementation) => {
            if test_only(&implementation.attrs) {
                return false;
            }
            if let syn::Type::Path(path) = implementation.self_ty.as_ref() {
                if path.path.segments.last().is_some_and(|segment| {
                    super::public_items::test_name(&segment.ident.to_string())
                }) {
                    return false;
                }
            }
            implementation.items.retain(|item| match item {
                ImplItem::Fn(function) => {
                    !(test_only(&function.attrs)
                        || exported(&function.vis)
                            && super::public_items::test_name(&function.sig.ident.to_string()))
                }
                _ => true,
            });
            true
        }
        // Imports of a handle type support the receiver/door declarations in
        // this control source. Arming calls in ordinary functions stay visible.
        Item::Use(item) => {
            let mut names = Vec::new();
            use_names(&item.tree, &mut names);
            !names.iter().all(|name| name.ends_with("ForTest"))
        }
        _ => true,
    });
}

fn exported(visibility: &syn::Visibility) -> bool {
    !matches!(visibility, syn::Visibility::Inherited)
}

fn use_names(tree: &syn::UseTree, names: &mut Vec<String>) {
    match tree {
        syn::UseTree::Path(path) => use_names(&path.tree, names),
        syn::UseTree::Name(name) => names.push(name.ident.to_string()),
        syn::UseTree::Rename(rename) => names.push(rename.ident.to_string()),
        syn::UseTree::Group(group) => {
            for item in &group.items {
                use_names(item, names);
            }
        }
        syn::UseTree::Glob(_) => names.push("*".into()),
    }
}
