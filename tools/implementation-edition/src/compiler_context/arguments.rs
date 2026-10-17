//! Rustc argument roles. Semantic payloads are opaque bytes; only established
//! path fields undergo relocation normalization.

use super::normalize_path;
use serde_json::Value;
use std::{fs, path::Path};

pub(super) fn semantic(arguments: &[String], session: &Value) -> Result<Vec<String>, String> {
    let mut result = Vec::new();
    let mut index = 0;
    while index < arguments.len() {
        let argument = &arguments[index];
        let (name, joined) = argument
            .split_once('=')
            .map_or((argument.as_str(), None), |(name, value)| {
                (name, Some(value))
            });
        match name {
            "--out-dir" | "-o" | "--error-format" | "--json" | "--diagnostic-width" | "--color" => {
                take_value(arguments, &mut index, joined)?;
            }
            "--extern" => {
                let value = take_value(arguments, &mut index, joined)?;
                let (name, _) = value
                    .split_once('=')
                    .ok_or("extern must name an observed artifact")?;
                result.push(format!("--extern={name}"));
            }
            "-L" => {
                let value = take_value(arguments, &mut index, joined)?;
                dependency_search(value)?;
            }
            "-C" => codegen(
                take_value(arguments, &mut index, joined)?,
                &mut result,
                session,
            )?,
            "--emit" => {
                let value = take_value(arguments, &mut index, joined)?;
                result.push(format!(
                    "--emit={}",
                    value
                        .split(',')
                        .map(|kind| kind.split('=').next().unwrap())
                        .collect::<Vec<_>>()
                        .join(",")
                ));
            }
            "--cfg" | "--check-cfg" | "--crate-name" | "--crate-type" | "--edition"
            | "--target" | "--cap-lints" | "--force-warn" | "--allow" | "--warn" | "--deny"
            | "--forbid" | "-A" | "-W" | "-D" | "-F" => {
                let value = take_value(arguments, &mut index, joined)?;
                // A cfg string may contain literal backslashes or path-shaped
                // text. Neither is a path field.
                result.push(format!("{name}={value}"));
            }
            "--remap-path-prefix" => {
                let value = take_value(arguments, &mut index, joined)?;
                let (from, to) = value.split_once('=').ok_or("remap prefix pair missing")?;
                result.push(format!(
                    "--remap-path-prefix={}={to}",
                    normalize_path(from, session)
                ));
            }
            "--test" => result.push(argument.clone()),
            _ if argument.starts_with("-C") => codegen(&argument[2..], &mut result, session)?,
            _ if argument.starts_with("-L") => dependency_search(&argument[2..])?,
            _ if ["-A", "-W", "-D", "-F"]
                .iter()
                .any(|prefix| argument.starts_with(prefix)) =>
            {
                result.push(argument.clone())
            }
            _ if !argument.starts_with('-') && Path::new(argument).is_file() => {
                let source = fs::canonicalize(argument).map_err(|error| error.to_string())?;
                result.push(normalize_path(&source.to_string_lossy(), session));
            }
            _ => {
                return Err(format!(
                    "compiler option requires input qualification: {name}"
                ))
            }
        }
        index += 1;
    }
    Ok(result)
}

/// Read activation from role-validated arguments, including every cfg occurrence.
pub(super) fn test_active(semantic: &[String]) -> bool {
    semantic.iter().any(|argument| {
        argument == "--test"
            || argument.strip_prefix("--cfg=").is_some_and(|cfg| {
                matches!(syn::parse_str::<syn::Meta>(cfg), Ok(syn::Meta::Path(path))
                    if path.get_ident().is_some_and(|name| crate::rust_identifier::semantic_name(name) == "test"))
            })
    })
}

fn take_value<'a>(
    arguments: &'a [String],
    index: &mut usize,
    joined: Option<&'a str>,
) -> Result<&'a str, String> {
    if let Some(value) = joined {
        return Ok(value);
    }
    *index += 1;
    arguments
        .get(*index)
        .map(String::as_str)
        .ok_or("compiler option value missing".into())
}

