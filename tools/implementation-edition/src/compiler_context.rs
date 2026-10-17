use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::{json, Value};

use crate::fingerprint;

mod arguments;

pub(crate) struct Context {
    pub(crate) package: String,
    pub(crate) root: PathBuf,
    pub(crate) output: PathBuf,
    pub(crate) arguments: Vec<String>,
    pub(crate) semantic: Value,
    pub(crate) test: bool,
}

pub(crate) fn read(
    compiler: &str,
    arguments: &[String],
    session: &Value,
) -> Result<Context, String> {
    let selected = session["compiler_path"]
        .as_str()
        .ok_or("selected compiler missing")?;
    if fs::canonicalize(compiler).map_err(|error| error.to_string())?
        != fs::canonicalize(selected).map_err(|error| error.to_string())?
    {
        return Err("unqualified nested compiler wrapper; use the selected rustc directly".into());
    }
    let root =
        fs::canonicalize(std::env::var_os("CARGO_MANIFEST_DIR").ok_or("Cargo unit root missing")?)
            .map_err(|error| error.to_string())?;
    let package = session["packages"]
        .as_array()
        .ok_or("observer packages missing")?
        .iter()
        .find(|package| Path::new(package["root"].as_str().unwrap_or_default()) == root)
        .ok_or("unit is outside the verified workspace package set")?;
    if arguments.iter().any(|arg| arg.starts_with("--sysroot"))
        || flag(arguments, "--crate-type")
            .is_some_and(|types| types.split(',').any(|kind| kind == "proc-macro"))
    {
        return Err("proc-macro or alternate-sysroot context requires input qualification".into());
    }
    let output =
        PathBuf::from(flag(arguments, "--out-dir").ok_or("compiler unit has no output directory")?);
    let links = flag(arguments, "--crate-type")
        .is_some_and(|types| types.split(',').any(|kind| kind == "bin"))
        || arguments.iter().any(|argument| argument == "--test");
    if links
        && !arguments
            .iter()
            .any(|argument| argument.starts_with("linker=") || argument.starts_with("-Clinker="))
    {
        return Err("default native linker context requires input qualification".into());
    }
    let mut actual = arguments.to_vec();
    // Stable virtual source paths also preserve file! semantics across checkouts.
    for package in session["packages"].as_array().unwrap() {
        let root = package["root"].as_str().ok_or("package path missing")?;
        let name = package["name"].as_str().ok_or("package name missing")?;
        actual.push(format!("--remap-path-prefix={root}=/worth/packages/{name}"));
        let ordinary = root.trim_start_matches("\\\\?\\");
        if ordinary != root {
            actual.push(format!(
                "--remap-path-prefix={ordinary}=/worth/packages/{name}"
            ));
        }
    }
    if let Some(out) = std::env::var_os("OUT_DIR") {
        actual.push(format!(
            "--remap-path-prefix={}=/worth/generated/{}",
            Path::new(&out).display(),
            package["name"].as_str().unwrap()
        ));
    }
    let identity = if session["compiler_path"] == compiler
        && flag(arguments, "--target")
            .is_none_or(|target| session["compiler_identity"]["target"] == target)
    {
        session["compiler_identity"].clone()
    } else {
        compiler_identity(compiler, flag(arguments, "--target"))?
    };
    let semantic_arguments = arguments::semantic(arguments, session)?;
    let test = arguments::test_active(&semantic_arguments);
    let semantic = json!({
        "compiler": identity,
        "package": [package["name"].clone(), package["version"].clone()],
        "arguments": semantic_arguments,
    });
    Ok(Context {
        package: package["name"].as_str().unwrap().into(),
        root,
        output,
        arguments: actual,
        semantic,
        test,
    })
}

pub(crate) fn flag<'a>(arguments: &'a [String], name: &str) -> Option<&'a str> {
    arguments.iter().enumerate().find_map(|(index, arg)| {
        if arg == name {
            arguments.get(index + 1).map(String::as_str)
        } else {
            arg.strip_prefix(&format!("{name}="))
        }
    })
}

pub(crate) fn compiler_identity(compiler: &str, target: Option<&str>) -> Result<Value, String> {
    let version = Command::new(compiler)
        .arg("-vV")
        .output()
        .map_err(|error| error.to_string())?;
    let sysroot = Command::new(compiler)
        .arg("--print=sysroot")
        .output()
        .map_err(|error| error.to_string())?;
    if !version.status.success() || !sysroot.status.success() {
        return Err("compiler identity query failed".into());
    }
    let version = String::from_utf8(version.stdout).map_err(|error| error.to_string())?;
    let host = version
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .ok_or("compiler host missing")?;
    let target = target.unwrap_or(host);
    if target.ends_with(".json") {
        return Err("custom target JSON requires input qualification".into());
    }
    let sysroot = PathBuf::from(
        String::from_utf8(sysroot.stdout)
            .map_err(|error| error.to_string())?
            .trim(),
    );
    let mut content = std::collections::BTreeMap::new();
    for directory in [
        sysroot.join("bin"),
        sysroot.join("lib/rustlib").join(target).join("lib"),
    ] {
        for entry in
            fs::read_dir(directory).map_err(|error| format!("compiler content: {error}"))?
        {
            let path = entry.map_err(|error| error.to_string())?.path();
            let name = path.file_name().unwrap().to_string_lossy();
            if path.is_file()
                && (name.starts_with("rustc") || path.extension().is_some_and(|ext| ext == "rlib"))
            {
                content.insert(
                    name.into_owned(),
                    fingerprint::bytes(&fs::read(path).map_err(|error| error.to_string())?),
                );
            }
        }
    }
    Ok(json!({"version": version, "target": target, "content": content}))
}

pub(crate) fn compiler_path(compiler: &str) -> Result<String, String> {
    let output = Command::new(compiler)
        .arg("--print=sysroot")
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err("configured compiler sysroot query failed".into());
    }
    let sysroot = String::from_utf8(output.stdout).map_err(|error| error.to_string())?;
    Ok(Path::new(sysroot.trim())
        .join("bin")
        .join(if cfg!(windows) { "rustc.exe" } else { "rustc" })
        .to_string_lossy()
        .into_owned())
}

pub(crate) fn normalize_path(value: &str, session: &Value) -> String {
    let mut value = value.replace('\\', "/").replace("//?/", "");
    if let Some(out) = std::env::var_os("OUT_DIR")
        && let Ok(package) = std::env::var("CARGO_PKG_NAME")
    {
        let root = out.to_string_lossy().replace('\\', "/").replace("//?/", "");
        if let Some(suffix) = path_suffix(&value, &root) {
            return format!("/worth/generated/{package}{suffix}");
        }
    }
    // Longest prefix first prevents a workspace package swallowing a child.
    let mut roots: Vec<_> = session["packages"].as_array().unwrap().iter().collect();
    roots.sort_by_key(|package| std::cmp::Reverse(package["root"].as_str().unwrap().len()));
    for package in roots {
        let root = package["root"]
            .as_str()
            .unwrap()
            .replace('\\', "/")
            .replace("//?/", "");
        if let Some(suffix) = path_suffix(&value, &root) {
            value = format!(
                "/worth/packages/{}{suffix}",
                package["name"].as_str().unwrap()
            );
            break;
        }
    }
    value
}

fn path_suffix<'a>(value: &'a str, root: &str) -> Option<&'a str> {
    value
        .strip_prefix(root)
        .filter(|suffix| suffix.is_empty() || suffix.starts_with('/'))
}
