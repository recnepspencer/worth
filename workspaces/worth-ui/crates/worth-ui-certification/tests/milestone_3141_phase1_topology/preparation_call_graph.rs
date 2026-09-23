use std::collections::BTreeSet;

use syn::visit::Visit;

pub(super) fn validate(
    preparation_source: &str,
    profile_source: &str,
    native_profile_source: &str,
) -> Result<usize, String> {
    validate_function(
        preparation_source,
        "prepare",
        &["Ok"],
        &["PREPARATION_IDENTITIES.issue", "profile.validate"],
    )?;
    validate_function(
        preparation_source,
        "issue",
        &[],
        &[
            "current.checked_add",
            "self.next.fetch_update",
            "self.next.fetch_update().map_err",
        ],
    )?;
    validate_function(
        profile_source,
        "validate",
        &["Err", "Ok", "validate_environment"],
        &["self.window.title.is_empty", "self.window.title.len"],
    )?;
    validate_function(profile_source, "validate_environment", &["Err", "Ok"], &[])?;
    validate_function(native_profile_source, "as_str", &[], &[])?;
    Ok(0)
}

fn validate_function(
    source: &str,
    name: &str,
    allowed_calls: &[&str],
    allowed_methods: &[&str],
) -> Result<(), String> {
    let syntax =
        syn::parse_file(source).map_err(|error| format!("invalid Rust source: {error}"))?;
    let mut finder = FunctionFinder {
        wanted: name,
        bodies: Vec::new(),
    };
    finder.visit_file(&syntax);
    if finder.bodies.len() != 1 {
        return Err(format!("expected one exact {name} transition"));
    }
    let mut calls = CallCollector::default();
    calls.visit_block(finder.bodies[0]);
    let allowed_calls = allowed_calls.iter().copied().collect::<BTreeSet<_>>();
    let allowed_methods = allowed_methods.iter().copied().collect::<BTreeSet<_>>();
    for call in &calls.calls {
        if !allowed_calls.contains(call.as_str()) {
            return Err(format!("{name} calls unqualified effect surface {call}"));
        }
    }
    for method in &calls.methods {
        if !allowed_methods.contains(method.as_str()) {
            return Err(format!("{name} calls unqualified receiver method {method}"));
        }
    }
    Ok(())
}

struct FunctionFinder<'a, 'ast> {
    wanted: &'a str,
    bodies: Vec<&'ast syn::Block>,
}

impl<'a, 'ast> Visit<'ast> for FunctionFinder<'a, 'ast> {
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        if item.sig.ident == self.wanted {
            self.bodies.push(&item.block);
        }
        syn::visit::visit_item_fn(self, item);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        if item.sig.ident == self.wanted {
            self.bodies.push(&item.block);
        }
        syn::visit::visit_impl_item_fn(self, item);
    }
}

#[derive(Default)]
struct CallCollector {
    calls: BTreeSet<String>,
    methods: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for CallCollector {
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = call.func.as_ref() {
            self.calls.insert(path_identity(&path.path));
        } else {
            self.calls.insert("dynamic-call".to_owned());
        }
        syn::visit::visit_expr_call(self, call);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        self.methods.insert(format!(
            "{}.{}",
            receiver_identity(&call.receiver),
            call.method
        ));
        syn::visit::visit_expr_method_call(self, call);
    }
}

fn receiver_identity(expression: &syn::Expr) -> String {
    match expression {
        syn::Expr::Path(path) => path_identity(&path.path),
        syn::Expr::Field(field) => format!(
            "{}.{}",
            receiver_identity(&field.base),
            match &field.member {
                syn::Member::Named(name) => name.to_string(),
                syn::Member::Unnamed(index) => index.index.to_string(),
            }
        ),
        syn::Expr::MethodCall(call) => {
            format!("{}.{}()", receiver_identity(&call.receiver), call.method)
        }
        syn::Expr::Paren(paren) => receiver_identity(&paren.expr),
        _ => "dynamic-receiver".to_owned(),
    }
}

fn path_identity(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

#[test]
fn effectful_direct_and_indirect_call_mutants_are_rejected() {
    assert_effectful_mutants_rejected();
}

pub(super) fn assert_effectful_mutants_rejected() {
    let direct = "fn prepare() { std::fs::write(\"effect\", b\"x\"); }";
    let indirect = "fn prepare() { perform_external_effect(); }";
    for mutant in [direct, indirect] {
        assert!(validate_function(mutant, "prepare", &["Ok"], &["issue", "validate"]).is_err());
    }
    let callee_mutant = "fn issue() { std::fs::write(\"effect\", b\"x\"); }";
    assert!(validate_function(
        callee_mutant,
        "issue",
        &[],
        &["checked_add", "fetch_update", "map_err"]
    )
    .is_err());
    let forged_receiver = "fn prepare() { effect.issue(); }";
    assert!(validate_function(
        forged_receiver,
        "prepare",
        &["Ok"],
        &["PREPARATION_IDENTITIES.issue", "profile.validate"]
    )
    .is_err());
}
