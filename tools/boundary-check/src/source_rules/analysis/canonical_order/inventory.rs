//! Written ordering contracts and the TypeId-bearing values they order.
use std::collections::{BTreeMap, BTreeSet};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Item, Type, UseTree};

pub(super) struct Finding {
    pub source: String,
    pub item: String,
    pub line: usize,
    pub reason: &'static str,
}

pub(super) fn scan(files: &[(&str, &[Item])]) -> Vec<Finding> {
    let mut vocabulary = Vocabulary::default();
    vocabulary.type_ids.insert("TypeId".into());
    vocabulary
        .ordering
        .extend(["Ord".into(), "PartialOrd".into()]);
    vocabulary
        .containers
        .extend(["BTreeMap".into(), "BTreeSet".into()]);
    for (_, items) in files {
        for item in *items {
            vocabulary.visit_item(item);
        }
    }
    let mut type_ids = vocabulary.type_ids.clone();
    loop {
        let before = type_ids.len();
        for (name, fields) in &vocabulary.fields {
            if fields.iter().any(|field| type_ids.contains(field)) {
                type_ids.insert(name.clone());
            }
        }
        if before == type_ids.len() {
            break;
        }
    }
    let mut findings = Vec::new();
    for (source, items) in files {
        let mut local = Vocabulary::default();
        for item in *items {
            local.visit_item(item);
        }
        let mut scanner = OrderingSites {
            source,
            item: String::new(),
            type_ids: &type_ids,
            local_fields: &local.fields,
            vocabulary: &vocabulary,
            findings: &mut findings,
        };
        for item in *items {
            scanner.visit_item(item);
        }
    }
    findings
}

#[derive(Default)]
struct Vocabulary {
    type_ids: BTreeSet<String>,
    ordering: BTreeSet<String>,
    containers: BTreeSet<String>,
    fields: BTreeMap<String, BTreeSet<String>>,
}

impl<'ast> Visit<'ast> for Vocabulary {
    fn visit_use_tree(&mut self, tree: &'ast UseTree) {
        if let UseTree::Rename(rename) = tree {
            let from = rename.ident.to_string();
            let to = rename.rename.to_string();
            if self.type_ids.contains(&from) {
                self.type_ids.insert(to.clone());
            }
            if self.ordering.contains(&from) {
                self.ordering.insert(to.clone());
            }
            if self.containers.contains(&from) {
                self.containers.insert(to);
            }
        }
        visit::visit_use_tree(self, tree);
    }
    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        self.fields
            .entry(item.ident.to_string())
            .or_default()
            .extend(field_names(item.fields.iter().map(|f| &f.ty)));
        visit::visit_item_struct(self, item);
    }
    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        self.fields
            .entry(item.ident.to_string())
            .or_default()
            .extend(field_names(
                item.variants
                    .iter()
                    .flat_map(|v| v.fields.iter().map(|f| &f.ty)),
            ));
        visit::visit_item_enum(self, item);
    }
    fn visit_item_union(&mut self, item: &'ast syn::ItemUnion) {
        self.fields
            .entry(item.ident.to_string())
            .or_default()
            .extend(field_names(item.fields.named.iter().map(|f| &f.ty)));
        visit::visit_item_union(self, item);
    }
    fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
        self.fields
            .entry(item.ident.to_string())
            .or_default()
            .extend(field_names([item.ty.as_ref()]));
        visit::visit_item_type(self, item);
    }
    // The graph already contains each inline module's items separately.
    fn visit_item_mod(&mut self, _: &'ast syn::ItemMod) {}
}

fn field_names<'a>(types: impl IntoIterator<Item = &'a Type>) -> BTreeSet<String> {
    #[derive(Default)]
    struct Names(BTreeSet<String>);
    impl<'ast> Visit<'ast> for Names {
        fn visit_type_path(&mut self, path: &'ast syn::TypePath) {
            if let Some(segment) = path.path.segments.last() {
                self.0.insert(segment.ident.to_string());
            }
            visit::visit_type_path(self, path);
        }
    }
    let mut names = Names::default();
    for ty in types {
        names.visit_type(ty);
    }
    names.0
}

struct OrderingSites<'a> {
    source: &'a str,
    item: String,
    type_ids: &'a BTreeSet<String>,
    local_fields: &'a BTreeMap<String, BTreeSet<String>>,
    vocabulary: &'a Vocabulary,
    findings: &'a mut Vec<Finding>,
}

