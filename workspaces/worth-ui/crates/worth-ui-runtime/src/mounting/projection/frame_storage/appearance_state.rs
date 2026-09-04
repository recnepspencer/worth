use crate::runtime::appearance::{
    UiAppearanceChangeReceipt, UiAppearanceInspectionDenial, UiAppearanceInspectionRecord,
    UiAppearanceInvalidationBatch, UiAppearanceMountAffinity, UiAppearanceProjectionAttempt,
};

pub(crate) const APPEARANCE_STATE_CAPACITY: usize = 4_096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAppearanceStateCapacityExceeded {
    capacity: usize,
}

impl UiAppearanceStateCapacityExceeded {
    pub const fn capacity(self) -> usize {
        self.capacity
    }
}

#[derive(Clone, Default)]
pub(crate) struct UiMountedAppearanceFrameState {
    entries: Vec<UiMountedAppearanceStateEntry>,
    staged: Vec<UiAppearanceProjectionAttempt>,
    reserved: Vec<UiMountedAppearanceStateKey>,
    batch: Option<UiAppearanceInvalidationBatch>,
    capacity_error: Option<UiAppearanceStateCapacityExceeded>,
    reconstruction: bool,
}

#[derive(Clone)]
struct UiMountedAppearanceStateEntry {
    key: UiMountedAppearanceStateKey,
    projection: crate::runtime::appearance::UiAppearanceProjection,
    sidecar: super::super::appearance::UiMountedAppearanceSidecar,
}

#[derive(Clone, Eq, PartialEq)]
struct UiMountedAppearanceStateKey {
    session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    graph_node: crate::graph::UiGraphNodeIdentity,
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    incarnation: worth_ui_host_contract::UiMountIncarnation,
}

impl UiMountedAppearanceFrameState {
    pub(crate) fn inherit_from(&mut self, predecessor: Option<&Self>) {
        self.entries = predecessor.map_or_else(Vec::new, |state| state.entries.clone());
        self.staged.clear();
        self.reserved.clear();
        self.batch = None;
        self.capacity_error = None;
        self.reconstruction = false;
    }

    pub(crate) fn set_batch(&mut self, batch: UiAppearanceInvalidationBatch) {
        self.batch = Some(batch);
    }

    pub(crate) fn begin_reconstruction(&mut self) {
        self.reconstruction = true;
    }

    pub(crate) fn batch(&self) -> Option<&UiAppearanceInvalidationBatch> {
        self.batch.as_ref()
    }

    pub(crate) fn prune_to_current_nodes(
        &mut self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        nodes: &[super::UiMountedAppearanceNodeInputContext],
    ) {
        self.entries.retain(|entry| {
            entry.key.session == session
                && entry.key.generation == *generation
                && nodes.iter().any(|node| {
                    entry.key.surface == node.semantic_surface
                        && entry.key.graph_node == node.graph_node
                        && entry.key.mounted_instance == node.mounted_instance
                        && entry.key.incarnation == node.incarnation
                })
        });
    }

    pub(crate) fn reserve(
        &mut self,
        context: &crate::runtime::appearance::UiAppearanceAttemptContext,
    ) -> Result<(), UiAppearanceStateCapacityExceeded> {
        let key = state_key(context);
        if self.entries.iter().any(|entry| entry.key == key)
            || self
                .staged
                .iter()
                .any(|attempt| state_key(attempt.context()) == key)
            || self.reserved.contains(&key)
        {
            return Ok(());
        }
        if self.membership_count() >= APPEARANCE_STATE_CAPACITY {
            return Err(UiAppearanceStateCapacityExceeded {
                capacity: APPEARANCE_STATE_CAPACITY,
            });
        }
        self.reserved.push(key);
        Ok(())
    }

    fn membership_count(&self) -> usize {
        let mut keys = self
            .entries
            .iter()
            .map(|entry| entry.key.clone())
            .collect::<Vec<_>>();
        for key in self
            .staged
            .iter()
            .map(|attempt| state_key(attempt.context()))
            .chain(self.reserved.iter().cloned())
        {
            if !keys.iter().any(|existing| existing == &key) {
                keys.push(key);
            }
        }
        keys.len()
    }

    pub(crate) fn stage(
        &mut self,
        attempt: UiAppearanceProjectionAttempt,
    ) -> Result<(), UiAppearanceStateCapacityExceeded> {
        if let Err(error) = self.reserve(attempt.context()) {
            self.capacity_error = Some(error);
            return Err(error);
        }
        let key = state_key(attempt.context());
        self.reserved.retain(|reserved| reserved != &key);
        self.staged.push(attempt);
        Ok(())
    }

    pub(crate) const fn capacity_error(&self) -> Option<UiAppearanceStateCapacityExceeded> {
        self.capacity_error
    }

