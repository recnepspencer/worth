use super::{BTreeSet, Body};
use syn::visit::{self, Visit};

pub(in super::super) fn clock_helpers(
    sources: &[(&str, &str)],
    hang_guards: &[(&str, &str, &str)],
) -> BTreeSet<String> {
    let files: Vec<_> = sources
        .iter()
        .map(|(name, source)| {
            (
                *name,
                syn::parse_file(source).expect("timing source must parse"),
            )
        })
        .collect();
    let mut helpers = BTreeSet::new();
    let clocks: BTreeSet<_> = files
        .iter()
        .flat_map(|(_, syntax)| super::named_clocks(syntax))
        .collect();
    loop {
        let before = helpers.len();
        for (file, syntax) in &files {
            Functions {
                file,
                hang_guards,
                helpers: &mut helpers,
                clocks: &clocks,
            }
            .visit_file(syntax);
        }
        if helpers.len() == before {
            return helpers;
        }
    }
}

struct Functions<'a> {
    file: &'a str,
    hang_guards: &'a [(&'a str, &'a str, &'a str)],
    helpers: &'a mut BTreeSet<String>,
    clocks: &'a BTreeSet<String>,
}

impl Functions<'_> {
    fn inspect(
        &mut self,
        attributes: &[syn::Attribute],
        signature: &syn::Signature,
        block: &syn::Block,
    ) {
        let name = signature.ident.to_string();
        if attributes.iter().any(|a| a.path().is_ident("test"))
            || self.hang_guards.iter().any(|(file, function, reason)| {
                *file == self.file && *function == name && !reason.is_empty()
            })
        {
            return;
        }
        let mut body = Body {
            helpers: self.helpers.clone(),
            ignore_reports: true,
            clocks: self.clocks.clone(),
            direct_clocks: self.clocks.clone(),
            ..Body::default()
        };
        for argument in &signature.inputs {
            let syn::FnArg::Typed(argument) = argument else {
                continue;
            };
            let syn::Pat::Ident(pattern) = &*argument.pat else {
                continue;
            };
            if matches!(&*argument.ty, syn::Type::Path(p) if p.path.segments.iter().any(|s| super::is_clock_type(&s.ident.to_string())))
            {
                body.clocks.insert(pattern.ident.to_string());
                body.direct_clocks.insert(pattern.ident.to_string());
            }
        }
        body.visit_block(block);
        if body.found {
            self.helpers.insert(name);
        }
    }
}

impl<'ast> Visit<'ast> for Functions<'_> {
    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        self.inspect(&function.attrs, &function.sig, &function.block);
        visit::visit_item_fn(self, function);
    }
    fn visit_impl_item_fn(&mut self, function: &'ast syn::ImplItemFn) {
        self.inspect(&function.attrs, &function.sig, &function.block);
        visit::visit_impl_item_fn(self, function);
    }
}
