use std::collections::{BTreeMap, BTreeSet};
use worth_ui_host_contract::{UiMountIncarnation, UiMountedInstanceIdentity};

pub(crate) struct UiApplicationTextRevisionSelection {
    pub(super) revisions: Box<[(Box<str>, crate::graph::UiGraphNodeIdentity, u64)]>,
}

pub(crate) struct UiApplicationTextPublication {
    pub(super) revisions: Box<
        [(
            Box<str>,
            crate::graph::UiGraphNodeIdentity,
            u64,
            UiApplicationTextMountedCoverage,
        )],
    >,
}

pub(crate) struct UiApplicationTextMountedCoverage {
    required: BTreeMap<UiMountedInstanceIdentity, UiMountIncarnation>,
    accepted: BTreeSet<UiMountedInstanceIdentity>,
}

impl UiApplicationTextMountedCoverage {
    pub(crate) fn from_occurrences(
        occurrences: impl IntoIterator<Item = (UiMountedInstanceIdentity, UiMountIncarnation, bool)>,
    ) -> Self {
        let mut required = BTreeMap::new();
        let mut accepted = BTreeSet::new();
        for (instance, incarnation, selected) in occurrences {
            required.insert(instance, incarnation);
            if selected {
                accepted.insert(instance);
            }
        }
        Self { required, accepted }
    }

    fn accumulate(&mut self, next: &Self) {
        if self.required != next.required {
            self.required.clone_from(&next.required);
            self.accepted.clear();
        }
        self.accepted.extend(next.accepted.iter().copied());
    }

    fn complete(&self) -> bool {
        !self.required.is_empty() && self.accepted.len() == self.required.len()
    }
}

impl UiApplicationTextRevisionSelection {
    pub(crate) fn is_empty(&self) -> bool {
        self.revisions.is_empty()
    }

    pub(crate) fn prepare(
        self,
        mounted: &crate::mounting::UiMountedIdentityState,
        surfaces: &[worth_ui_host_contract::UiMountedSurfaceBindingRequirement],
    ) -> Result<
        (UiApplicationTextPublication, usize),
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        let mut revisions = Vec::with_capacity(self.revisions.len());
        let mut work = 0usize;
        for (identity, graph, revision) in self.revisions {
            let (coverage, probes) = mounted.text_publication_coverage(graph, surfaces)?;
            work = work.checked_add(probes).ok_or(
                crate::mounting::UiMountedFramePreparationDenial::Projection(
                    crate::mounting::UiMountedProjectionDenial::CostCounterOverflow,
                ),
            )?;
            revisions.push((identity, graph, revision, coverage));
        }
        Ok((
            UiApplicationTextPublication {
                revisions: revisions.into_boxed_slice(),
            },
            work,
        ))
    }
}

impl super::UiApplicationPresentationState {
    pub(crate) fn settle_published_text(&mut self, publication: &UiApplicationTextPublication) {
        for (identity, graph, revision, coverage) in &publication.revisions {
            let Some(row) = self.rows.get_mut(identity.as_ref()) else {
                continue;
            };
            if row.graph_node != Some(*graph) || row.presentation_revision != *revision {
                continue;
            }
            let pending = row.pending_publication_coverage.get_or_insert_with(|| {
                UiApplicationTextMountedCoverage {
                    required: BTreeMap::new(),
                    accepted: BTreeSet::new(),
                }
            });
            pending.accumulate(coverage);
            if pending.complete() {
                row.projected_presentation_revision = Some(*revision);
                row.pending_publication_coverage = None;
            }
        }
    }
}
