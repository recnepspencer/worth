use std::collections::BTreeMap;
use std::sync::{Arc, PoisonError, RwLock};

use super::{
    BridgeConditionalDenial, BridgeConditionalDenialKind, BridgeInstalledConditionalLowering,
};
use crate::correspondence::{
    BridgeSemanticDependencyCandidate, InstalledCorrespondenceTarget, ProvenCorrespondenceTargets,
};

#[derive(Default)]
pub(super) struct BridgeOwnedConditionalTargetIndex {
    by_dependency: BTreeMap<String, BridgeOwnedConditionalTargetBucket>,
}

struct BridgeOwnedConditionalTargetBucket {
    dependency: BridgeSemanticDependencyCandidate,
    targets: BTreeMap<InstalledCorrespondenceTarget, usize>,
}

pub(super) struct BridgeOwnedConditionalTargetReservation {
    index: Arc<RwLock<BridgeOwnedConditionalTargetIndex>>,
    correspondences: Arc<[crate::correspondence::BridgeInstalledSemanticCorrespondence]>,
    active: bool,
}

impl BridgeOwnedConditionalTargetIndex {
    pub(super) fn register(&mut self, lowering: &BridgeInstalledConditionalLowering) {
        for correspondence in lowering.correspondences.iter() {
            let dependency = correspondence.dependency();
            let key = dependency.canonical_registration_key();
            let bucket = self.by_dependency.entry(key).or_insert_with(|| {
                BridgeOwnedConditionalTargetBucket {
                    dependency: dependency.clone(),
                    targets: BTreeMap::new(),
                }
            });
            assert_eq!(
                bucket.dependency, *dependency,
                "canonical semantic dependency identity must retain one exact meaning",
            );
            for target in correspondence.targets.as_slice() {
                *bucket.targets.entry(target.clone()).or_default() += 1;
            }
        }
    }

    pub(super) fn unregister(&mut self, lowering: &BridgeInstalledConditionalLowering) {
        self.unregister_correspondences(&lowering.correspondences);
    }

    fn unregister_correspondences(
        &mut self,
        correspondences: &[crate::correspondence::BridgeInstalledSemanticCorrespondence],
    ) {
        let mut empty = Vec::new();
        for correspondence in correspondences {
            let dependency = correspondence.dependency();
            let key = dependency.canonical_registration_key();
            let bucket = self
                .by_dependency
                .get_mut(&key)
                .expect("installed owned lowering remains indexed until retirement");
            assert_eq!(bucket.dependency, *dependency);
            for target in correspondence.targets.as_slice() {
                let references = bucket
                    .targets
                    .get_mut(target)
                    .expect("installed owned target remains indexed until retirement");
                if *references == 1 {
                    bucket.targets.remove(target);
                } else {
                    *references -= 1;
                }
            }
            if bucket.targets.is_empty() {
                empty.push(key);
            }
        }
        for key in empty {
            self.by_dependency.remove(&key);
        }
    }

    pub(super) fn reserve_existing(
        index: &Arc<RwLock<Self>>,
        correspondences: Arc<[crate::correspondence::BridgeInstalledSemanticCorrespondence]>,
    ) -> Result<BridgeOwnedConditionalTargetReservation, BridgeConditionalDenial> {
        let mut installed = index.write().unwrap_or_else(PoisonError::into_inner);
        for correspondence in correspondences.iter() {
            let dependency = correspondence.dependency();
            let key = dependency.canonical_registration_key();
            let bucket = installed
                .by_dependency
                .get(&key)
                .filter(|bucket| bucket.dependency == *dependency)
                .ok_or_else(|| {
                    BridgeConditionalDenial::new(
                        BridgeConditionalDenialKind::DeclarationCorrespondenceMismatch,
                        "definition successor lost its installed semantic target bucket",
                    )
                })?;
            for target in correspondence.targets.as_slice() {
                let references = bucket.targets.get(target).ok_or_else(|| {
                    BridgeConditionalDenial::new(
                        BridgeConditionalDenialKind::DeclarationCorrespondenceMismatch,
                        "definition successor lost an exact installed semantic target",
                    )
                })?;
                references.checked_add(1).ok_or_else(|| {
                    BridgeConditionalDenial::new(
                        BridgeConditionalDenialKind::ConditionalEvaluationAdmissionCapacity,
                        "owned conditional target reference count is exhausted",
                    )
                })?;
            }
        }
        for correspondence in correspondences.iter() {
            let key = correspondence.dependency().canonical_registration_key();
            let bucket = installed.by_dependency.get_mut(&key).unwrap_or_else(|| {
                unreachable!("validated successor target bucket remains installed")
            });
            for target in correspondence.targets.as_slice() {
                if let Some(references) = bucket.targets.get_mut(target) {
                    *references += 1;
                }
            }
        }
        drop(installed);
        Ok(BridgeOwnedConditionalTargetReservation {
            index: Arc::clone(index),
            correspondences,
            active: true,
        })
    }

    pub(super) fn resolve(
        &self,
        dependency: &BridgeSemanticDependencyCandidate,
    ) -> Result<ProvenCorrespondenceTargets, BridgeConditionalDenial> {
        let key = dependency.canonical_registration_key();
        let bucket = self
            .by_dependency
            .get(&key)
            .filter(|bucket| bucket.dependency == *dependency)
            .ok_or_else(|| {
                BridgeConditionalDenial::new(
                    BridgeConditionalDenialKind::DeclarationCorrespondenceMismatch,
                    "owned semantic dependency has no current target registration",
                )
            })?;
        ProvenCorrespondenceTargets::admit(bucket.targets.keys().cloned().collect()).map_err(
            |kind| {
                BridgeConditionalDenial::new(
                    BridgeConditionalDenialKind::CorrespondenceAdmission,
                    format!("owned semantic target index was denied: {kind:?}"),
                )
            },
        )
    }
}

impl BridgeOwnedConditionalTargetReservation {
    pub(super) fn commit(mut self) {
        self.active = false;
    }
}

impl Drop for BridgeOwnedConditionalTargetReservation {
    fn drop(&mut self) {
        if self.active {
            self.index
                .write()
                .unwrap_or_else(PoisonError::into_inner)
                .unregister_correspondences(&self.correspondences);
        }
    }
}
