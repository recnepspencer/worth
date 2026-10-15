//! Immediate payload names must have an actual public route, not a guessed basename.
use super::{CrateIndex, Definition, Resolver};
use std::path::Path;

pub(crate) fn payload_name(
    root: &Path,
    manifest: &str,
    audience: &str,
    item: &syn::ItemEnum,
    ty: &syn::Type,
    crates: &CrateIndex,
) -> Result<String, String> {
    let syn::Type::Path(path) = ty else {
        return Err("unsupported fact payload type".into());
    };
    if path.qself.is_some()
        || path
            .path
            .segments
            .iter()
            .any(|s| !matches!(s.arguments, syn::PathArguments::None))
    {
        return Err("generic/qualified payload needs an owning fact emitter".into());
    }
    let segments = path
        .path
        .segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>();
    let name = segments.last().ok_or("empty payload path")?;
    if segments.len() == 1
        && item
            .generics
            .type_params()
            .any(|p| p.ident == name.as_str())
    {
        return Ok(name.clone());
    }
    let id = crates.load(root.join(manifest).parent().ok_or("missing crate root")?)?;
    let resolver = Resolver::new(crates);
    let audience_path = audience.split("::").map(str::to_owned).collect::<Vec<_>>();
    let definitions = resolver.resolve_public_path(id, &audience_path)?;
    let [Definition::Item { krate, module, .. }] = definitions.as_slice() else {
        return Err("missing enum payload owner".into());
    };
    let actual = resolver.resolve_path(*krate, module, &segments)?;
    if actual.is_empty()
        && segments.len() == 1
        && [
            "bool", "char", "str", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32",
            "i64", "i128", "isize", "f32", "f64",
        ]
        .contains(&name.as_str())
    {
        return Ok(name.clone());
    }
    if actual.len() != 1 || !matches!(actual[0], Definition::Item { .. }) {
        return Err("payload source definition is unsupported".into());
    }
    super::knowledge::require_definition_source(crates, &actual[0])?;
    if let Some(name) =
        audience_payload_route(crates, &resolver, id, &audience_path, name, &actual)?
    {
        return Ok(name);
    }
    // Fully qualified source payloads may name another public owning facade.
    // Verify every public segment there; never shorten an unproven local alias.
    if segments.len() > 1 && !["crate", "self", "super"].contains(&segments[0].as_str()) {
        let root = resolver.resolve_path(*krate, &[], &segments[..1])?;
        if let [Definition::Module {
            krate: dependency,
            path,
        }] = root.as_slice()
        {
            if path.is_empty()
                && resolver.resolve_public_path(*dependency, &segments[1..])? == actual
            {
                super::knowledge::require_fact_route(
                    crates,
                    &resolver,
                    *dependency,
                    &segments[1..],
                )?;
                return Ok(segments.join("::"));
            }
        }
    }
    Err(format!("payload {name} has no supported public audience route (aliases require an explicit owning route)"))
}

fn audience_payload_route(
    crates: &CrateIndex,
    resolver: &Resolver<'_>,
    id: super::CrateId,
    audience_path: &[String],
    name: &String,
    actual: &[Definition],
) -> Result<Option<String>, String> {
    let mut candidate = audience_path[..audience_path.len() - 1].to_vec();
    candidate.push(name.clone());
    if resolver.resolve_public_path(id, &candidate)? == actual {
        super::knowledge::require_fact_route(crates, resolver, id, &candidate)?;
        return Ok(Some(name.clone()));
    }
    let parent = &audience_path[..audience_path.len() - 1];
    for definition in resolver.resolve_public_path(id, parent)? {
        if let Definition::Module { krate, path } = definition {
            let owner = crates.crate_at(krate);
            let node = owner
                .graph
                .modules
                .get(&path)
                .ok_or("missing audience module")?;
            for member in &node.items {
                let syn::Item::Use(import) = member else {
                    continue;
                };
                if !matches!(import.vis, syn::Visibility::Public(_)) {
                    continue;
                }
                for binding in super::use_bindings::use_bindings(import) {
                    let super::use_bindings::BoundName::Named(alias) = binding.bound else {
                        continue;
                    };
                    let mut route = parent.to_vec();
                    route.push(alias.clone());
                    if resolver.resolve_public_path(id, &route)? == actual {
                        super::knowledge::require_fact_route(crates, resolver, id, &route)?;
                        return Ok(Some(alias));
                    }
                }
            }
        }
    }
    // Audience sibling modules are actual declared modules/reexports, not a
    // maintained payload-name inventory. This covers primary_graph payloads
    // carried by an application_entry outcome without exposing kernel layout.
    if let [Definition::Module {
        krate: facade_crate,
        path: facade_path,
    }] = resolver
        .resolve_public_path(id, &["facade".into()])?
        .as_slice()
    {
        let owner = crates.crate_at(*facade_crate);
        let node = owner
            .graph
            .modules
            .get(facade_path)
            .ok_or("missing audience facade")?;
        let mut modules = Vec::new();
        for member in &node.items {
            match member {
                syn::Item::Mod(m) if matches!(m.vis, syn::Visibility::Public(_)) => {
                    modules.push(m.ident.to_string())
                }
                syn::Item::Use(u) if matches!(u.vis, syn::Visibility::Public(_)) => {
                    for binding in super::use_bindings::use_bindings(u) {
                        if let super::use_bindings::BoundName::Named(name) = binding.bound {
                            modules.push(name);
                        }
                    }
                }
                _ => {}
            }
        }
        for module in modules {
            let route = vec!["facade".into(), module, name.clone()];
            // An exported non-module cannot contain a payload route.
            if let Ok(found) = resolver.resolve_public_path(id, &route) {
                if found == actual {
                    super::knowledge::require_fact_route(crates, resolver, id, &route)?;
                    return Ok(Some(name.clone()));
                }
            }
        }
    }
    Ok(None)
}
