use std::collections::BTreeMap;

use crate::transactions::data::AspectEmissionTrace;

#[derive(Debug, Clone)]
pub(in crate::authority::commit::pipeline) struct PreparedAspectEmissionTrace {
    target: crate::transactions::data::RecordRef,
    patch_record_index: u64,
    structural_change: crate::publication::patch::data::RecordStructuralChange,
    changed_aspects: Vec<worth_foundational::facade::AspectKey>,
    contains_opaque_aspect: bool,
}

impl PreparedAspectEmissionTrace {
    pub(in crate::authority::commit::pipeline) fn publish(
        self,
        patch_position: crate::publication::patch::data::PatchStreamPosition,
    ) -> AspectEmissionTrace {
        AspectEmissionTrace {
            target: self.target,
            patch_position,
            patch_record_index: self.patch_record_index,
            structural_change: self.structural_change,
            changed_aspects: self.changed_aspects,
            contains_opaque_aspect: self.contains_opaque_aspect,
        }
    }
}

pub(super) fn derive_aspect_emission_traces(
    patch_records: &[crate::publication::patch::data::PublishedAuthoritativeRecordPatch],
    deltas: &[crate::authority::mutation::CanonicalRecordAspectDelta],
) -> Vec<PreparedAspectEmissionTrace> {
    let delta_index = deltas
        .iter()
        .map(|delta| (delta.target.clone(), delta))
        .collect::<BTreeMap<_, _>>();
    patch_records
        .iter()
        .enumerate()
        .map(|(patch_record_index, record)| {
            let delta = delta_index.get(&record.target).copied().unwrap_or_else(|| {
                panic!(
                    "missing canonical aspect delta for emitted patch target {:?}",
                    record.target
                )
            });
            PreparedAspectEmissionTrace {
                target: delta.target.clone(),
                patch_record_index: patch_record_index as u64,
                structural_change: delta.structural_change,
                changed_aspects: delta.changed_aspects.clone(),
                contains_opaque_aspect: delta.contains_opaque_aspect,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::derive_aspect_emission_traces;
    use crate::authority::mutation::CanonicalRecordAspectDelta;
    use crate::identity::data::{EntityId, KindId, PartitionId};
    use crate::publication::patch::data::{
        ordered_aspect_keys, PatchDetail, PatchStreamPosition, PublishedAuthoritativeRecordPatch,
        RecordStructuralChange,
    };
    use crate::schema::data::AspectContractPlanRevision;
    use crate::transactions::data::RecordRef;
    use worth_foundational::facade::AspectKey;

    #[test]
    fn aspect_emission_traces_use_indexed_target_lookup() {
        let target_a = RecordRef::Entity(EntityId::new(PartitionId(3), 1, 1));
        let target_b = RecordRef::Entity(EntityId::new(PartitionId(3), 2, 1));
        let aspect_a = AspectKey::new("a").unwrap();
        let aspect_b = AspectKey::new("b").unwrap();
        let deltas = vec![
            CanonicalRecordAspectDelta {
                target: target_a.clone(),
                kind_id: KindId(7),
                plan_revision: AspectContractPlanRevision(1),
                structural_change: RecordStructuralChange::Updated,
                changed_aspects: ordered_aspect_keys([aspect_a.clone()]),
                evaluated_bindings: Default::default(),
                contains_opaque_aspect: false,
            },
            CanonicalRecordAspectDelta {
                target: target_b.clone(),
                kind_id: KindId(7),
                plan_revision: AspectContractPlanRevision(1),
                structural_change: RecordStructuralChange::Created,
                changed_aspects: ordered_aspect_keys([aspect_b.clone()]),
                evaluated_bindings: Default::default(),
                contains_opaque_aspect: true,
            },
        ];
        let patch_records = vec![
            PublishedAuthoritativeRecordPatch {
                target: target_b.clone(),
                structural_change: RecordStructuralChange::Created,
                relation_birth: None,
                authoritative_patch:
                    crate::publication::patch::data::PublishedAuthoritativePatch::empty(),
                semantic_changes: Vec::new(),
                contains_opaque_aspect: true,
                detail: PatchDetail::DenseBitset(Vec::new()),
            },
            PublishedAuthoritativeRecordPatch {
                target: target_a.clone(),
                structural_change: RecordStructuralChange::Updated,
                relation_birth: None,
                authoritative_patch:
                    crate::publication::patch::data::PublishedAuthoritativePatch::empty(),
                semantic_changes: Vec::new(),
                contains_opaque_aspect: false,
                detail: PatchDetail::DenseBitset(Vec::new()),
            },
        ];

        let traces = derive_aspect_emission_traces(&patch_records, &deltas)
            .into_iter()
            .map(|trace| trace.publish(PatchStreamPosition(9)))
            .collect::<Vec<_>>();
        assert_eq!(traces.len(), 2);
        assert_eq!(traces[0].target, target_b);
        assert_eq!(traces[0].changed_aspects, ordered_aspect_keys([aspect_b]));
        assert!(traces[0].contains_opaque_aspect);
        assert_eq!(traces[1].target, target_a);
        assert_eq!(traces[1].changed_aspects, ordered_aspect_keys([aspect_a]));
        assert!(!traces[1].contains_opaque_aspect);
    }
}
