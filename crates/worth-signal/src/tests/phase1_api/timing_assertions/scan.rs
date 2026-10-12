use std::collections::BTreeSet;
use syn::parse::Parser;
use syn::visit::{self, Visit};
use syn::{BinOp, Expr, ItemFn, Pat};

pub(super) fn violations(source: &str) -> Vec<String> {
    let file = syn::parse_file(source).expect("timing guard source must parse");
    struct PredicateFunctions(BTreeSet<String>);
    impl<'ast> Visit<'ast> for PredicateFunctions {
        fn visit_item_fn(&mut self, function: &'ast ItemFn) {
            if matches!(&function.sig.output, syn::ReturnType::Type(_, t)
                if matches!(&**t, syn::Type::Path(p) if p.path.is_ident("bool")))
            {
                let mut body = Body {
                    predicates: self.0.clone(),
                    ..Body::default()
                };
                body.visit_block(&function.block);
                let expression = Expr::Block(syn::ExprBlock {
                    attrs: Vec::new(),
                    label: None,
                    block: (*function.block).clone(),
                });
                if compares_clock(&expression, &body.clocks, &body.predicates) {
                    self.0.insert(function.sig.ident.to_string());
                }
            }
            visit::visit_item_fn(self, function);
        }
    }
    let mut functions = PredicateFunctions(BTreeSet::new());
    loop {
        let before = functions.0.len();
        functions.visit_file(&file);
        if functions.0.len() == before {
            break;
        }
    }
    struct Tests {
        found: Vec<String>,
        functions: BTreeSet<String>,
    }
    impl<'ast> Visit<'ast> for Tests {
        fn visit_item_fn(&mut self, function: &'ast ItemFn) {
            if function.attrs.iter().any(|a| a.path().is_ident("test"))
                && !function.attrs.iter().any(|a| a.path().is_ident("ignore"))
            {
                let mut body = Body {
                    predicates: self.functions.clone(),
                    ..Body::default()
                };
                body.visit_block(&function.block);
                if body.found {
                    self.found.push(function.sig.ident.to_string());
                }
            }
            visit::visit_item_fn(self, function);
        }
    }
    let mut tests = Tests {
        found: Vec::new(),
        functions: functions.0,
    };
    tests.visit_file(&file);
    tests.found
}

#[derive(Default)]
struct Body {
    clocks: BTreeSet<String>,
    direct_clocks: BTreeSet<String>,
    predicates: BTreeSet<String>,
    found: bool,
}

impl<'ast> Visit<'ast> for Body {
    fn visit_expr_binary(&mut self, binary: &'ast syn::ExprBinary) {
        if is_comparison(&binary.op) {
            self.found |= clock_operand(&binary.left, &self.direct_clocks, false)
                || clock_operand(&binary.right, &self.direct_clocks, false);
        }
        visit::visit_expr_binary(self, binary);
    }
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        self.found |= method_compares_clock(call, &self.direct_clocks, false);
        visit::visit_expr_method_call(self, call);
    }
    fn visit_local(&mut self, local: &'ast syn::Local) {
        let pattern = match &local.pat {
            Pat::Type(p) => &*p.pat,
            p => p,
        };
        if let (Pat::Ident(name), Some(init)) = (pattern, &local.init) {
            if clock_operand(&init.expr, &self.direct_clocks, false)
                || matches!(&local.pat, Pat::Type(p) if matches!(&*p.ty, syn::Type::Path(t) if t.path.segments.last().is_some_and(|s| s.ident == "Duration")))
            {
                self.direct_clocks.insert(name.ident.to_string());
            }
            if is_clock(&init.expr, &self.clocks)
                || matches!(&local.pat, Pat::Type(p) if matches!(&*p.ty, syn::Type::Path(t) if t.path.segments.last().is_some_and(|s| s.ident == "Duration")))
            {
                self.clocks.insert(name.ident.to_string());
            }
            if compares_clock(&init.expr, &self.clocks, &self.predicates) {
                self.predicates.insert(name.ident.to_string());
            }
        }
        visit::visit_local(self, local);
    }
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        let name = mac.path.segments.last().unwrap().ident.to_string();
        if matches!(
            name.as_str(),
            "assert"
                | "assert_eq"
                | "assert_ne"
                | "debug_assert"
                | "debug_assert_eq"
                | "debug_assert_ne"
        ) {
            let arguments = syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated
                .parse2(mac.tokens.clone())
                .expect("assertion arguments must parse");
            if name.ends_with("_eq") || name.ends_with("_ne") {
                self.found |= arguments.iter().take(2).any(|e| is_clock(e, &self.clocks));
            } else if let Some(condition) = arguments.first() {
                self.found |= compares_clock(condition, &self.clocks, &self.predicates);
            }
        }
    }
}

