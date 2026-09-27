use std::path::Path;

use syn::{Item, UseTree, Visibility};

pub(super) fn validate_exact_public_reexport(
    path: &Path,
    required_route: &str,
) -> Result<(), String> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| format!("read {}: {error}", path.display()))?;
    let syntax = crate::source_syntax::parse_file(&text)
        .map_err(|error| format!("parse {}: {error}", path.display()))?;
    let mut routes = Vec::new();
    for item in syntax.items {
        if let Item::Use(item_use) = item {
            if matches!(item_use.vis, Visibility::Public(_)) {
                collect_routes(&item_use.tree, &mut Vec::new(), &mut routes, path)?;
            }
        }
    }
    if routes.iter().any(|route| route == required_route) {
        Ok(())
    } else {
        Err(format!(
            "configured facade route `{required_route}` is absent from {}",
            path.display()
        ))
    }
}

fn collect_routes(
    tree: &UseTree,
    prefix: &mut Vec<String>,
    routes: &mut Vec<String>,
    path: &Path,
) -> Result<(), String> {
    match tree {
        UseTree::Path(value) => {
            prefix.push(value.ident.to_string());
            let result = collect_routes(&value.tree, prefix, routes, path);
            prefix.pop();
            result
        }
        UseTree::Name(value) if value.ident == "self" => {
            routes.push(prefix.join("::"));
            Ok(())
        }
        UseTree::Name(value) => {
            prefix.push(value.ident.to_string());
            routes.push(prefix.join("::"));
            prefix.pop();
            Ok(())
        }
        UseTree::Rename(value) => {
            prefix.push(value.ident.to_string());
            routes.push(prefix.join("::"));
            prefix.pop();
            Ok(())
        }
        UseTree::Group(value) => value
            .items
            .iter()
            .try_for_each(|item| collect_routes(item, prefix, routes, path)),
        UseTree::Glob(_) => Err(format!(
            "glob export cannot prove an exact facade route in {}",
            path.display()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_route_rejects_a_same_named_retarget() {
        let path = temporary_facade("pub use worth_query_execution::facade::primary_graph;\n");
        assert!(validate_exact_public_reexport(
            &path,
            "worth_query_execution::facade::primary_graph"
        )
        .is_ok());
        std::fs::write(
            &path,
            "pub use worth_query_execution::domain_computation as primary_graph;\n",
        )
        .unwrap();
        assert!(validate_exact_public_reexport(
            &path,
            "worth_query_execution::facade::primary_graph"
        )
        .is_err());
        std::fs::remove_file(path).unwrap();
    }

    fn temporary_facade(contents: &str) -> std::path::PathBuf {
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("boundary-check-route-{id}.rs"));
        std::fs::write(&path, contents).unwrap();
        path
    }
}
