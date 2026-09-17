use std::collections::BTreeMap;
use std::sync::Arc;

use super::super::{UiCollectionProjectionValue, UiPresentProjection, UiProjectionAvailability};
use super::collection_catalog::{UiProjectionInputCollectionCatalog, UiProjectionOptionKey};
use super::{
    UiCollectionProjectionInputFact, UiProjectionInputCollectionRow,
    UiProjectionInputFactReference, UiProjectionInputPosture, UiProjectionInputRevision,
    UiProjectionInputSlot, UiProjectionInputTransitionStopKind, UiProjectionInputTransitionWork,
};

fn collection_input(
    availability: &UiProjectionAvailability<UiCollectionProjectionValue>,
) -> (
    UiProjectionInputPosture,
    Option<super::UiCollectionCompleteness>,
    Box<[UiProjectionInputCollectionRow]>,
) {
    match availability {
        UiProjectionAvailability::Present(UiPresentProjection::Current(value)) => (
            UiProjectionInputPosture::Current,
            Some(value.completeness()),
            collection_rows(value),
        ),
        UiProjectionAvailability::Present(UiPresentProjection::RetainedStale {
            value,
            activity,
        }) => (
            UiProjectionInputPosture::RetainedStale(activity.kind()),
            Some(value.completeness()),
            collection_rows(value),
        ),
        UiProjectionAvailability::Unavailable(receipt) => (
            UiProjectionInputPosture::Unavailable(receipt.kind()),
            None,
            Box::default(),
        ),
        UiProjectionAvailability::Stopped(receipt) => (
            UiProjectionInputPosture::Stopped(receipt.kind()),
            None,
            Box::default(),
        ),
    }
}

