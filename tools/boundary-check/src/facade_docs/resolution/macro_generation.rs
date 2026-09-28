//! Attribute a name to the `macro_rules!` invocation that generates it.
//!
//! An item-position invocation generates `name` when its tokens pass `name`
//! to a template that declares an item through a metavariable (`struct $n`),
//! or when the template itself spells `keyword name`. Declared items shadow
//! this attribution. The template is searched in the invoking module's
//! ancestry, then the whole crate, then exported macros of path dependencies.

use super::item_shape::{attributes, is_test_only, macro_invocation, macro_rules_name};
use super::{CrateId, Definition, MacroTemplate, Resolver};
use proc_macro2::{TokenStream, TokenTree};
use syn::Item;

const ITEM_KEYWORDS: [&str; 9] = [
    "struct", "enum", "union", "trait", "type", "fn", "const", "static", "mod",
];

impl Resolver<'_> {
    /// Items named `name` that macro invocations in `krate::module` generate.
    pub(super) fn generated(
        &self,
        krate: CrateId,
        module: &[String],
        name: &str,
    ) -> Result<Vec<Definition>, String> {
        let indexed = self.crates.crate_at(krate);
        let Some(node) = indexed.graph.modules.get(module) else {
            return Ok(Vec::new());
        };
        let mut found = Vec::new();
        for (index, item) in node.items.iter().enumerate() {
            let Some(invocation) = macro_invocation(item) else {
                continue;
            };
            let Some(macro_name) = invocation.mac.path.segments.last() else {
                continue;
            };
            let macro_name = macro_name.ident.to_string();
            if macro_name == "macro_rules" || is_test_only(&invocation.attrs) {
                continue;
            }
            let passes_name = mentions_ident(&invocation.mac.tokens, name);
            let Some(template) = self.template(krate, module, &macro_name, passes_name)? else {
                continue;
            };
            let body = self.template_tokens(&template);
            let generates = (passes_name && emits_item(&body)) || defines(&body, name);
            if generates {
                found.push(Definition::Generated {
                    krate,
                    module: module.to_vec(),
                    invocation: index,
                    template,
                });
            }
        }
        Ok(found)
    }

    /// The template tokens of a `macro_rules!` definition.
    pub(crate) fn template_tokens(&self, template: &MacroTemplate) -> TokenStream {
        let indexed = self.crates.crate_at(template.krate);
        match &indexed.graph.modules[&template.module].items[template.index] {
            Item::Macro(definition) => definition.mac.tokens.clone(),
            _ => TokenStream::new(),
        }
    }

    fn template(
        &self,
        krate: CrateId,
        module: &[String],
        macro_name: &str,
        search_dependencies: bool,
    ) -> Result<Option<MacroTemplate>, String> {
        let indexed = self.crates.crate_at(krate);
        let ancestry = (0..=module.len()).rev().map(|depth| &module[..depth]);
        for ancestor in ancestry {
            if let Some(index) =
                definition_index(&indexed.graph.modules[ancestor].items, macro_name)
            {
                return Ok(Some(template_at(krate, ancestor, index)));
            }
        }
        for (path, node) in &indexed.graph.modules {
            if let Some(index) = definition_index(&node.items, macro_name) {
                return Ok(Some(template_at(krate, path, index)));
            }
        }
        if !search_dependencies {
            return Ok(None);
        }
        for dependency in self.crates.loaded_dependencies(krate)? {
            let indexed = self.crates.crate_at(dependency);
            let exported = indexed.exported_macros.iter().find(|(path, index)| {
                macro_rules_name(&indexed.graph.modules[path].items[*index]).as_deref()
                    == Some(macro_name)
            });
            if let Some((path, index)) = exported {
                return Ok(Some(template_at(dependency, path, *index)));
            }
        }
        Ok(None)
    }
}

fn definition_index(items: &[Item], macro_name: &str) -> Option<usize> {
    items.iter().position(|item| {
        macro_rules_name(item).as_deref() == Some(macro_name) && !is_test_only(attributes(item))
    })
}

fn template_at(krate: CrateId, module: &[String], index: usize) -> MacroTemplate {
    MacroTemplate {
        krate,
        module: module.to_vec(),
        index,
    }
}

/// Idents of a token stream in order, descending into groups. A macro
/// metavariable keeps its `$` so it never matches a literal name.
fn idents(tokens: &TokenStream) -> Vec<String> {
    let mut found = Vec::new();
    let mut after_dollar = false;
    for token in tokens.clone() {
        match token {
            TokenTree::Ident(ident) if after_dollar => found.push(format!("${ident}")),
            TokenTree::Ident(ident) => found.push(ident.to_string()),
            TokenTree::Group(group) => found.extend(idents(&group.stream())),
            TokenTree::Punct(punct) if punct.as_char() == '$' => {
                after_dollar = true;
                continue;
            }
            TokenTree::Punct(_) | TokenTree::Literal(_) => {}
        }
        after_dollar = false;
    }
    found
}

fn mentions_ident(tokens: &TokenStream, name: &str) -> bool {
    idents(tokens).iter().any(|ident| ident == name)
}

/// Whether the template declares an item named by a metavariable, such as
/// `struct $name`.
fn emits_item(template: &TokenStream) -> bool {
    idents(template)
        .windows(2)
        .any(|pair| ITEM_KEYWORDS.contains(&pair[0].as_str()) && pair[1].starts_with('$'))
}

fn defines(template: &TokenStream, name: &str) -> bool {
    idents(template)
        .windows(2)
        .any(|pair| ITEM_KEYWORDS.contains(&pair[0].as_str()) && pair[1] == name)
}
