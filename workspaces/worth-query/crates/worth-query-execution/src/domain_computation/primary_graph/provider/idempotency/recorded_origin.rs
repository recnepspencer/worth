//! Co-committed receipt origin and its canonical affinity grammar.
use super::WorthQueryProductIdempotencyAffinity;
use worth_foundational::facade::BoundaryProtocolVersion;

/// The row's association reference resolves through its immutable creation
/// version to the enclosing canonical Native artifact. The actual association
/// is issued by World after registration, never predicted or copied from a key.
pub(super) const ENCLOSING_NATIVE_ASSOCIATION: u64 = 1;
const MEMORY_AFFINITY: &str = "memory-only";

pub(in crate::domain_computation::primary_graph::provider) fn affinity_text(
    product: &WorthQueryProductIdempotencyAffinity,
) -> String {
    let Some(owner) = product.branch().owner_identity().durable() else {
        return MEMORY_AFFINITY.to_owned();
    };
    home_affinity_text(
        owner,
        product.branch().name().as_str(),
        product.incarnation().ordinal(),
    )
}

pub(in crate::domain_computation::primary_graph::provider) fn original_affinity_text(
    source: &super::super::OriginalPublication,
) -> String {
    match source {
        super::super::OriginalPublication::Memory(_) => MEMORY_AFFINITY.to_owned(),
        super::super::OriginalPublication::Home(source) => {
            let after = source.after();
            let owner = after
                .branch_identity()
                .owner_identity()
                .durable()
                .expect("Home source");
            home_affinity_text(
                owner,
                after.branch_identity().name().as_str(),
                after.lifecycle_incarnation().ordinal(),
            )
        }
    }
}

fn home_affinity_text(
    owner: worth_foundational::facade::DurableWorldOwnerIdentity,
    branch: &str,
    incarnation: u64,
) -> String {
    let home = owner
        .home()
        .bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!(
        "home-affinity-v{}:{home}:{}:{}:{branch}:{}",
        BoundaryProtocolVersion::new(1),
        owner.slot().get(),
        branch.len(),
        incarnation
    )
}
