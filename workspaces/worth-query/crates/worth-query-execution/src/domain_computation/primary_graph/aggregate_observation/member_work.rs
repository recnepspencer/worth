//! Semantic coordinate work is paid by the existing caller's original poll.
use worth_relational::facade::schema::NativeMemberKey;

/// Four linear passes cover the actual new full-coordinate comparison,
/// formatting/copy into a locator, and hashing of that emitted representation.
/// A comparison-only site conservatively pays the same bound. Fixed Entity
/// coordinates retain their existing per-member work; no name-byte surcharge.
pub(in crate::domain_computation::primary_graph) fn checkpoint_names<E>(
    member: &NativeMemberKey,
    poll: &mut impl FnMut() -> Result<(), E>,
) -> Result<(), E> {
    let NativeMemberKey::Projected(member) = member else {
        return Ok(());
    };
    for text in [
        member.domain.schema.0.as_str(),
        member.domain.name.as_str(),
        member.case.as_str(),
    ] {
        for _ in text.bytes() {
            for _ in 0..4 {
                poll()?;
            }
        }
    }
    Ok(())
}
