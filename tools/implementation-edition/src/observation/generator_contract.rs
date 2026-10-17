//! Source-derived finite generator effects. Neither package identity nor a
//! reviewed digest grants effects; every branch is checked before execution.
use crate::{compiler_context, inputs::Discovered, rust_identifier::semantic_name};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};
use syn::{visit::Visit, Expr};

mod effects;
mod environment;
pub(super) mod evaluation;
mod formatting;
mod methods;
mod numeric;
mod provenance;
mod syntax;
use syntax::Syntax;
#[cfg(test)]
mod tests;
mod values;

pub(super) struct Program {
    functions: BTreeMap<String, syn::ItemFn>,
    constants: BTreeMap<String, Expr>,
    imports: BTreeMap<String, String>,
    pub(super) reads: BTreeSet<String>,
    environment: BTreeSet<String>,
}

pub(super) fn validate(
    arguments: &[String],
    discovered: &Discovered,
) -> Result<Option<PathBuf>, String> {
    if compiler_context::flag(arguments, "--crate-name") != Some("build_script_build") {
        return Ok(None);
    }
    if arguments
        .iter()
        .any(|a| a == "--extern" || a.starts_with("--extern="))
    {
        return Err("generator build dependencies require input qualification".into());
    }
    let sources: Vec<_> = discovered
        .files
        .iter()
        .filter(|p| p.extension().is_some_and(|e| e == "rs"))
        .collect();
    if sources.len() != 1 {
        return Err("generator requires one qualified Rust source".into());
    }
    Program::parse(&fs::read_to_string(sources[0]).map_err(|e| e.to_string())?)?;
    Ok(Some(sources[0].clone()))
}

impl Program {
    pub(super) fn parse(source: &str) -> Result<Self, String> {
        let file = syn::parse_file(source).map_err(|e| e.to_string())?;
        if !file.attrs.is_empty() {
            return Err("generator file attributes require qualification".into());
        }
        let mut program = Self {
            functions: BTreeMap::new(),
            constants: BTreeMap::new(),
            imports: BTreeMap::new(),
            reads: BTreeSet::new(),
            environment: BTreeSet::new(),
        };
        for item in file.items {
            match item {
                syn::Item::Fn(function)
                    if function.attrs.is_empty()
                        && function.sig.inputs.is_empty()
                        && function.sig.generics.params.is_empty()
                        && function.sig.unsafety.is_none()
                        && function.sig.abi.is_none()
                        && function.sig.asyncness.is_none() =>
                {
                    if program
                        .functions
                        .insert(semantic_name(&function.sig.ident), function)
                        .is_some()
                    {
                        return Err("duplicate generator function".into());
                    }
                }
                syn::Item::Const(item)
                    if item.attrs.is_empty() && matches!(*item.expr, Expr::Lit(_)) =>
                {
                    if program
                        .constants
                        .insert(semantic_name(&item.ident), *item.expr)
                        .is_some()
                    {
                        return Err("duplicate generator constant".into());
                    }
                }
                syn::Item::Use(item) if item.attrs.is_empty() => {
                    program.import(&item.tree, String::new())?
                }
                _ => return Err("generator item requires input qualification".into()),
            }
        }
        if !program.functions.contains_key("main") {
            return Err("generator main missing".into());
        }
        for function in program.functions.values_mut() {
            numeric::resolve(function)?;
        }
        let mut syntax = Syntax {
            program: &program,
            reads: BTreeSet::new(),
            environment: BTreeSet::new(),
            bindings: provenance::Bindings::default(),
            error: None,
        };
        for function in program.functions.values() {
            syntax.bindings = provenance::Bindings::of(&function.block);
            syntax.visit_block(&function.block);
        }
        if let Some(error) = syntax.error {
            return Err(error);
        }
        let reads = syntax.reads;
        let environment = syntax.environment;
        program.reads = reads;
        program.environment = environment;
        Ok(program)
    }

    fn import(&mut self, tree: &syn::UseTree, prefix: String) -> Result<(), String> {
        match tree {
            syn::UseTree::Path(path) => self.import(
                &path.tree,
                format!("{prefix}{}::", semantic_name(&path.ident)),
            ),
            syn::UseTree::Group(group) => {
                for item in &group.items {
                    self.import(item, prefix.clone())?;
                }
                Ok(())
            }
            syn::UseTree::Name(name) => self.import_name(
                prefix,
                semantic_name(&name.ident),
                semantic_name(&name.ident),
            ),
            syn::UseTree::Rename(name) => self.import_name(
                prefix,
                semantic_name(&name.ident),
                semantic_name(&name.rename),
            ),
            _ => Err("generator glob import requires qualification".into()),
        }
    }
    fn import_name(&mut self, prefix: String, name: String, alias: String) -> Result<(), String> {
        let full = prefix + &name;
        if !matches!(
            full.as_str(),
            "std::env" | "std::fs" | "std::path::PathBuf" | "std::process::Command" | "std::str"
        ) || self.imports.insert(alias, full).is_some()
        {
            return Err("generator import requires qualification".into());
        }
        Ok(())
    }
    fn path(&self, path: &syn::Path) -> String {
        let mut parts: Vec<_> = path
            .segments
            .iter()
            .map(|s| semantic_name(&s.ident))
            .collect();
        if let Some(first) = parts.first_mut()
            && let Some(import) = self.imports.get(first)
        {
            *first = import.clone();
        }
        parts.join("::")
    }
}
