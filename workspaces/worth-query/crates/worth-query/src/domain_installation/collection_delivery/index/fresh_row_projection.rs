use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use std::collections::{BTreeMap, BTreeSet};

use super::{
    WorthQueryCollectionIndexPreview, WorthQueryCollectionMaintenanceIndex,
    WorthQueryCollectionMaintenanceTarget,
};
use crate::domain_installation::{
    WorthQueryBoundCollectionWindow, WorthQueryCollectionDeliveryCounters,
    WorthQueryNativeAccessKey,
};
use crate::memory_workspace::{WorthQueryEntity, WorthQueryEntityIdentity};

impl WorthQueryCollectionMaintenanceIndex {
    pub(in crate::domain_installation::collection_delivery) fn preview_fresh_rows(
        &self,
        window: &WorthQueryBoundCollectionWindow,
        affected: &BTreeSet<WorthQueryEntityIdentity>,
        keys: &[WorthQueryNativeAccessKey],
        replacement_targets: &[WorthQueryCollectionMaintenanceTarget],
        fresh_rows: &[WorthQueryEntity],
        counters: &mut WorthQueryCollectionDeliveryCounters,
    ) -> WorthQueryCollectionIndexPreview {
        let fresh = fresh_rows
            .iter()
            .map(|row| (row.identity(), row))
            .collect::<BTreeMap<_, _>>();
        let delta = self.delta_from(affected, counters, |identity| {
            let fresh = (*fresh.get(identity)?).clone();
            let prior = self
                .source_identities
                .get(identity)
                .and_then(|consumer| self.identities.get(consumer))
                .and_then(|ordering| self.rows.get(ordering));
            Some(prior.map_or(fresh.clone(), |prior| {
                let replacements = replacement_targets
                    .iter()
                    .map(|target| target.replacement(&fresh));
                prior.entity.clone().replace_native_values(replacements)
            }))
        });
        self.preview_delta(window, affected, keys, delta, counters)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use worth_foundational::facade::{
        AspectKey, AspectValue, CanonicalFieldPath, FieldKey, StructAspectValue,
    };

    use super::*;

    #[test]
    fn fresh_path_replacement_normalizes_present_then_absent_support_truth(
    ) -> Result<(), IdentityDerivationDenial> {
        let aspect = AspectKey::new("PortfolioFacts").unwrap();
        let rank = FieldKey::new("PortfolioRankField").unwrap();
        let path =
            CanonicalFieldPath::new([FieldKey::new("PortfolioFacts").unwrap(), rank.clone()])
                .unwrap();
        let identity = crate::memory_workspace::admit_authored_entity_label("support-row")?;
        let prior = row(identity.clone(), &aspect, &rank, &path, Some(1));
        let fresh = row(identity.clone(), &aspect, &rank, &path, Some(2));
        let target = WorthQueryCollectionMaintenanceTarget::new(
            aspect.clone(),
            CanonicalFieldPath::new([rank.clone()]).unwrap(),
        );
        let updated = prior.replace_native_values([target.replacement(&fresh)]);
        assert_eq!(
            updated
                .struct_aspect_value(&aspect)
                .and_then(|value| value.get(&rank)),
            Some(&AspectValue::UInt64(2))
        );
        assert_eq!(
            updated.scalar_value_at(&path),
            Some(&AspectValue::UInt64(2))
        );

        let absent = row(identity, &aspect, &rank, &path, None);
        let cleared = updated.replace_native_values([target.replacement(&absent)]);
        assert_eq!(cleared.struct_aspect_value(&aspect), None);
        assert_eq!(cleared.aspect_value(&aspect), None);
        assert_eq!(cleared.scalar_value_at(&path), None);

        Ok(())
    }

    #[test]
    fn namespaced_aspect_replacement_updates_exact_native_owner(
    ) -> Result<(), IdentityDerivationDenial> {
        let aspect = AspectKey::new("Portfolio.Facts").unwrap();
        let rank = FieldKey::new("PortfolioRankField").unwrap();
        let path = CanonicalFieldPath::new([
            FieldKey::new("Portfolio").unwrap(),
            FieldKey::new("Facts").unwrap(),
            rank.clone(),
        ])
        .unwrap();
        let identity = crate::memory_workspace::admit_authored_entity_label("namespaced-row")?;
        let prior = row(identity.clone(), &aspect, &rank, &path, Some(1));
        let fresh = row(identity, &aspect, &rank, &path, Some(9));
        let target = WorthQueryCollectionMaintenanceTarget::new(
            aspect.clone(),
            CanonicalFieldPath::new([rank.clone()]).unwrap(),
        );
        let updated = prior.replace_native_values([target.replacement(&fresh)]);
        assert_eq!(
            updated
                .struct_aspect_value(&aspect)
                .and_then(|value| value.get(&rank)),
            Some(&AspectValue::UInt64(9))
        );
        assert_eq!(
            updated.scalar_value_at(&path),
            Some(&AspectValue::UInt64(9))
        );
        assert_eq!(
            updated.struct_aspect_value(&AspectKey::new("Portfolio").unwrap()),
            None
        );

        Ok(())
    }

    fn row(
        identity: WorthQueryEntityIdentity,
        aspect: &AspectKey,
        field: &FieldKey,
        path: &CanonicalFieldPath,
        value: Option<u64>,
    ) -> WorthQueryEntity {
        let scalar = value.map(AspectValue::UInt64);
        WorthQueryEntity::from_aspect_projection(
            identity,
            scalar
                .clone()
                .map(|value| BTreeMap::from([(aspect.clone(), value)]))
                .unwrap_or_default(),
            scalar
                .clone()
                .map(|value| {
                    BTreeMap::from([(
                        aspect.clone(),
                        StructAspectValue::new([(field.clone(), value)]).unwrap(),
                    )])
                })
                .unwrap_or_default(),
            scalar
                .map(|value| BTreeMap::from([(path.clone(), value)]))
                .unwrap_or_default(),
        )
    }
}
