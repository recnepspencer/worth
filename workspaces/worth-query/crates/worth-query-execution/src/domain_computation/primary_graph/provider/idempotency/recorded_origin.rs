//! Co-committed receipt origin and its canonical affinity grammar.
use super::WorthQueryProductIdempotencyAffinity;
use worth_foundational::facade::BoundaryProtocolVersion;

/// The row's association reference resolves through its immutable creation
/// version to the enclosing canonical Native artifact. The actual association
/// is issued by World after registration, never predicted or copied from a key.
pub(super) const ENCLOSING_NATIVE_ASSOCIATION: u64 = 1;

pub(super) fn affinity_text(product: &WorthQueryProductIdempotencyAffinity) -> String {
    let Some(owner) = product.branch().owner_identity().durable() else {
        return "memory-only".to_owned();
    };
    let home = owner
        .home()
        .bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let branch = product.branch().name().as_str();
    format!(
        "home-affinity-v{}:{home}:{}:{}:{branch}:{}",
        BoundaryProtocolVersion::new(1),
        owner.slot().get(),
        branch.len(),
        product.incarnation().ordinal()
    )
}
