use super::UiSelectedTextCluster;

/// Neutral clusters use their surrounding shaping context. A space does not
/// create a new script run, but face, style, bidi and control boundaries do.
pub(super) fn resolve(clusters: &mut [UiSelectedTextCluster]) {
    let mut start = 0;
    while start < clusters.len() {
        let first = clusters[start];
        let mut end = start + 1;
        while end < clusters.len()
            && first.face_slot.is_some()
            && first.coverage == super::UiTextCoverageDisposition::QualifiedFace
            && clusters[end].coverage == first.coverage
            && clusters[end].face_slot == first.face_slot
            && clusters[end].style_index == first.style_index
            && clusters[end].bidi_level == first.bidi_level
            && !first.rgi_emoji
            && !clusters[end].rgi_emoji
        {
            end += 1;
        }
        let run = &mut clusters[start..end];
        let mut context = run
            .iter()
            .find(|cluster| !neutral(cluster.script_tag))
            .map(|cluster| cluster.script_tag);
        for cluster in run {
            if neutral(cluster.script_tag) {
                if let Some(script) = context {
                    cluster.script_tag = script;
                }
            } else {
                context = Some(cluster.script_tag);
            }
        }
        start = end;
    }
}

fn neutral(script: [u8; 4]) -> bool {
    script == *b"Zyyy" || script == *b"Zinh" || script == *b"Zzzz"
}