fn dependency_search(value: &str) -> Result<(), String> {
    if value.starts_with("dependency=") {
        Ok(())
    } else {
        Err("native/unclassified library search path requires input qualification".into())
    }
}

fn codegen(value: &str, result: &mut Vec<String>, session: &Value) -> Result<(), String> {
    let name = value.split('=').next().unwrap();
    match name {
        "incremental" | "extra-filename" | "metadata" => {}
        "linker" => {
            let path = value.strip_prefix("linker=").ok_or("linker path missing")?;
            if !Path::new(path).is_file() {
                return Err("linker executable content requires input qualification".into());
            }
            let sysroot = Path::new(
                session["compiler_path"]
                    .as_str()
                    .ok_or("selected compiler missing")?,
            )
            .parent()
            .and_then(Path::parent)
            .ok_or("selected compiler sysroot missing")?;
            let linker = fs::canonicalize(path).map_err(|error| error.to_string())?;
            if !linker.starts_with(fs::canonicalize(sysroot).map_err(|error| error.to_string())?)
                || linker.file_stem().is_none_or(|name| name != "rust-lld")
            {
                return Err("non-toolchain linker requires input qualification".into());
            }
            result.push(format!(
                "linker-content={}",
                crate::fingerprint::bytes(&fs::read(path).map_err(|error| error.to_string())?)
            ));
        }
        "opt-level"
        | "debug-assertions"
        | "overflow-checks"
        | "embed-bitcode"
        | "strip"
        | "debuginfo"
        | "panic"
        | "codegen-units"
        | "target-cpu"
        | "target-feature"
        | "lto"
        | "relocation-model"
        | "code-model"
        | "force-frame-pointers"
        | "force-unwind-tables"
        | "symbol-mangling-version" => result.push(format!("-C{value}")),
        _ => {
            return Err(format!(
                "native/input-bearing codegen option requires input qualification: {name}"
            ))
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_activation_uses_boolean_cfg_in_any_argument_position() {
        let session = json!({"packages": []});
        for args in [
            vec!["--test"],
            vec!["--cfg", "test"],
            vec!["--cfg=test"],
            vec!["--cfg", " test "],
            vec!["--cfg=test /* activation */"],
            vec!["--cfg", "r#test"],
            vec!["--cfg=r#test"],
            vec!["--cfg", "feature=\"x\"", "--cfg", "test"],
        ] {
            let args = args.into_iter().map(str::to_owned).collect::<Vec<_>>();
            assert!(test_active(&semantic(&args, &session).unwrap()));
        }
        for args in [
            vec![],
            vec!["--cfg", "test=\"value\""],
            vec!["--cfg", "r#test=\"value\""],
            vec!["--cfg", "testing"],
        ] {
            let args = args.into_iter().map(str::to_owned).collect::<Vec<_>>();
            assert!(!test_active(&semantic(&args, &session).unwrap()));
        }
    }

    #[test]
    fn cfg_payloads_are_never_rewritten_as_paths() {
        let session = json!({"packages": []});
        let backslash = vec!["--cfg".into(), r#"select="a\\b""#.into()];
        let slash = vec!["--cfg".into(), r#"select="a//b""#.into()];
        assert_ne!(
            semantic(&backslash, &session).unwrap(),
            semantic(&slash, &session).unwrap()
        );
    }

    #[test]
    fn separated_and_joined_native_input_options_are_refused() {
        let session = json!({"packages": []});
        for args in [
            vec!["-C", "link-arg=library.a"],
            vec!["-Clink-arg=library.a"],
            vec!["-C", "link-args=@response"],
            vec!["-lstatic=foreign"],
            vec!["-Lnative=foreign"],
            vec!["-Cprofile-use=profile.data"],
        ] {
            assert!(semantic(
                &args.into_iter().map(str::to_owned).collect::<Vec<_>>(),
                &session
            )
            .is_err());
        }
    }
}
