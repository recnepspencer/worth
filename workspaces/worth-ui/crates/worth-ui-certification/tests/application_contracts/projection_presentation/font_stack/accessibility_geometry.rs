use sha2::{Digest, Sha256};
use worth_ui_host_contract::{
    UiQualifiedTextCaretRecord, UiQualifiedTextLineRecord, UiQualifiedTextVisualRunRecord,
    UiTextCaretAffinity, UiTextVisualEdge,
};
use worth_ui_host_headless::UiHeadlessTextAccessibilityGeometry;

const EXPECTED_RECORD_DIGEST: &str =
    "3b2286509caaad182fac1f6d6ae2c55a215bc3fdb26f9388ed153416077aa414";

pub(super) fn assert_exact_multiline_bidi_records(
    geometry: &UiHeadlessTextAccessibilityGeometry<'_>,
) {
    assert_eq!(
        geometry.lines().len(),
        2,
        "hard break must retain two lines"
    );
    assert_eq!(
        geometry.visual_runs().len(),
        6,
        "application faces, bidi, and emoji retain six visual runs"
    );
    assert_eq!(
        geometry
            .visual_runs()
            .iter()
            .map(|run| run.bidi_level())
            .collect::<Vec<_>>(),
        [0, 0, 0, 1, 1, 1]
    );
    assert_eq!(geometry.carets().len(), 26);
    assert_runs_tile_their_lines(geometry);

    let observed = record_digest(geometry.lines(), geometry.visual_runs(), geometry.carets());
    assert_eq!(hex(observed), EXPECTED_RECORD_DIGEST);
    assert_ne!(
        record_digest(&[], geometry.visual_runs(), geometry.carets()),
        observed,
        "empty accessibility lines must be rejected"
    );
    assert_ne!(
        record_digest(geometry.lines(), &[], geometry.carets()),
        observed,
        "empty accessibility visual runs must be rejected"
    );
    assert_ne!(
        record_digest(geometry.lines(), geometry.visual_runs(), &[]),
        observed,
        "empty accessibility carets must be rejected"
    );
    let mut reversed = geometry.carets().to_vec();
    reversed.reverse();
    assert_ne!(
        record_digest(geometry.lines(), geometry.visual_runs(), &reversed),
        observed,
        "logical-order or affinity-corrupted carets must be rejected"
    );
}

/// Every line is covered by its own runs with nothing left over. A hard break
/// paints nothing, but it is part of the line it ends and an authored span may
/// style it, so the run that ends the line reaches it; a byte of a line that no
/// run carries is a byte an assistive reader cannot name.
fn assert_runs_tile_their_lines(geometry: &UiHeadlessTextAccessibilityGeometry<'_>) {
    for (index, line) in geometry.lines().iter().enumerate() {
        let range = line.visual_run_range();
        let mut spans = geometry.visual_runs()[range.start as usize..range.end as usize]
            .iter()
            .map(|run| (run.original_range().start(), run.original_range().end()))
            .collect::<Vec<_>>();
        spans.sort_unstable();
        let mut covered = line.original_range().start();
        for (start, end) in spans {
            assert_eq!(start, covered, "line {index} leaves a gap between its runs");
            covered = end;
        }
        assert_eq!(
            covered,
            line.original_range().end(),
            "line {index} spans further than its runs carry"
        );
    }
}

fn record_digest(
    lines: &[UiQualifiedTextLineRecord],
    runs: &[UiQualifiedTextVisualRunRecord],
    carets: &[UiQualifiedTextCaretRecord],
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"worth-ui-accessibility-geometry-v1\0");
    hash_len(&mut hash, lines.len());
    for line in lines {
        hash_range(&mut hash, line.original_range());
        let visual = line.visual_run_range();
        hash.update(visual.start.to_be_bytes());
        hash.update(visual.end.to_be_bytes());
        hash_rect(&mut hash, line.logical_bounds());
        hash_rect(&mut hash, line.ink_bounds());
        hash.update(line.baseline_millipoints().to_be_bytes());
        hash.update([u8::from(line.hard_break()), u8::from(line.overflowed())]);
    }
    hash_len(&mut hash, runs.len());
    for run in runs {
        hash_range(&mut hash, run.original_range());
        hash.update(run.line_index().to_be_bytes());
        let logical = run.logical_run_range();
        hash.update(logical.start.to_be_bytes());
        hash.update(logical.end.to_be_bytes());
        hash.update([run.bidi_level()]);
        hash_rect(&mut hash, run.bounds());
    }
    hash_len(&mut hash, carets.len());
    for caret in carets {
        let position = caret.position();
        hash_range(&mut hash, position.original_boundary());
        hash.update([match position.visual_edge() {
            UiTextVisualEdge::Leading => 0,
            UiTextVisualEdge::Trailing => 1,
        }]);
        hash.update([match position.affinity() {
            UiTextCaretAffinity::Upstream => 0,
            UiTextCaretAffinity::Downstream => 1,
        }]);
        hash.update(caret.line_index().to_be_bytes());
        hash.update(caret.visual_run_index().to_be_bytes());
        hash.update(caret.x_millipoints().to_be_bytes());
        hash.update(caret.top_millipoints().to_be_bytes());
        hash.update(caret.bottom_millipoints().to_be_bytes());
    }
    hash.finalize().into()
}

fn hash_len(hash: &mut Sha256, len: usize) {
    hash.update(
        u64::try_from(len)
            .expect("qualified record count fits u64")
            .to_be_bytes(),
    );
}

fn hash_range(hash: &mut Sha256, range: worth_ui_host_contract::UiTextOriginalRange) {
    hash.update(range.start().to_be_bytes());
    hash.update(range.end().to_be_bytes());
}

fn hash_rect(hash: &mut Sha256, rect: worth_ui_host_contract::UiTextRect) {
    hash.update(rect.left_millipoints().to_be_bytes());
    hash.update(rect.top_millipoints().to_be_bytes());
    hash.update(rect.right_millipoints().to_be_bytes());
    hash.update(rect.bottom_millipoints().to_be_bytes());
}

fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
