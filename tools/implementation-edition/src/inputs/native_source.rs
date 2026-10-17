//! Foreign input boundaries, rather than ABI spelling. Native macro imports
//! are refused before renaming can hide them; expansion tokens cannot introduce
//! an unqualified native reference behind a local Rust macro either.

use proc_macro2::{Delimiter, Ident, TokenStream, TokenTree};
use syn::visit::Visit;

use crate::rust_identifier::semantic_name;

const NATIVE: &str = "native source inputs require input qualification";

pub(super) fn validate(file: &syn::File) -> Result<(), String> {
    let mut inputs = NativeInputs { error: None };
    inputs.visit_file(file);
    inputs.error.map_or(Ok(()), Err)
}

fn native_macro(name: &Ident) -> bool {
    matches!(
        semantic_name(name).as_str(),
        "asm" | "global_asm" | "naked_asm"
    )
}

struct NativeInputs {
    error: Option<String>,
}

impl NativeInputs {
    fn refuse(&mut self, reason: &str) {
        self.error.get_or_insert_with(|| reason.to_owned());
    }
}

impl<'ast> Visit<'ast> for NativeInputs {
    fn visit_item_foreign_mod(&mut self, _: &'ast syn::ItemForeignMod) {
        self.refuse(NATIVE);
    }

    fn visit_item_extern_crate(&mut self, item: &'ast syn::ItemExternCrate) {
        if !matches!(
            semantic_name(&item.ident).as_str(),
            "std" | "core" | "alloc"
        ) {
            self.refuse("implicit external crate resolution requires input qualification");
        }
    }

    fn visit_use_tree(&mut self, tree: &'ast syn::UseTree) {
        let imported = match tree {
            syn::UseTree::Name(name) => Some(&name.ident),
            syn::UseTree::Rename(rename) => Some(&rename.ident),
            _ => None,
        };
        // No expansion-origin proof exists for imports of these macro names.
        // This intentionally refuses unresolved imports, including re-exports,
        // rather than assuming an alias denotes an ordinary Rust macro.
        if imported.is_some_and(native_macro) {
            self.refuse(NATIVE);
        }
        syn::visit::visit_use_tree(self, tree);
    }

    fn visit_macro(&mut self, invocation: &'ast syn::Macro) {
        if invocation
            .path
            .segments
            .last()
            .is_some_and(|segment| native_macro(&segment.ident))
            || !expansion_inputs_qualified(invocation.tokens.clone())
        {
            self.refuse(NATIVE);
        }
    }
}

fn expansion_inputs_qualified(tokens: TokenStream) -> bool {
    let tokens: Vec<_> = tokens.into_iter().collect();
    for (index, token) in tokens.iter().enumerate() {
        if let TokenTree::Ident(name) = token {
            // Macro arguments/transcribers can carry a native path into a use
            // rename. Until expanded input observation exists, do not infer
            // harmlessness from an identifier's eventual invocation spelling.
            if native_macro(name) {
                return false;
            }
            if name == "extern" {
                let mut next = index + 1;
                if matches!(tokens.get(next), Some(TokenTree::Literal(_))) {
                    next += 1;
                }
                match tokens.get(next) {
                    Some(TokenTree::Ident(name)) if name == "fn" => {}
                    Some(TokenTree::Ident(name)) if name == "crate" => {
                        if !matches!(tokens.get(next + 1), Some(TokenTree::Ident(name)) if matches!(semantic_name(name).as_str(), "std" | "core" | "alloc"))
                        {
                            return false;
                        }
                    }
                    // Foreign blocks and unresolved macro-built extern forms
                    // both require qualification; Rust ABI definitions do not.
                    Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                        return false
                    }
                    _ => return false,
                }
            }
        }
        if let TokenTree::Group(group) = token
            && !expansion_inputs_qualified(group.stream())
        {
            return false;
        }
    }
    true
}
