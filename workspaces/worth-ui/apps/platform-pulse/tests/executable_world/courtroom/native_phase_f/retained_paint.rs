pub(super) fn assert_foreground_invariant_intrinsic_keys(
    evidence: &serde_json::Value,
    keys: &std::collections::BTreeSet<String>,
) {
    let frames = evidence["retained_frame_intrinsic_glyphs"]
        .as_array()
        .unwrap();
    let mut foregrounds = std::collections::BTreeSet::new();
    let mut appearances = std::collections::BTreeMap::<String, usize>::new();
    for glyph in frames
        .iter()
        .flat_map(|frame| frame["glyphs"].as_array().unwrap())
    {
        let key = glyph["raster_key"].as_str().unwrap().to_owned();
        if keys.contains(&key) {
            foregrounds.insert(glyph["foreground"].to_string());
            *appearances.entry(key).or_default() += 1;
        }
    }
    assert!(
        foregrounds.contains("[216,232,255,255]"),
        "retained intrinsic foregrounds={foregrounds:?}; frames={frames:?}"
    );
    assert!(
        foregrounds.contains("[255,255,255,255]"),
        "retained intrinsic foregrounds={foregrounds:?}; frames={frames:?}"
    );
    assert!(keys
        .iter()
        .all(|key| appearances.get(key).copied().unwrap_or_default() >= 2));
    let pin_frames = evidence["text_pin_frames"].as_array().unwrap();
    assert!(keys.iter().all(|key| pin_frames
        .iter()
        .filter(|frame| {
            frame
                .as_array()
                .unwrap()
                .iter()
                .any(|pin| pin["raster_key"].as_str() == Some(key.as_str()))
        })
        .count()
        >= 2));
}
