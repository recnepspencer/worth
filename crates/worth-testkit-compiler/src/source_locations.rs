//! Source identity comes from the fixture's complete structural owner path.
use std::path::Path;

use crate::paths::source_key;

pub(crate) struct Location {
    pub(crate) prefix: String,
    pub(crate) path: String,
    pub(crate) coordinates: String,
    pub(crate) external: bool,
    pub(crate) known: bool,
}

pub(crate) struct SourceContext {
    source: String,
    fixtures: Option<String>,
    relative_fixtures: Option<String>,
    owner: Option<String>,
    workspace: Option<String>,
}

impl SourceContext {
    pub(crate) fn new(source: &Path) -> Self {
        let source = source_key(source);
        let mut fixtures = None;
        let mut relative_fixtures = None;
        let mut owner_name = None;
        for (offset, _) in source.match_indices("/tests/") {
            let tail = &source[offset + 7..];
            if let Some((_, tail)) = tail.split_once('/') {
                if let Some(tail) = tail.strip_prefix("sources/") {
                    if let Some((owner, _)) = tail.split_once('/') {
                        let length = source.len() - tail.len() + owner.len() + 1;
                        fixtures = Some(source[..length].to_owned());
                        relative_fixtures = Some(source[offset + 1..length].to_owned());
                        owner_name = Some(owner.replace('_', "-"));
                        break;
                    }
                }
            }
        }
        if fixtures.is_none() {
            if let Some((prefix, _)) = source.split_once("/assets/") {
                if let Some((_, name)) = prefix.rsplit_once("/crates/worth-testkit-") {
                    if !name.contains('/') {
                        fixtures = Some(format!("{prefix}/assets/"));
                        owner_name = Some(format!("worth-{name}"));
                    }
                }
            }
        }
        let workspace = source
            .rsplit_once("/crates/")
            .map(|(prefix, _)| format!("{prefix}/"));
        let owner = workspace
            .as_ref()
            .zip(owner_name)
            .map(|(workspace, name)| format!("{workspace}crates/{name}/"));
        Self {
            source,
            fixtures,
            relative_fixtures,
            owner,
            workspace,
        }
    }

    pub(crate) fn location(&self, line: &str) -> Option<Location> {
        let leading = line.len() - line.trim_start().len();
        let trimmed = &line[leading..];
        if !trimmed.starts_with("--> ") && !trimmed.starts_with("::: ") {
            return None;
        }
        let start = leading + 4;
        let (path, coordinates) = split_coordinates(&line[start..]);
        let path = self.path(path);
        let fixture = path.starts_with("tests/") || path == self.path(&self.source);
        let external = path.starts_with("src/")
            || ["$WORKSPACE/", "$RUST/", "$CARGO/"]
                .iter()
                .any(|prefix| path.starts_with(prefix));
        Some(Location {
            prefix: line[..start].to_owned(),
            path,
            coordinates: coordinates.to_owned(),
            external,
            known: fixture || external,
        })
    }

    fn path(&self, path: &str) -> String {
        let symbolic = path.replace('\\', "/");
        if let Some((workspace, relative)) = self
            .workspace
            .as_ref()
            .zip(symbolic.strip_prefix("$WORKSPACE/"))
        {
            return self.path(&format!("{workspace}{relative}"));
        }
        for marker in ["$WORKSPACE/", "$RUST/", "$CARGO/"] {
            if let Some(relative) = symbolic.strip_prefix(marker) {
                return format!("{marker}{relative}");
            }
        }
        let key = source_key(Path::new(path));
        if let Some(relative) = self
            .relative_fixtures
            .as_ref()
            .and_then(|root| key.strip_prefix(root))
        {
            return format!("tests/{relative}");
        }
        if let Some(relative) = self
            .fixtures
            .as_ref()
            .and_then(|root| key.strip_prefix(root))
        {
            return format!("tests/{relative}");
        }
        if let Some(relative) = self.owner.as_ref().and_then(|root| key.strip_prefix(root)) {
            return relative.to_owned();
        }
        if let Some(relative) = self
            .workspace
            .as_ref()
            .and_then(|root| key.strip_prefix(root))
        {
            return format!("$WORKSPACE/{relative}");
        }
        if let Some(relative) = rust_library(&key) {
            return format!("$RUST/{relative}");
        }
        if let Some((_, tail)) = key.split_once("/registry/src/") {
            if let Some((registry, relative)) = tail.split_once('/') {
                if let Some((_, hash)) = registry.rsplit_once('-') {
                    if hex(hash, 16) {
                        return format!("$CARGO/{relative}");
                    }
                }
            }
        }
        key
    }
}

fn split_coordinates(value: &str) -> (&str, &str) {
    let mut parts = value.rsplitn(3, ':');
    if let (Some(column), Some(line), Some(path)) = (parts.next(), parts.next(), parts.next()) {
        if !column.is_empty()
            && !line.is_empty()
            && column.bytes().all(|byte| byte.is_ascii_digit())
            && line.bytes().all(|byte| byte.is_ascii_digit())
        {
            return (path, &value[path.len()..]);
        }
    }
    (value, "")
}

fn rust_library(key: &str) -> Option<&str> {
    if let Some((_, relative)) = key.split_once("/rustlib/src/rust/library/") {
        return Some(relative);
    }
    if let Some((_, relative)) = key.split_once("/rustlib/src/rust/src/") {
        return Some(relative);
    }
    let tail = key.strip_prefix("/rustc/")?;
    let (hash, relative) = tail.split_once("/library/")?;
    hex(hash, 40).then_some(relative)
}

fn hex(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub(crate) fn normalize(text: &str, source: &Path) -> String {
    let context = SourceContext::new(source);
    text.replace("\r\n", "\n")
        .lines()
        .map(|line| match context.location(line) {
            Some(location) => format!(
                "{}{}{}",
                location.prefix, location.path, location.coordinates
            ),
            None => line.to_owned(),
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end_matches('\n')
        .to_owned()
}