    pub(crate) fn lower(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    ) -> Vec<UiAppearanceInspectionRecord> {
        if self.reconstruction {
            return self.lower_reconstruction();
        }
        let staged = std::mem::take(&mut self.staged);
        let mut records = Vec::with_capacity(staged.len());
        for attempt in staged {
            let context = attempt.context();
            let Some(projection) = attempt.projection() else {
                records.push(denial_record(
                    context.clone(),
                    attempt
                        .denial()
                        .unwrap_or(UiAppearanceInspectionDenial::Basis),
                ));
                continue;
            };
            let key = state_key(context);
            let predecessor = self
                .entries
                .iter()
                .find(|entry| entry.key == key)
                .map(|entry| (entry.projection.clone(), entry.sidecar.clone()));
            let mut sidecar = predecessor
                .as_ref()
                .map_or_else(Default::default, |(_, sidecar)| sidecar.clone());
            let input = match context.lower_resolved(projection, presentation) {
                Ok(input) => input,
                Err(_) => {
                    records.push(denial_record(
                        context.clone(),
                        UiAppearanceInspectionDenial::MountLowering,
                    ));
                    continue;
                }
            };
            let affinity = UiAppearanceMountAffinity {
                session: context.target().session(),
                generation: context.generation().clone(),
                frame: context.frame(),
                surface: context.semantic_surface(),
                graph_node: context.graph_node(),
                mounted_instance: context.mounted_instance(),
                incarnation: context.incarnation(),
                node_receipt: context.node_receipt(),
                issuer: context.issuer(),
                presentation,
            };
            let work = match sidecar.mount(input) {
                Ok(work) => work,
                Err(_) => {
                    records.push(denial_record(
                        context.clone(),
                        UiAppearanceInspectionDenial::MountLowering,
                    ));
                    continue;
                }
            };
            let predecessor_projection = predecessor.as_ref().map(|(projection, _)| projection);
            let receipt = match UiAppearanceChangeReceipt::from_resolved_mount(
                predecessor_projection,
                projection,
                &work,
                affinity,
            ) {
                Ok(receipt) => receipt,
                Err(_) => {
                    records.push(denial_record(
                        context.clone(),
                        UiAppearanceInspectionDenial::MountAffinity,
                    ));
                    continue;
                }
            };
            replace_entry(
                &mut self.entries,
                UiMountedAppearanceStateEntry {
                    key,
                    projection: projection.clone(),
                    sidecar,
                },
            )
            .expect("appearance state reservation admits the staged successor");
            records.push(UiAppearanceInspectionRecord::Projection {
                projection: projection.clone(),
                consumers_selected: context.consumers_selected(),
                receipt,
            });
        }
        records
    }

    fn lower_reconstruction(&self) -> Vec<UiAppearanceInspectionRecord> {
        self.entries
            .iter()
            .map(|entry| {
                let work = entry
                    .sidecar
                    .reconstruction_work()
                    .expect("retained appearance state has mounted facts");
                let receipt = UiAppearanceChangeReceipt::from_reconstruction_mount(&work)
                    .expect("retained appearance facts produce reconstruction work");
                UiAppearanceInspectionRecord::Projection {
                    projection: entry.projection.clone(),
                    consumers_selected: 0,
                    receipt,
                }
            })
            .collect()
    }
}

fn state_key(
    context: &crate::runtime::appearance::UiAppearanceAttemptContext,
) -> UiMountedAppearanceStateKey {
    UiMountedAppearanceStateKey {
        session: context.target().session(),
        generation: context.generation().clone(),
        surface: context.semantic_surface(),
        graph_node: context.graph_node(),
        mounted_instance: context.mounted_instance(),
        incarnation: context.incarnation(),
    }
}

fn replace_entry(
    entries: &mut Vec<UiMountedAppearanceStateEntry>,
    successor: UiMountedAppearanceStateEntry,
) -> Result<(), UiAppearanceStateCapacityExceeded> {
    if let Some(entry) = entries.iter_mut().find(|entry| entry.key == successor.key) {
        *entry = successor;
    } else if entries.len() < APPEARANCE_STATE_CAPACITY {
        entries.push(successor);
    } else {
        return Err(UiAppearanceStateCapacityExceeded {
            capacity: APPEARANCE_STATE_CAPACITY,
        });
    }
    Ok(())
}

fn denial_record(
    context: crate::runtime::appearance::UiAppearanceAttemptContext,
    denial: UiAppearanceInspectionDenial,
) -> UiAppearanceInspectionRecord {
    UiAppearanceInspectionRecord::Denial {
        context,
        denial,
        receipt: UiAppearanceChangeReceipt::for_denial(),
    }
}
