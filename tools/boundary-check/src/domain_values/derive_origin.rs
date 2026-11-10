//! Glob imports may shadow a derive; certify their actual exported namespaces.
use super::{
    modules::{Module, Sources},
    type_resolution::Lookup,
};
use std::collections::{BTreeMap, BTreeSet};
use syn::{Attribute, Item};

impl Sources<'_> {
    pub(super) fn certify_context(&self, context: &[String]) -> Result<Module, String> {
        if let Some(module) = self.certified.borrow().get(context) {
            return Ok(module.clone());
        }
        let mut pending = vec![context.to_vec()];
        let mut checked = BTreeMap::new();
        while let Some(context) = pending.pop() {
            if checked.contains_key(&context) || self.certified.borrow().contains_key(&context) {
                continue;
            }
            let module = self.source_context(&context)?;
            let mut lookup = Lookup::default();
            for namespace in derive_namespaces(&module.items)? {
                if self
                    .glob_candidate(&module, std::slice::from_ref(&namespace), 0, &mut lookup)?
                    .is_some()
                {
                    return Err(format!("glob-shadowed derive namespace: {namespace}"));
                }
            }
            checked.insert(context, module);
            pending.extend(lookup.contexts);
        }
        self.certified.borrow_mut().extend(checked);
        Ok(self.certified.borrow()[context].clone())
    }
}

fn derive_namespaces(items: &[Item]) -> Result<BTreeSet<String>, String> {
    let mut namespaces = BTreeSet::new();
    for item in items {
        let attributes: &[Attribute] = match item {
            Item::Struct(value) => &value.attrs,
            Item::Enum(value) => &value.attrs,
            Item::ExternCrate(value)
                if value
                    .attrs
                    .iter()
                    .any(|attr| attr.path().is_ident("macro_use")) =>
            {
                return Err("external macro imports require an explicit source origin".into());
            }
            _ => continue,
        };
        for attribute in attributes
            .iter()
            .filter(|attr| attr.path().is_ident("derive"))
        {
            let paths = attribute
                .parse_args_with(
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                )
                .map_err(|error| error.to_string())?;
            for path in paths {
                if path.leading_colon.is_none() {
                    namespaces.insert(
                        path.segments
                            .first()
                            .ok_or("empty derive path")?
                            .ident
                            .to_string(),
                    );
                }
            }
        }
    }
    Ok(namespaces)
}
