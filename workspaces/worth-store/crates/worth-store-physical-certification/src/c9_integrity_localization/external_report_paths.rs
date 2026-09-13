use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExternalReportPaths {
    #[cfg(test)]
    runtime: PathBuf,
    #[cfg(test)]
    offline: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExternalReportPath(PathBuf);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExternalReportPathDenial {
    InsideStoreRoot,
    ParentTraversal,
    PathResolution,
    #[cfg(test)]
    ReusedPath,
}

impl ExternalReportPaths {
    #[cfg(test)]
    pub(crate) fn new(
        store_root: &Path,
        runtime: PathBuf,
        offline: PathBuf,
    ) -> Result<Self, ExternalReportPathDenial> {
        if [&runtime, &offline]
            .into_iter()
            .any(|path| path.components().any(|part| part == Component::ParentDir))
        {
            return Err(ExternalReportPathDenial::ParentTraversal);
        }
        let store_root = resolve_prospective_path(store_root)?;
        let runtime = Self::authorize_resolved(&store_root, runtime)?.0;
        let offline = Self::authorize_resolved(&store_root, offline)?.0;
        if runtime == offline {
            return Err(ExternalReportPathDenial::ReusedPath);
        }
        Ok(Self { runtime, offline })
    }

    pub(crate) fn authorize(
        store_root: &Path,
        report: PathBuf,
    ) -> Result<ExternalReportPath, ExternalReportPathDenial> {
        if report
            .components()
            .any(|component| component == Component::ParentDir)
        {
            return Err(ExternalReportPathDenial::ParentTraversal);
        }
        let store_root = resolve_prospective_path(store_root)?;
        Self::authorize_resolved(&store_root, report)
    }

    fn authorize_resolved(
        resolved_store_root: &Path,
        report: PathBuf,
    ) -> Result<ExternalReportPath, ExternalReportPathDenial> {
        let report = resolve_prospective_path(&report)?;
        if report.starts_with(resolved_store_root) {
            return Err(ExternalReportPathDenial::InsideStoreRoot);
        }
        Ok(ExternalReportPath(report))
    }

    #[cfg(test)]
    pub(crate) fn runtime(&self) -> &Path {
        &self.runtime
    }

    #[cfg(test)]
    pub(crate) fn offline(&self) -> &Path {
        &self.offline
    }
}

impl ExternalReportPath {
    pub(crate) fn as_path(&self) -> &Path {
        &self.0
    }
}

fn absolute(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .expect("test orchestration requires a working directory")
            .join(path)
    }
}

pub(super) fn resolve_prospective_path(path: &Path) -> Result<PathBuf, ExternalReportPathDenial> {
    let absolute = absolute(path);
    let mut ancestor = absolute.as_path();
    let mut missing: Vec<OsString> = Vec::new();
    while !ancestor.exists() {
        let name = ancestor
            .file_name()
            .ok_or(ExternalReportPathDenial::PathResolution)?;
        missing.push(name.to_os_string());
        ancestor = ancestor
            .parent()
            .ok_or(ExternalReportPathDenial::PathResolution)?;
    }
    let mut resolved =
        std::fs::canonicalize(ancestor).map_err(|_| ExternalReportPathDenial::PathResolution)?;
    for component in missing.into_iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}
