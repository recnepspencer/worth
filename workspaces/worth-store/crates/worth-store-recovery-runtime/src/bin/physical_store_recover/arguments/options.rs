use std::path::{Path, PathBuf};

pub(super) fn parse(
    root: &Path,
    arguments: &[std::ffi::OsString],
) -> Result<
    (
        Option<PathBuf>,
        Option<worth_store::physical_runtime::PhysicalRecoveryProcessYieldpoint>,
    ),
    String,
> {
    let mut report = None;
    let mut stage = None;
    let mut reached = None;
    let mut release = None;
    let mut cancel = None;
    let mut deadline_ms = None;
    for argument in arguments {
        let argument = argument.to_string_lossy();
        if let Some(path) = argument.strip_prefix("--report=") {
            assign_path(&mut report, path, "recovery report")?;
        } else if let Some(label) = argument.strip_prefix("--yieldpoint-stage=") {
            assign_stage(&mut stage, label)?;
        } else if let Some(path) = argument.strip_prefix("--yieldpoint-reached=") {
            assign_path(&mut reached, path, "recovery yieldpoint reached marker")?;
        } else if let Some(path) = argument.strip_prefix("--yieldpoint-release=") {
            assign_path(&mut release, path, "recovery yieldpoint release marker")?;
        } else if let Some(path) = argument.strip_prefix("--yieldpoint-cancel=") {
            assign_path(&mut cancel, path, "recovery yieldpoint cancel marker")?;
        } else if let Some(value) = argument.strip_prefix("--yieldpoint-deadline-ms=") {
            if deadline_ms.is_some() {
                return Err("recovery yieldpoint deadline must be supplied exactly once".into());
            }
            deadline_ms = Some(
                value
                    .parse::<u64>()
                    .map_err(|_| "recovery yieldpoint deadline must be milliseconds".to_owned())?,
            );
        } else {
            return Err("unsupported optional argument".into());
        }
    }
    if let Some(report_path) = &report {
        reject_path_inside_store(root, report_path, "recovery report")?;
    }
    let yieldpoint = match (stage, reached, release, cancel) {
        (None, None, None, None) if deadline_ms.is_none() => None,
        (Some(stage), Some(reached), Some(release), Some(cancel)) => {
            reject_path_inside_store(root, &reached, "recovery yieldpoint reached marker")?;
            reject_path_inside_store(root, &release, "recovery yieldpoint release marker")?;
            reject_path_inside_store(root, &cancel, "recovery yieldpoint cancel marker")?;
            Some(match deadline_ms {
                Some(deadline_ms) => {
                    worth_store::physical_runtime::PhysicalRecoveryProcessYieldpoint::new_with_wait_deadline(
                        stage,
                        reached,
                        release,
                        cancel,
                        std::time::Duration::from_millis(deadline_ms),
                    )
                }
                None => worth_store::physical_runtime::PhysicalRecoveryProcessYieldpoint::new(
                    stage, reached, release, cancel,
                ),
            })
        }
        _ => {
            return Err(
                "recovery yieldpoint requires --yieldpoint-stage, --yieldpoint-reached, --yieldpoint-release, and --yieldpoint-cancel"
                    .into(),
            )
        }
    };
    Ok((report, yieldpoint))
}

fn assign_path(destination: &mut Option<PathBuf>, value: &str, role: &str) -> Result<(), String> {
    if value.is_empty() || destination.replace(PathBuf::from(value)).is_some() {
        return Err(format!("{role} must be supplied exactly once"));
    }
    Ok(())
}

fn assign_stage(
    destination: &mut Option<worth_store::physical_runtime::PhysicalRecoveryYieldpointStage>,
    label: &str,
) -> Result<(), String> {
    if destination.is_some() {
        return Err("recovery yieldpoint stage must be supplied exactly once".into());
    }
    *destination =
        worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::from_label(label);
    if destination.is_none() {
        return Err(format!("unsupported recovery yieldpoint stage: {label}"));
    }
    Ok(())
}

fn reject_path_inside_store(root: &Path, path: &Path, role: &str) -> Result<(), String> {
    let root = root
        .canonicalize()
        .map_err(|error| format!("could not resolve recovery store root: {error}"))?;
    let path = if path.exists() {
        path.canonicalize()
            .map_err(|error| format!("could not resolve {role}: {error}"))?
    } else {
        let parent = path
            .parent()
            .ok_or_else(|| format!("{role} has no parent"))?
            .canonicalize()
            .map_err(|error| format!("could not resolve {role} directory: {error}"))?;
        parent.join(
            path.file_name()
                .ok_or_else(|| format!("{role} has no file name"))?,
        )
    };
    if path == root || path.starts_with(&root) {
        return Err(format!("{role} must be outside the observed Store root"));
    }
    Ok(())
}
