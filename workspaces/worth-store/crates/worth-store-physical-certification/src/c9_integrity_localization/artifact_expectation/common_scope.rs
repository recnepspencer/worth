//! Report addresses are checked against immutable producer coordinates, not verdicts.
use super::{ArtifactGranule, Operator as Op};
use serde_json::{json, Value};
use std::path::Path;
use worth_store_physical_format::{PhysicalArtifactReadTarget, RecordArtifactFile as File};

pub(in crate::c9_integrity_localization) fn require(
    row: &Value,
    target: &ArtifactGranule,
    baseline: &Path,
    operator: Option<Op>,
    runtime: bool,
) {
    let PhysicalArtifactReadTarget::Record(file) = target.target else {
        return;
    };
    let family = match target.family {
        "inline_page" => "page_frame",
        "extent_chunk" => "extent_chunk_frame",
        family => family,
    };
    assert_eq!(row["family"], family, "{row}");
    assert_eq!(
        row["path"],
        target.path.to_string_lossy().replace('\\', "/")
    );
    let (identity, generation) = match file {
        File::BootstrapCatalog => ("bootstrap-catalog".to_owned(), None),
        File::CurrentRootSelector | File::PreviousRootSelector => {
            let identity_admitted = operator.is_none()
                || operator == Some(Op::Duplicate)
                || (!runtime && operator == Some(Op::ScopeSubstitution));
            let identity = if identity_admitted {
                let bytes = std::fs::read(baseline.join(&target.path)).unwrap();
                format!(
                    "selector:{:016x}",
                    u64::from_le_bytes(bytes[28..36].try_into().unwrap())
                )
            } else if file == File::CurrentRootSelector {
                "current-selector".to_owned()
            } else {
                "previous-selector".to_owned()
            };
            (identity, None)
        }
        File::RootManifest { generation } => (format!("root:{generation:016x}"), Some(generation)),
        File::RootRoutingBlock { generation, block }
        | File::SegmentMembershipBlock { generation, block }
        | File::FreeSpaceMembershipBlock { generation, block } => {
            (format!("block:{block:016x}"), Some(generation))
        }
        File::FreeSpaceManifest { generation } => {
            (format!("free-space:{generation:016x}"), Some(generation))
        }
        File::Segment { segment, .. } => {
            let page = target.scope.page_identity().unwrap();
            (
                format!("page:{segment:016x}:{:016x}", page.page_id().get()),
                Some(page.generation().get()),
            )
        }
        File::ExtentManifest { extent, generation } => {
            (format!("extent:{extent:016x}"), Some(generation))
        }
        File::Extent { extent, generation } => (
            format!(
                "extent:{extent:016x}:chunk:{}",
                target.scope.extent_chunk_coordinate().unwrap().ordinal()
            ),
            Some(generation),
        ),
        _ => unreachable!("inventory contains only production-issued artifact shapes"),
    };
    assert_eq!(row["identity"], identity, "{row}");
    assert_eq!(row["generation"], json!(generation), "{row}");
    if !runtime
        && operator == Some(Op::Remove)
        && matches!(
            file,
            File::RootRoutingBlock { .. }
                | File::SegmentMembershipBlock { .. }
                | File::FreeSpaceMembershipBlock { .. }
        )
    {
        assert!(
            row["range"].is_null(),
            "a missing variable-width tree has no acquired byte range: {row}"
        );
        return;
    }
    // A tree's parent carries a bounded variable-width reference, not a frame
    // length. When truncated, its observed envelope range is the surviving
    // prefix; the independent reader localizes the missing suffix separately.
    let length = if !runtime
        && operator == Some(Op::Truncate)
        && matches!(
            file,
            File::RootRoutingBlock { .. }
                | File::SegmentMembershipBlock { .. }
                | File::FreeSpaceMembershipBlock { .. }
        ) {
        target.length() / 2
    } else {
        target.length()
    };
    assert_eq!(
        row["range"],
        json!({"offset":target.offset(), "length":length}),
        "{row}"
    );
}
