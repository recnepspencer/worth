pub fn descriptor_semantics_version_for_envelopes(
    checkpoint_envelopes: &[worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit],
    tail_log: &[crate::durability::migration::ReadmittedCanonicalCommit],
) -> worth_relational_contracts::internal_port::schema::data::continuity::descriptor_versions::DescriptorSemanticsVersion{
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
