use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::Value;

use crate::{
    cargo_build::{REOBSERVE, SESSION},
    compiler_context::{self, Context},
    inputs, stamp,
};

mod generated;
mod generator_contract;
pub(crate) mod generator_execution;

pub(crate) fn compile(compiler: &str, arguments: &[String]) -> Result<(), String> {
    // Cargo queries do not compile an artifact and cannot mint a component.
    if arguments.iter().any(|argument| {
        argument == "-vV" || argument == "--print" || argument.starts_with("--print=")
    }) {
        let status = Command::new(compiler)
            .args(arguments)
            .status()
            .map_err(|error| error.to_string())?;
        return if status.success() {
            Ok(())
        } else {
            Err("compiler query failed".into())
        };
    }
    let session_path =
        std::env::var_os(SESSION).ok_or("compiler observation requires its Cargo session")?;
    let session: Value =
        serde_json::from_slice(&fs::read(session_path).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
    if session["run"].as_str() != std::env::var(REOBSERVE).ok().as_deref() {
        return Err("compiler observer session does not match this invocation".into());
    }
    let context = compiler_context::read(compiler, arguments, &session)?;
    let package = session["packages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| Path::new(p["root"].as_str().unwrap()) == context.root)
        .ok_or("compiler package missing")?;
    let content = crate::package_content::verify(package, &session)?;
    let mut dependencies = dependencies(arguments)?;
    let generated = generated::basis(&context, &session)?;
    dependencies.extend(generated.dependencies);
    let directory = Path::new(session["state"].as_str().ok_or("observer state missing")?)
        .join(format!("probe-{}", std::process::id()));
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let probe = directory.join("inputs.d");
    discover(compiler, &context, &probe)?;
    let discovered = inputs::dep_info(&probe)?;
    let generator_source = generator_contract::validate(arguments, &discovered)?;
    let generator_reads = if let Some(source) = &generator_source {
        generator_contract::Program::parse(&fs::read_to_string(source).map_err(|e| e.to_string())?)?
            .reads
            .iter()
            .map(|p| fs::canonicalize(context.root.join(p)).map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?
    } else {
        Vec::new()
    };
    validate_locations(&discovered, &context, &session)?;
    for path in &discovered.files {
        if path.starts_with(&context.root)
            && !crate::package_content::contains_input(&content, &context.root, path)
        {
            return Err("unverified file entered external compiler inputs".into());
        }
        if std::env::var_os("OUT_DIR").is_some_and(|root| path.starts_with(root))
            && !generated.writes.contains(path)
        {
            return Err("compiler input is not a validated generated write".into());
        }
    }
    let mut input_records = inputs::records(&discovered, &session, context.test)?;
    input_records.extend(generated.inputs.clone());
    input_records.extend(content.inputs.clone());
    let mut semantic_context = context.semantic.clone();
    semantic_context["environment"] = inputs::environment(&discovered, &session)?;
    semantic_context["package_content"] = content.identity;
    let generator_reads: Vec<_> = generator_reads.into_iter().map(|path| serde_json::json!({
        "identity": compiler_context::normalize_path(&path.to_string_lossy(), &session), "path": path,
    })).collect();
    semantic_context["generator_reads"] = serde_json::json!(generator_reads
        .iter()
        .map(|input| input["identity"].clone())
        .collect::<Vec<_>>());
    let unit = stamp::admit(
        semantic_context,
        input_records.clone(),
        dependencies,
        generated.semantic,
        generator_reads,
    );
    let artifacts = output_files(compiler, &context)?;
    for artifact in &artifacts {
        let old = stamp::path(artifact);
        if old.exists() {
            fs::remove_file(old).map_err(|error| error.to_string())?;
        }
    }
    let output = Command::new(
        session["wrapper"]
            .as_str()
            .ok_or("sccache wrapper missing")?,
    )
    .arg(compiler)
    .args(&context.arguments)
    .env_remove("CARGO_TARGET_DIR")
    .env_remove(REOBSERVE)
    .env_remove(SESSION)
    .output()
    .map_err(|error| error.to_string())?;
    if !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        return Err("observed compilation failed".into());
    }
    // Re-read actual inputs after compilation; a changing input cannot stamp
    // the artifact with the pre-compilation observation.
    let mut after = inputs::records(&discovered, &session, context.test)?;
    for input in generated.inputs {
        let path = Path::new(
            input["path"]
                .as_str()
                .ok_or("generated input path missing")?,
        );
        if input["digest"]
            != crate::fingerprint::bytes(&fs::read(path).map_err(|error| error.to_string())?)
        {
            return Err("generator inputs changed during compilation; component withheld".into());
        }
        after.push(input);
    }
    after.extend(crate::package_content::verify(package, &session)?.inputs);
    if after != input_records {
        return Err("compiler inputs changed during compilation; component withheld".into());
    }
    let actual_dep = actual_dep_info(&context, arguments)?;
    let actual = inputs::dep_info(&actual_dep)?;
    if actual.files != discovered.files || actual.environment != discovered.environment {
        return Err("discovery and actual compilation input/context closure differ".into());
    }
    stamp::store(&unit, &artifacts)?;
    if let Some(source) = generator_source {
        generator_execution::install(&unit, &artifacts, &source, &context.root, &session)?;
    }
    // This is a descriptive dispatcher dependency, not a semantic compiler
    // input. It forces Cargo to invoke the observer on the next build while
    // leaving rustc/sccache inputs unchanged, including on a cache hit.
    writeln!(
        fs::OpenOptions::new()
            .append(true)
            .open(actual_dep)
            .map_err(|error| error.to_string())?,
        "\n# env-dep:{REOBSERVE}={}",
        std::env::var(REOBSERVE).unwrap()
    )
    .map_err(|error| error.to_string())?;
    // Withhold Cargo's metadata-ready events until both the rmeta and the
    // final compiler artifact have a verified component.
    print!("{}", String::from_utf8_lossy(&output.stdout));
    eprint!("{}", String::from_utf8_lossy(&output.stderr));
    eprintln!(
        "implementation-edition: observed {} {}",
        context.package,
        unit.implementation.closure_digest()
    );
    // Only the file this observation wrote is removed; unrelated contents
    // cannot be swept up by a recursive cleanup of a computed path.
    fs::remove_file(probe).map_err(|error| error.to_string())?;
    fs::remove_dir(directory).map_err(|error| error.to_string())?;
    Ok(())
}

fn dependencies(arguments: &[String]) -> Result<Vec<Value>, String> {
    let mut dependencies = Vec::new();
    for (index, argument) in arguments.iter().enumerate() {
        if argument == "--extern" {
            let external = arguments.get(index + 1).ok_or("extern value missing")?;
            let (_, artifact) = external
                .split_once('=')
                .ok_or("extern without an exact artifact is refused")?;
            dependencies.push(stamp::dependency(Path::new(artifact))?);
        } else if argument.starts_with("--extern=") {
            return Err("joined extern argument requires qualification".into());
        }
    }
    Ok(dependencies)
}

fn discover(compiler: &str, context: &Context, path: &Path) -> Result<(), String> {
    let mut arguments = Vec::new();
    let mut skip = false;
    for argument in &context.arguments {
        if skip {
            skip = false;
            continue;
        }
        if matches!(argument.as_str(), "--emit" | "-o") {
            skip = true;
            continue;
        }
        if argument.starts_with("--emit=") {
            continue;
        }
        arguments.push(argument.clone());
    }
    arguments.push(format!("--emit=dep-info={}", path.display()));
    let output = Command::new(compiler)
        .args(arguments)
        .env_remove(REOBSERVE)
        .env_remove(SESSION)
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "compiler input discovery failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

fn output_files(compiler: &str, context: &Context) -> Result<Vec<PathBuf>, String> {
    let output = Command::new(compiler)
        .args(&context.arguments)
        .args(["--print", "file-names"])
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err("compiler output-name query failed".into());
    }
    let mut artifacts: Vec<_> = String::from_utf8(output.stdout)
        .map_err(|error| error.to_string())?
        .lines()
        .map(|name| context.output.join(name))
        .collect();
    if compiler_context::flag(&context.arguments, "--emit")
        .is_some_and(|emit| emit.split(',').any(|kind| kind == "metadata"))
    {
        let metadata: Vec<_> = artifacts
            .iter()
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "rlib")
            })
            .map(|path| path.with_extension("rmeta"))
            .collect();
        artifacts.extend(metadata);
    }
    Ok(artifacts)
}

fn actual_dep_info(context: &Context, arguments: &[String]) -> Result<PathBuf, String> {
    let name = compiler_context::flag(arguments, "--crate-name").ok_or("crate name missing")?;
    let extra = arguments
        .windows(2)
        .find_map(|args| {
            (args[0] == "-C")
                .then(|| args[1].strip_prefix("extra-filename="))
                .flatten()
        })
        .unwrap_or_default();
    Ok(context.output.join(format!("{name}{extra}.d")))
}

fn validate_locations(
    discovered: &inputs::Discovered,
    context: &Context,
    session: &Value,
) -> Result<(), String> {
    let generated = std::env::var_os("OUT_DIR").map(PathBuf::from);
    for file in &discovered.files {
        let package = session["packages"]
            .as_array()
            .unwrap()
            .iter()
            .any(|package| file.starts_with(Path::new(package["root"].as_str().unwrap())));
        let generated = generated
            .as_ref()
            .is_some_and(|root| file.starts_with(root));
        if !package && !generated {
            return Err(format!(
                "unqualified compiler input outside workspace/generator roots: {} ({})",
                file.display(),
                context.package
            ));
        }
    }
    Ok(())
}