impl OrderingSites<'_> {
    fn record(&mut self, line: usize, reason: &'static str) {
        self.findings.push(Finding {
            source: self.source.into(),
            item: self.item.clone(),
            line,
            reason,
        });
    }
    fn derives<'a>(
        &mut self,
        name: &syn::Ident,
        attributes: &[syn::Attribute],
        fields: impl IntoIterator<Item = &'a Type>,
    ) {
        self.item = name.to_string();
        if !field_names(fields)
            .iter()
            .any(|field| self.type_ids.contains(field))
        {
            return;
        }
        for attribute in attributes.iter().filter(|a| a.path().is_ident("derive")) {
            let paths = attribute.parse_args_with(
                syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
            );
            if paths.is_ok_and(|paths| {
                paths.iter().any(|p| {
                    p.segments
                        .last()
                        .is_some_and(|s| self.vocabulary.ordering.contains(&s.ident.to_string()))
                })
            }) {
                self.record(
                    name.span().start().line,
                    "ordering derive on a TypeId-bearing type",
                );
            }
        }
    }
}

impl<'ast> Visit<'ast> for OrderingSites<'_> {
    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        self.derives(&item.ident, &item.attrs, item.fields.iter().map(|f| &f.ty));
        visit::visit_item_struct(self, item);
    }
    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        self.derives(
            &item.ident,
            &item.attrs,
            item.variants
                .iter()
                .flat_map(|v| v.fields.iter().map(|f| &f.ty)),
        );
        visit::visit_item_enum(self, item);
    }
    fn visit_item_union(&mut self, item: &'ast syn::ItemUnion) {
        self.derives(
            &item.ident,
            &item.attrs,
            item.fields.named.iter().map(|f| &f.ty),
        );
        visit::visit_item_union(self, item);
    }
    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        self.item = match item.self_ty.as_ref() {
            Type::Path(path) => path.path.segments.last().map(|s| s.ident.to_string()),
            _ => None,
        }
        .unwrap_or_default();
        if item.trait_.as_ref().is_some_and(|(_, path, _)| {
            path.segments
                .last()
                .is_some_and(|s| self.vocabulary.ordering.contains(&s.ident.to_string()))
        }) && self
            .local_fields
            .get(&self.item)
            .map(|fields| fields.iter().any(|field| self.type_ids.contains(field)))
            .unwrap_or_else(|| self.type_ids.contains(&self.item))
        {
            self.record(
                item.span().start().line,
                "ordering impl on a TypeId-bearing type",
            );
        }
        visit::visit_item_impl(self, item);
    }
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        self.item = item.sig.ident.to_string();
        visit::visit_item_fn(self, item);
    }
    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        self.item = item.sig.ident.to_string();
        visit::visit_impl_item_fn(self, item);
    }
    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        self.item = item.sig.ident.to_string();
        visit::visit_trait_item_fn(self, item);
    }
    fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
        self.item = item.ident.to_string();
        visit::visit_item_type(self, item);
    }
    fn visit_path(&mut self, path: &'ast syn::Path) {
        for segment in &path.segments {
            if !self
                .vocabulary
                .containers
                .contains(&segment.ident.to_string())
            {
                continue;
            }
            let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                continue;
            };
            if let Some(syn::GenericArgument::Type(key)) = arguments.args.first() {
                if field_names([key])
                    .iter()
                    .any(|name| self.type_ids.contains(name))
                {
                    self.record(
                        path.span().start().line,
                        "ordered container keyed by TypeId",
                    );
                }
            }
        }
        visit::visit_path(self, path);
    }
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        if super::expression_identity::decides_order(&call.method.to_string())
            && super::expression_identity::names_type_identity(
                std::iter::once(call.receiver.as_ref()).chain(call.args.iter()),
                &self.vocabulary.type_ids,
            )
        {
            self.record(call.span().start().line, "ordering expression names TypeId");
        }
        visit::visit_expr_method_call(self, call);
    }
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        let deciding = match call.func.as_ref() {
            syn::Expr::Path(path) => path.path.segments.last().is_some_and(|segment| {
                super::expression_identity::decides_order(&segment.ident.to_string())
            }),
            _ => false,
        };
        if deciding
            && super::expression_identity::names_type_identity(
                call.args.iter(),
                &self.vocabulary.type_ids,
            )
        {
            self.record(call.span().start().line, "ordering expression names TypeId");
        }
        visit::visit_expr_call(self, call);
    }
    fn visit_item_mod(&mut self, _: &'ast syn::ItemMod) {}
}
