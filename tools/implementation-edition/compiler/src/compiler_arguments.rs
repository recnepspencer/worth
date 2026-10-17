//! Closed compiler subset until the qualified Cargo unit-context owner is joined.
//! In particular response files and implicit library search cannot load a DLL
//! before the expanded-input callback has a chance to inspect syntax.
pub(super) fn check(arguments: &[String]) -> Result<(), String> {
    if arguments.iter().any(|arg| arg.starts_with('@')) {
        return Err("response-file compiler context requires qualification".into());
    }
    let mut args = arguments.iter();
    while let Some(argument) = args.next() {
        if matches!(argument.as_str(), "-vV" | "-V" | "--version") {
            continue;
        }
        let (name, attached) = argument
            .split_once('=')
            .map_or((argument.as_str(), None), |(n, v)| (n, Some(v)));
        if name == "--extern" || name.starts_with("-L") {
            return Err("external artifacts require the qualified unit dependency owner".into());
        }
        if name == "--sysroot" {
            return Err("alternate sysroots require qualification".into());
        }
        if matches!(
            name,
            "--print"
                | "--crate-type"
                | "--emit"
                | "--edition"
                | "--crate-name"
                | "--cfg"
                | "--check-cfg"
                | "--cap-lints"
                | "--color"
                | "--error-format"
                | "--json"
                | "--out-dir"
                | "-o"
        ) {
            let value = attached
                .or_else(|| args.next().map(String::as_str))
                .ok_or_else(|| format!("missing compiler value: {name}"))?;
            let valid = match name {
                "--print" => matches!(value, "sysroot" | "file-names"),
                "--crate-type" => matches!(value, "lib" | "rlib"),
                "--emit" => value
                    .split(',')
                    .all(|v| matches!(v, "dep-info" | "metadata" | "link")),
                "--edition" => matches!(value, "2015" | "2018" | "2021" | "2024"),
                _ => true,
            };
            if !valid {
                return Err(format!("unqualified compiler context: {name}={value}"));
            }
        } else if argument.starts_with('-') || !argument.ends_with(".rs") {
            return Err(format!("unqualified compiler argument: {argument}"));
        }
    }
    Ok(())
}
