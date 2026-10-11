//! Own-unit-test modules ship no doors; their parent mount determines membership.
use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path, PathBuf},
};

pub(super) fn files_under(root: &Path) -> Result<Vec<(PathBuf, String)>, String> {
    let mut files = Vec::new();
    for file in crate::governed_sources::files_under(root)? {
        let name = file.to_string_lossy().replace('\\', "/");
        if file.extension().is_none_or(|ext| ext != "rs")
            || name.contains("/tests/")
            || name.contains("/fixtures/")
            || name.contains("/examples/")
        {
            continue;
        }
        let source = fs::read_to_string(&file).map_err(|error| error.to_string())?;
        files.push((file, source));
    }
    let mut excluded = BTreeSet::new();
    for (file, source) in &files {
        if !source.contains("cfg(test)") {
            continue;
        }
        let parsed =
            syn::parse_file(source).map_err(|error| format!("{}: {error}", file.display()))?;
        let parent = file.parent().unwrap();
        let directory = if matches!(
            file.file_name().and_then(|name| name.to_str()),
            Some("lib.rs" | "main.rs" | "mod.rs")
        ) {
            parent.to_path_buf()
        } else {
            file.with_extension("")
        };
        mounts(&parsed.items, &directory, parent, &mut excluded);
    }
    Ok(files
        .into_iter()
        .filter(|(file, _)| {
            !excluded
                .iter()
                .any(|prefix| normalized(file).starts_with(prefix))
        })
        .collect())
}

fn mounts(
    items: &[syn::Item],
    directory: &Path,
    attribute_directory: &Path,
    excluded: &mut BTreeSet<PathBuf>,
) {
    for item in items {
        let syn::Item::Mod(module) = item else {
            continue;
        };
        if let Some((_, items)) = &module.content {
            if !super::source_regions::test_only(&module.attrs) {
                let directory = directory.join(module.ident.to_string());
                mounts(items, &directory, &directory, excluded);
            }
        } else if super::source_regions::test_only(&module.attrs) {
            let path = module.attrs.iter().find_map(|attr| {
                if !attr.path().is_ident("path") {
                    return None;
                }
                let syn::Meta::NameValue(value) = &attr.meta else {
                    return None;
                };
                let syn::Expr::Lit(value) = &value.value else {
                    return None;
                };
                let syn::Lit::Str(value) = &value.lit else {
                    return None;
                };
                Some(attribute_directory.join(value.value()))
            });
            if let Some(path) = path {
                excluded.insert(normalized(&path));
                excluded.insert(normalized(
                    &if path.file_name().is_some_and(|name| name == "mod.rs") {
                        path.parent().unwrap().to_path_buf()
                    } else {
                        path.with_extension("")
                    },
                ));
            } else {
                let path = directory.join(module.ident.to_string());
                excluded.insert(normalized(&path.with_extension("rs")));
                excluded.insert(normalized(&path));
            }
        }
    }
}

fn normalized(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {}
            other => result.push(other),
        }
    }
    result
}
