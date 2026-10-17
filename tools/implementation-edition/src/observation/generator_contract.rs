//! Closed first generator grammar. Accepting arbitrary build-script Rust would
//! turn Cargo's rerun hints into an unjustified input-closure assertion.

use std::{collections::BTreeSet, fs, path::PathBuf};

use quote::ToTokens;
use syn::{visit::Visit, Expr};

use crate::{compiler_context, inputs::Discovered};

pub(super) fn validate(
    arguments: &[String],
    discovered: &Discovered,
) -> Result<Vec<PathBuf>, String> {
    if compiler_context::flag(arguments, "--crate-name") != Some("build_script_build") {
        return Ok(Vec::new());
    }
    if arguments
        .iter()
        .any(|argument| argument == "--extern" || argument.starts_with("--extern="))
    {
        return Err("generator build dependencies require input qualification".into());
    }
    let source = discovered
        .files
        .iter()
        .find(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .ok_or("generator source missing")?;
    if discovered
        .files
        .iter()
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .count()
        != 1
    {
        return Err("multi-source generator requires input qualification".into());
    }
    let parsed = syn::parse_file(&fs::read_to_string(source).map_err(|error| error.to_string())?)
        .map_err(|error| error.to_string())?;
    if parsed.items.len() != 1
        || !matches!(&parsed.items[0], syn::Item::Fn(function) if function.sig.ident == "main" && function.sig.inputs.is_empty() && function.attrs.is_empty())
    {
        return Err("generator requires the closed single-main input grammar".into());
    }
    let mut contract = GeneratorContract::default();
    contract.visit_file(&parsed);
    if let Some(error) = contract.error {
        return Err(error);
    }
    let mut reads = Vec::new();
    let root = std::env::var_os("CARGO_MANIFEST_DIR").ok_or("generator package root missing")?;
    for input in contract.reads {
        if !contract.declared.contains(&input) {
            return Err(format!("generator file read is not declared: {input}"));
        }
        reads.push(
            fs::canonicalize(std::path::Path::new(&root).join(input))
                .map_err(|error| error.to_string())?,
        );
    }
    Ok(reads)
}

#[derive(Default)]
struct GeneratorContract {
    declared: BTreeSet<String>,
    reads: BTreeSet<String>,
    error: Option<String>,
}

impl GeneratorContract {
    fn refuse(&mut self, detail: impl Into<String>) {
        self.error.get_or_insert_with(|| {
            format!(
                "generator input grammar requires qualification: {}",
                detail.into()
            )
        });
    }
}

impl<'ast> Visit<'ast> for GeneratorContract {
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        let Expr::Path(callee) = call.func.as_ref() else {
            self.refuse("indirect call");
            return;
        };
        let path = callee.path.to_token_stream().to_string().replace(' ', "");
        match path.as_str() {
            "std::fs::read_to_string" | "std::fs::read" => {
                if let Some(Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(path),
                    ..
                })) = call.args.first()
                {
                    self.reads.insert(path.value());
                } else {
                    self.refuse("file read must name one literal declared input");
                }
            }
            "std::env::var_os" => {
                if !matches!(call.args.first(), Some(Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(name), .. })) if name.value() == "OUT_DIR")
                {
                    self.refuse("ambient generator environment");
                }
            }
            "std::fs::write" | "std::path::PathBuf::from" => {}
            _ => self.refuse(format!("call {path}")),
        }
        syn::visit::visit_expr_call(self, call);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        if !matches!(
            call.method.to_string().as_str(),
            "unwrap" | "trim" | "parse" | "join"
        ) {
            self.refuse(format!("method {}", call.method));
        }
        syn::visit::visit_expr_method_call(self, call);
    }

    fn visit_macro(&mut self, invocation: &'ast syn::Macro) {
        if invocation.path.is_ident("println") {
            match syn::parse2::<syn::LitStr>(invocation.tokens.clone()) {
                Ok(text) if text.value().starts_with("cargo::rerun-if-changed=") => {
                    self.declared
                        .insert(text.value()["cargo::rerun-if-changed=".len()..].to_owned());
                }
                _ => self.refuse("only literal finite generator file directives are supported"),
            }
        } else if invocation.path.is_ident("format") {
            // Interpolation is limited to primitive locals in the closed
            // single-main grammar; macro argument expressions are refused.
            if syn::parse2::<syn::LitStr>(invocation.tokens.clone()).is_err() {
                self.refuse("format arguments must be primitive local interpolation");
            }
        } else {
            self.refuse(format!("macro {}", invocation.path.to_token_stream()));
        }
    }

    fn visit_item(&mut self, item: &'ast syn::Item) {
        if !matches!(item, syn::Item::Fn(function) if function.sig.ident == "main") {
            self.refuse("nested item or custom type");
        }
        syn::visit::visit_item(self, item);
    }
}
