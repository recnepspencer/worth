//! Derive factory return types from the registered roots' owning source crates.
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use syn::visit::{self, Visit};

#[derive(Default)]
pub(super) struct Factories(BTreeMap<(String, String, String), BTreeSet<String>>);
impl Factories {
    pub(super) fn derive(root: &Path, owners: &BTreeSet<String>) -> Result<Self, String> {
        let mut result = Self::default();
        for owner in owners {
            let directory = root.join(owner);
            let manifest = crate::source_documents::read_to_string(&directory.join("Cargo.toml"))
                .map_err(|e| e.to_string())?;
            let manifest: toml::Value = toml::from_str(&manifest).map_err(|e| e.to_string())?;
            let package = manifest["package"]["name"]
                .as_str()
                .ok_or("factory owner has no package name")?
                .replace('-', "_");
            let mut pending = vec![directory.join("src")];
            while let Some(path) = pending.pop() {
                if !path.exists() {
                    continue;
                }
                let canonical = path.canonicalize().map_err(|e| e.to_string())?;
                let boundary = directory.canonicalize().map_err(|e| e.to_string())?;
                if !canonical.starts_with(&boundary) {
                    return Err(format!(
                        "factory source escapes root owner: {}",
                        path.display()
                    ));
                }
                if path.is_dir() {
                    for entry in std::fs::read_dir(&path).map_err(|e| e.to_string())? {
                        pending.push(entry.map_err(|e| e.to_string())?.path());
                    }
                } else if path.extension().is_some_and(|e| e == "rs") {
                    let source = crate::source_documents::read_to_string(&path)
                        .map_err(|e| e.to_string())?;
                    if !source.contains("fn ") {
                        continue;
                    }
                    let file = syn::parse_file(&source)
                        .map_err(|e| format!("factory signatures {}: {e}", path.display()))?;
                    Signatures {
                        package: &package,
                        owner: String::new(),
                        into: &mut result,
                    }
                    .visit_file(&file);
                }
            }
        }
        Ok(result)
    }
    pub(super) fn returns(&self, identity: &str, method: &str) -> Option<String> {
        let (package, ty) = identity.split_once("::")?;
        let candidates = self.0.get(&(package.into(), ty.into(), method.into()))?;
        (candidates.len() == 1).then(|| candidates.first().unwrap().clone())
    }
    pub(super) fn free_returns(&self, package: &str, function: &str) -> Option<String> {
        self.returns(&format!("{package}::"), function)
    }
}
struct Signatures<'a> {
    package: &'a str,
    owner: String,
    into: &'a mut Factories,
}
impl Signatures<'_> {
    fn record(&mut self, sig: &syn::Signature) {
        let syn::ReturnType::Type(_, ty) = &sig.output else {
            return;
        };
        if let Some(result) = returned_type(ty, self.package, &self.owner) {
            self.into
                .0
                .entry((
                    self.package.into(),
                    self.owner.clone(),
                    sig.ident.to_string(),
                ))
                .or_default()
                .insert(result);
        }
    }
}
impl<'ast> Visit<'ast> for Signatures<'_> {
    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let syn::Type::Path(ty) = &*item.self_ty else {
            return;
        };
        let Some(name) = ty.path.segments.last() else {
            return;
        };
        let previous = std::mem::replace(&mut self.owner, name.ident.to_string());
        for member in &item.items {
            if let syn::ImplItem::Fn(method) = member {
                self.record(&method.sig);
            }
        }
        self.owner = previous;
    }
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        self.record(&item.sig);
    }
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        visit::visit_item_mod(self, item);
    }
}
fn returned_type(ty: &syn::Type, package: &str, owner: &str) -> Option<String> {
    let syn::Type::Path(ty) = ty else {
        return None;
    };
    let last = ty.path.segments.last()?;
    let name = last.ident.to_string();
    if ["Result", "Option"].contains(&name.as_str()) {
        if let syn::PathArguments::AngleBracketed(args) = &last.arguments {
            if let Some(syn::GenericArgument::Type(ty)) = args.args.first() {
                return returned_type(ty, package, owner);
            }
        }
        return None;
    }
    let first = ty.path.segments.first()?.ident.to_string();
    let namespace =
        if ty.path.segments.len() > 1 && !["crate", "self", "super"].contains(&first.as_str()) {
            first.as_str()
        } else {
            package
        };
    let name = if name == "Self" { owner } else { name.as_str() };
    (!name.is_empty()).then(|| format!("{namespace}::{name}"))
}
