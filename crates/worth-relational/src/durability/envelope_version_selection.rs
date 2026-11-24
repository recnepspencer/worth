pub(crate) fn descriptor_semantics_version_for_envelopes(
    checkpoint_envelopes: &[crate::history::data::PositionedCanonicalCommit],
    tail_log: &[crate::durability::migration::ReadmittedCanonicalCommit],
) -> crate::schema::data::DescriptorSemanticsVersion {
    tail_log
        .last()
        .map(|entry| entry.envelope().descriptor_semantics_version)
        .or_else(|| {
            checkpoint_envelopes
                .last()
                .map(|entry| entry.envelope().descriptor_semantics_version)
        })
        .unwrap_or_default()
}
