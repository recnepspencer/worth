use std::path::Path;

/// Sorted names of every published record segment page artifact.
pub(super) fn segment_names(root: &Path) -> Vec<String> {
    let mut names = std::fs::read_dir(root.join("families").join("records").join("segments"))
        .unwrap()
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".pages"))
        .collect::<Vec<_>>();
    names.sort();
    names
}

/// Segment id and generation encoded in a canonical
/// `segment-{id:016x}-{generation:016x}.pages` name.
pub(super) fn segment_identity(name: &str) -> (u64, u64) {
    let body = name
        .strip_prefix("segment-")
        .and_then(|body| body.strip_suffix(".pages"))
        .expect("canonical segment page name");
    let (segment, generation) = body.split_once('-').expect("segment and generation");
    (
        u64::from_str_radix(segment, 16).unwrap(),
        u64::from_str_radix(generation, 16).unwrap(),
    )
}
