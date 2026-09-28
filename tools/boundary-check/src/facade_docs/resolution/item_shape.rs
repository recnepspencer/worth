//! Read the name, visibility, kind, attributes, and source line of `syn` items.

use proc_macro2::TokenTree;
use syn::spanned::Spanned;
use syn::{Attribute, Item, ItemMacro, Meta, Visibility};

/// The name an item defines in the type or value namespace, with its visibility.
pub(crate) fn defined_name(item: &Item) -> Option<(String, &Visibility)> {
    let (ident, vis) = match item {
        Item::Const(item) => (&item.ident, &item.vis),
        Item::Enum(item) => (&item.ident, &item.vis),
        Item::Fn(item) => (&item.sig.ident, &item.vis),
        Item::Static(item) => (&item.ident, &item.vis),
        Item::Struct(item) => (&item.ident, &item.vis),
        Item::Trait(item) => (&item.ident, &item.vis),
        Item::TraitAlias(item) => (&item.ident, &item.vis),
        Item::Type(item) => (&item.ident, &item.vis),
        Item::Union(item) => (&item.ident, &item.vis),
        _ => return None,
    };
    Some((ident.to_string(), vis))
}

/// Name of a `macro_rules!` definition; `None` for invocations and other items.
pub(crate) fn macro_rules_name(item: &Item) -> Option<String> {
    match item {
        Item::Macro(definition) if definition.mac.path.is_ident("macro_rules") => {
            definition.ident.as_ref().map(ToString::to_string)
        }
        _ => None,
    }
}

/// An item-position macro invocation such as `generate_basis!(Name);`.
pub(crate) fn macro_invocation(item: &Item) -> Option<&ItemMacro> {
    match item {
        Item::Macro(invocation) if invocation.ident.is_none() => Some(invocation),
        _ => None,
    }
}

pub(crate) fn attributes(item: &Item) -> &[Attribute] {
    match item {
        Item::Const(item) => &item.attrs,
        Item::Enum(item) => &item.attrs,
        Item::ExternCrate(item) => &item.attrs,
        Item::Fn(item) => &item.attrs,
        Item::ForeignMod(item) => &item.attrs,
        Item::Impl(item) => &item.attrs,
        Item::Macro(item) => &item.attrs,
        Item::Mod(item) => &item.attrs,
        Item::Static(item) => &item.attrs,
        Item::Struct(item) => &item.attrs,
        Item::Trait(item) => &item.attrs,
        Item::TraitAlias(item) => &item.attrs,
        Item::Type(item) => &item.attrs,
        Item::Union(item) => &item.attrs,
        Item::Use(item) => &item.attrs,
        _ => &[],
    }
}

/// `#[cfg(test)]` (without `not`): absent from the compiled library.
pub(crate) fn is_test_only(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        let Meta::List(list) = &attribute.meta else {
            return false;
        };
        if !attribute.path().is_ident("cfg") {
            return false;
        }
        let idents = list
            .tokens
            .clone()
            .into_iter()
            .filter_map(|token| match token {
                TokenTree::Ident(ident) => Some(ident.to_string()),
                _ => None,
            })
            .collect::<Vec<_>>();
        idents.iter().any(|ident| ident == "test") && !idents.iter().any(|ident| ident == "not")
    })
}

pub(crate) fn is_macro_export(attributes: &[Attribute]) -> bool {
    attributes
        .iter()
        .any(|attribute| attribute.path().is_ident("macro_export"))
}

/// Human kind label for diagnostics.
pub(crate) fn kind_label(item: &Item) -> &'static str {
    match item {
        Item::Const(_) => "const",
        Item::Enum(_) => "enum",
        Item::Fn(_) => "fn",
        Item::Macro(_) => "macro",
        Item::Mod(_) => "module",
        Item::Static(_) => "static",
        Item::Struct(_) => "struct",
        Item::Trait(_) | Item::TraitAlias(_) => "trait",
        Item::Type(_) => "type alias",
        Item::Union(_) => "union",
        _ => "item",
    }
}

/// Line of the item's defining keyword, where a `///` comment belongs above.
pub(crate) fn defining_line(item: &Item) -> usize {
    let span = match item {
        Item::Const(item) => item.const_token.span,
        Item::Enum(item) => item.enum_token.span,
        Item::Fn(item) => item.sig.fn_token.span,
        Item::Macro(item) => item.mac.path.span(),
        Item::Mod(item) => item.mod_token.span,
        Item::Static(item) => item.static_token.span,
        Item::Struct(item) => item.struct_token.span,
        Item::Trait(item) => item.trait_token.span,
        Item::TraitAlias(item) => item.trait_token.span,
        Item::Type(item) => item.type_token.span,
        Item::Union(item) => item.union_token.span,
        other => other.span(),
    };
    span.start().line
}
