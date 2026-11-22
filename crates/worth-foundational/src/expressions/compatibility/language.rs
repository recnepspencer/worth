//! The language version a draft's vocabulary requires.
//!
//! A draft encodes the oldest language version that covers every reserved
//! callable and builtin type name it uses, so a V1-only draft keeps its V1
//! encoding and every draft still has exactly one encoding.

use crate::expressions::functions::callable_language;
use crate::expressions::language_version::LanguageVersion;
use crate::expressions::syntax::ast::{SyntaxNode, SyntaxTree, TypeArgument, TypeSyntax};

/// O(nodes + type syntax), with an explicit worklist for nested types.
pub(super) fn draft_language(tree: &SyntaxTree) -> LanguageVersion {
    let mut types: Vec<&TypeSyntax> = Vec::new();
    let mut language = LanguageVersion::V1;
    for node in tree.nodes() {
        match node {
            SyntaxNode::Call {
                function,
                type_arguments,
                ..
            } => {
                if let Some(name) = function.single() {
                    language = language.max(callable_language(name));
                }
                types.extend(type_arguments.iter().filter_map(type_argument));
            }
            SyntaxNode::None(ty) => types.push(ty),
            _ => {}
        }
    }
    while let Some(ty) = types.pop() {
        if let Some(name) = ty.name.single() {
            language = language.max(LanguageVersion::of_type_name(name));
        }
        types.extend(ty.arguments.iter().filter_map(type_argument));
    }
    language
}

fn type_argument(argument: &TypeArgument) -> Option<&TypeSyntax> {
    match argument {
        TypeArgument::Type(ty) => Some(ty),
        TypeArgument::Width(_) | TypeArgument::Product(_) => None,
    }
}