fn is_comparison(operation: &BinOp) -> bool {
    matches!(
        operation,
        BinOp::Eq(_) | BinOp::Ne(_) | BinOp::Lt(_) | BinOp::Le(_) | BinOp::Gt(_) | BinOp::Ge(_)
    )
}

fn method_compares_clock(
    call: &syn::ExprMethodCall,
    clocks: &BTreeSet<String>,
    reports: bool,
) -> bool {
    matches!(
        call.method.to_string().as_str(),
        "eq" | "ne" | "lt" | "le" | "gt" | "ge" | "cmp" | "partial_cmp"
    ) && (clock_operand(&call.receiver, clocks, reports)
        || call
            .args
            .iter()
            .any(|arg| clock_operand(arg, clocks, reports)))
}

fn is_clock(expression: &Expr, clocks: &BTreeSet<String>) -> bool {
    clock_operand(expression, clocks, true)
}

fn clock_operand(expression: &Expr, clocks: &BTreeSet<String>, reports: bool) -> bool {
    struct Clock<'a> {
        clocks: &'a BTreeSet<String>,
        reports: bool,
        found: bool,
    }
    impl<'ast> Visit<'ast> for Clock<'_> {
        fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
            self.found |= path.path.segments.iter().any(|s| s.ident == "Duration")
                || path
                    .path
                    .get_ident()
                    .is_some_and(|i| self.clocks.contains(&i.to_string()));
            visit::visit_expr_path(self, path);
        }
        // These report fields are the owner's integer projections of Duration.
        fn visit_expr_field(&mut self, field: &'ast syn::ExprField) {
            self.found |= self.reports
                && matches!(&field.member, syn::Member::Named(name)
                if matches!(name.to_string().as_str(), "warm_median_micros" | "warm_p95_micros"
                    | "completed_batches_per_second" | "completed_batches_per_second_milli"));
            visit::visit_expr_field(self, field);
        }
        fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
            self.found |= call.method == "elapsed";
            visit::visit_expr_method_call(self, call);
        }
    }
    let mut clock = Clock {
        clocks,
        reports,
        found: false,
    };
    clock.visit_expr(expression);
    clock.found
}

fn compares_clock(
    expression: &Expr,
    clocks: &BTreeSet<String>,
    predicates: &BTreeSet<String>,
) -> bool {
    struct Comparisons<'a> {
        clocks: &'a BTreeSet<String>,
        predicates: &'a BTreeSet<String>,
        found: bool,
    }
    impl<'ast> Visit<'ast> for Comparisons<'_> {
        fn visit_expr_binary(&mut self, binary: &'ast syn::ExprBinary) {
            if is_comparison(&binary.op) {
                self.found |=
                    is_clock(&binary.left, self.clocks) || is_clock(&binary.right, self.clocks);
            }
            visit::visit_expr_binary(self, binary);
        }
        fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
            self.found |= method_compares_clock(call, self.clocks, true);
            visit::visit_expr_method_call(self, call);
        }
        fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
            self.found |= path
                .path
                .get_ident()
                .is_some_and(|i| self.predicates.contains(&i.to_string()));
            visit::visit_expr_path(self, path);
        }
    }
    let mut comparison = Comparisons {
        clocks,
        predicates,
        found: false,
    };
    comparison.visit_expr(expression);
    comparison.found
}
