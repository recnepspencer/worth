//! Lower typed OneOf membership to the same Native keys as its case records.
use std::collections::{BTreeMap, BTreeSet};
use worth_foundational::facade::{AspectExclusionGroupSpec, AspectKey, CanonicalDigestId};
use worth_query_declaration::facade::application_schema::fragment::FragmentCatalog;
pub(super) type WorthQueryInstalledAspectExclusionGroup =
    AspectExclusionGroupSpec<BTreeSet<AspectKey>>;

pub(super) fn lower(
    fragments: &FragmentCatalog,
) -> BTreeMap<String, Vec<WorthQueryInstalledAspectExclusionGroup>> {
    let mut by_entity = BTreeMap::<_, Vec<_>>::new();
    for (entity, contract_digest, required, cases) in fragments.exclusive_groups() {
        let aspects = cases
            .map(|key| AspectKey::new(key).expect("encoded Native aspect key"))
            .collect();
        by_entity.entry(entity.to_owned()).or_default().push(
            WorthQueryInstalledAspectExclusionGroup {
                // Rust marker names and case tuple order are process-local.
                group_id: CanonicalDigestId::new(*contract_digest).render_hex(),
                aspects,
                required,
            },
        );
    }
    for groups in by_entity.values_mut() {
        groups.sort_by(|left, right| left.group_id.cmp(&right.group_id));
    }
    by_entity
}