fn collection_rows(value: &UiCollectionProjectionValue) -> Box<[UiProjectionInputCollectionRow]> {
    value
        .rows()
        .iter()
        .map(|row| {
            UiProjectionInputCollectionRow::query_issued(
                row.row().clone(),
                row.selected_values()
                    .iter()
                    .map(|value| Arc::from(value.as_str()))
                    .collect(),
                row.application_item_key(),
            )
        })
        .collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiProjectionInputFactTransition {
    kind: UiProjectionInputFactTransitionKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum UiProjectionInputFactTransitionKind {
    Replace(UiProjectionInputFactReference),
    CollectionPatch(UiCollectionProjectionInputPatch),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UiCollectionProjectionInputPatch {
    revision: UiProjectionInputRevision,
    posture: UiProjectionInputPosture,
    completeness: Option<super::UiCollectionCompleteness>,
    changes: Box<[UiCollectionProjectionInputChange]>,
    preparation_stop: Option<UiProjectionInputTransitionStopKind>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum UiCollectionProjectionInputChange {
    Insert {
        row: UiProjectionInputCollectionRow,
        at: usize,
    },
    Remove {
        row: super::UiCollectionProjectionRowReference,
        from: usize,
    },
    Move {
        row: super::UiCollectionProjectionRowReference,
        from: usize,
        to: usize,
    },
    Regroup(super::UiCollectionProjectionRowReference),
    Update(UiProjectionInputCollectionRow),
    WindowShift,
}

impl UiProjectionInputFactTransition {
    pub(super) fn replace(input: UiProjectionInputFactReference) -> Self {
        Self {
            kind: UiProjectionInputFactTransitionKind::Replace(input),
        }
    }

    pub fn revision(&self) -> &UiProjectionInputRevision {
        match &self.kind {
            UiProjectionInputFactTransitionKind::Replace(input) => input.revision(),
            UiProjectionInputFactTransitionKind::CollectionPatch(patch) => &patch.revision,
        }
    }

    pub fn apply(
        &self,
        predecessor: Option<&UiProjectionInputFactReference>,
    ) -> UiProjectionInputFactReference {
        match &self.kind {
            UiProjectionInputFactTransitionKind::Replace(input) => input.clone(),
            UiProjectionInputFactTransitionKind::CollectionPatch(patch) => patch.apply(predecessor),
        }
    }
}

impl UiCollectionProjectionInputPatch {
    fn apply(
        &self,
        predecessor: Option<&UiProjectionInputFactReference>,
    ) -> UiProjectionInputFactReference {
        if self.posture != UiProjectionInputPosture::Current {
            return self.invalidated(self.posture);
        }
        if let Some(stop) = self.preparation_stop {
            return self.stopped(stop);
        }
        let Some(UiProjectionInputFactReference::Collection(predecessor)) = predecessor else {
            let stop = match predecessor {
                None => UiProjectionInputTransitionStopKind::MissingPredecessor,
                Some(_) => UiProjectionInputTransitionStopKind::WrongShape,
            };
            return self.stopped(stop);
        };
        if !predecessor
            .revision
            .has_same_projection_owner(&self.revision)
        {
            return self.stopped(UiProjectionInputTransitionStopKind::ProjectionChanged);
        }
        if predecessor.posture != UiProjectionInputPosture::Current {
            return self.stopped(UiProjectionInputTransitionStopKind::PredecessorNotCurrent);
        }
        let Some(mut catalog) = predecessor.catalog.clone() else {
            return self.stopped(UiProjectionInputTransitionStopKind::PredecessorNotCurrent);
        };
        let mut work = UiProjectionInputTransitionWork::default();
        for change in &self.changes {
            let mutation = match change {
                UiCollectionProjectionInputChange::Insert { row, at } => {
                    catalog.insert(row.clone(), *at)
                }
                UiCollectionProjectionInputChange::Remove { row, from } => {
                    catalog.remove(row.query_identity(), *from)
                }
                UiCollectionProjectionInputChange::Move { row, from, to } => {
                    catalog.move_row(row.query_identity(), *from, *to)
                }
                UiCollectionProjectionInputChange::Regroup(row) => {
                    catalog.require(row.query_identity())
                }
                UiCollectionProjectionInputChange::Update(row) => catalog.update(row.clone()),
                UiCollectionProjectionInputChange::WindowShift => {
                    Ok(UiProjectionInputTransitionWork::default())
                }
            };
            let Ok(mutation) = mutation else {
                return self.stopped(UiProjectionInputTransitionStopKind::MalformedPatch);
            };
            if work.record_change(mutation).is_err() {
                return self.stopped(UiProjectionInputTransitionStopKind::MalformedPatch);
            }
        }
        collection_reference(
            self.revision.clone(),
            self.posture,
            self.completeness,
            Some(catalog),
            work,
        )
    }

    fn invalidated(&self, posture: UiProjectionInputPosture) -> UiProjectionInputFactReference {
        collection_reference(
            self.revision.clone(),
            posture,
            self.completeness,
            None,
            UiProjectionInputTransitionWork::default(),
        )
    }

    fn stopped(&self, stop: UiProjectionInputTransitionStopKind) -> UiProjectionInputFactReference {
        self.invalidated(UiProjectionInputPosture::TransitionStopped(stop))
    }
}

pub(super) fn from_fact(
    fact: &super::UiCollectionProjectionFactReceipt,
    slot: UiProjectionInputSlot,
) -> UiProjectionInputFactTransition {
    let revision = UiProjectionInputRevision::from_fact(slot, fact.core());
    let (posture, completeness, rows) = collection_input(fact.availability());
    match fact.delivery() {
        super::UiCollectionProjectionDelivery::Snapshot => {
            snapshot_transition(revision, posture, completeness, rows)
        }
        super::UiCollectionProjectionDelivery::Patch => {
            patch_transition(fact, revision, posture, completeness, rows)
        }
    }
}

fn snapshot_transition(
    revision: UiProjectionInputRevision,
    posture: UiProjectionInputPosture,
    completeness: Option<super::UiCollectionCompleteness>,
    rows: Box<[UiProjectionInputCollectionRow]>,
) -> UiProjectionInputFactTransition {
    let (catalog, work, posture) = if matches!(
        posture,
        UiProjectionInputPosture::Current | UiProjectionInputPosture::RetainedStale(_)
    ) {
        match UiProjectionInputCollectionCatalog::replace(rows) {
            Ok((catalog, work)) => (Some(catalog), work, posture),
            Err(()) => (
                None,
                UiProjectionInputTransitionWork::default(),
                UiProjectionInputPosture::TransitionStopped(
                    UiProjectionInputTransitionStopKind::MalformedPatch,
                ),
            ),
        }
    } else {
        (None, UiProjectionInputTransitionWork::default(), posture)
    };
    UiProjectionInputFactTransition::replace(collection_reference(
        revision,
        posture,
        completeness,
        catalog,
        work,
    ))
}

fn patch_transition(
    fact: &super::UiCollectionProjectionFactReceipt,
    revision: UiProjectionInputRevision,
    posture: UiProjectionInputPosture,
    completeness: Option<super::UiCollectionCompleteness>,
    rows: Box<[UiProjectionInputCollectionRow]>,
) -> UiProjectionInputFactTransition {
    let (changes, preparation_stop) = if posture == UiProjectionInputPosture::Current {
        prepare_changes(fact.changes(), rows)
    } else {
        (Box::default(), None)
    };
    UiProjectionInputFactTransition {
        kind: UiProjectionInputFactTransitionKind::CollectionPatch(
            UiCollectionProjectionInputPatch {
                revision,
                posture,
                completeness,
                changes,
                preparation_stop,
            },
        ),
    }
}

fn prepare_changes(
    changes: &[super::UiCollectionProjectionChange],
    rows: Box<[UiProjectionInputCollectionRow]>,
) -> (
    Box<[UiCollectionProjectionInputChange]>,
    Option<UiProjectionInputTransitionStopKind>,
) {
    let mut changed_rows = rows
        .into_vec()
        .into_iter()
        .map(|row| {
            (
                UiProjectionOptionKey::new(row.row().query_identity().clone()),
                row,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut prepared = Vec::with_capacity(changes.len());
    for change in changes {
        let next = match change {
            super::UiCollectionProjectionChange::Insert { row, at } => changed_rows
                .remove(&UiProjectionOptionKey::new(row.query_identity().clone()))
                .map(|row| UiCollectionProjectionInputChange::Insert { row, at: *at }),
            super::UiCollectionProjectionChange::Remove { row, from } => {
                Some(UiCollectionProjectionInputChange::Remove {
                    row: row.clone(),
                    from: *from,
                })
            }
            super::UiCollectionProjectionChange::Move { row, from, to } => {
                Some(UiCollectionProjectionInputChange::Move {
                    row: row.clone(),
                    from: *from,
                    to: *to,
                })
            }
            super::UiCollectionProjectionChange::Regroup { row, .. } => {
                Some(UiCollectionProjectionInputChange::Regroup(row.clone()))
            }
            super::UiCollectionProjectionChange::Update { row } => changed_rows
                .remove(&UiProjectionOptionKey::new(row.query_identity().clone()))
                .map(UiCollectionProjectionInputChange::Update),
            super::UiCollectionProjectionChange::WindowShift => {
                Some(UiCollectionProjectionInputChange::WindowShift)
            }
            super::UiCollectionProjectionChange::ResetRequired { .. } => None,
        };
        let Some(next) = next else {
            return (
                Box::default(),
                Some(UiProjectionInputTransitionStopKind::MalformedPatch),
            );
        };
        prepared.push(next);
    }
    if !changed_rows.is_empty() {
        return (
            Box::default(),
            Some(UiProjectionInputTransitionStopKind::MalformedPatch),
        );
    }
    (prepared.into_boxed_slice(), None)
}

fn collection_reference(
    revision: UiProjectionInputRevision,
    posture: UiProjectionInputPosture,
    completeness: Option<super::UiCollectionCompleteness>,
    catalog: Option<UiProjectionInputCollectionCatalog>,
    transition_work: UiProjectionInputTransitionWork,
) -> UiProjectionInputFactReference {
    UiProjectionInputFactReference::Collection(Arc::new(UiCollectionProjectionInputFact {
        revision,
        posture,
        completeness,
        catalog,
        transition_work,
    }))
}
