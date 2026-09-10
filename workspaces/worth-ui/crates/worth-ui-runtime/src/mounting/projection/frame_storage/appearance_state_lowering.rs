use crate::mounting::projection::appearance::UiMountedAppearanceGeometryScope;
use crate::runtime::appearance::{
    UiAppearanceChangeReceipt, UiAppearanceInspectionDenial, UiAppearanceInspectionRecord,
    UiAppearanceMountAffinity, UiAppearanceProjectionAttempt,
};

use super::super::appearance_output::UiMountedAppearanceOutputDenial;
use super::super::appearance_state_membership::{self, UiMountedAppearanceStateMembership};
use super::super::appearance_state_predecessor::UiMountedAppearanceStatePredecessor;
use super::{UiMountedAppearanceFrameState, UiMountedAppearanceStateEntry};

#[derive(Clone, Copy)]
pub(super) enum AppearanceLoweringPosture {
    Delta,
    Reconstruction,
}

impl UiMountedAppearanceFrameState {
    pub(in crate::mounting::projection::frame_storage) fn lower_retirements(
        &mut self,
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    ) -> Result<(), crate::mounting::projection::appearance::UiMountedAppearanceLoweringDenial>
    {
        if let Some(epoch) = &self.epoch {
            let (removals, work) = self.retirements.lower(epoch.session, frame, presentation);
            self.selection.record_membership_work(work);
            self.node_work.extend(removals?);
        }
        Ok(())
    }

    pub(in crate::mounting::projection::frame_storage) fn take_node_work(
        &mut self,
    ) -> Vec<super::super::appearance_output::UiMountedAppearanceNodeWork> {
        std::mem::take(&mut self.node_work)
    }

    pub(in crate::mounting::projection::frame_storage) fn take_overlay_work(
        &mut self,
    ) -> Vec<super::super::appearance_output::UiMountedAppearanceOverlayWork> {
        std::mem::take(&mut self.overlay_work)
    }

    pub(in crate::mounting::projection::frame_storage) fn reject_unpublished_output(
        &mut self,
        records: Vec<UiAppearanceInspectionRecord>,
    ) -> Vec<UiAppearanceInspectionRecord> {
        let denials = records
            .into_iter()
            .filter_map(|record| match record {
                UiAppearanceInspectionRecord::Denial {
                    context, denial, ..
                } => Some((context.mounted_instance(), denial)),
                _ => None,
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        let denial_for = |context: &crate::runtime::appearance::UiAppearanceAttemptContext| {
            denials
                .get(&context.mounted_instance())
                .copied()
                .unwrap_or(UiAppearanceInspectionDenial::MountLowering)
        };
        let retained_keys = if self.reconstruction_nodes.is_some() {
            let (keys, work) = self.members.keys_for_reconstruction();
            self.selection.record_membership_work(work);
            keys
        } else {
            Vec::new()
        };
        let mut records = Vec::new();
        for key in self.members.take_pending_keys_for_lowering() {
            let (membership, work) = self.members.remove(&key);
            self.selection.record_membership_work(work);
            match membership {
                Some(UiMountedAppearanceStateMembership::Staged {
                    attempt,
                    mut predecessor,
                }) => {
                    records.push(denial_record(
                        attempt.context().clone(),
                        denial_for(attempt.context()),
                    ));
                    self.restore_predecessor(&mut predecessor);
                }
                Some(UiMountedAppearanceStateMembership::Retained(entry)) => {
                    self.restore_entry(entry)
                }
                Some(UiMountedAppearanceStateMembership::PhysicalOnly(physical)) => {
                    self.restore_physical_predecessor(
                        UiMountedAppearanceStatePredecessor::PhysicalOnly(physical),
                    );
                }
                _ => {}
            }
        }
        records.extend(
            retained_keys
                .iter()
                .filter_map(|key| self.members.retained_entry_for_local_node(key))
                .map(|entry| denial_record(entry.context.clone(), denial_for(&entry.context))),
        );
        records
    }

    #[cfg(test)]
    pub(crate) fn lower(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        geometry: &UiMountedAppearanceGeometryScope,
    ) -> Result<Vec<UiAppearanceInspectionRecord>, UiMountedAppearanceOutputDenial> {
        self.lower_with_portals(presentation, geometry, &Default::default())
    }

    pub(crate) fn lower_with_portals(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        geometry: &UiMountedAppearanceGeometryScope,
        portal_instances: &std::collections::BTreeSet<
            worth_ui_host_contract::UiMountedInstanceIdentity,
        >,
    ) -> Result<Vec<UiAppearanceInspectionRecord>, UiMountedAppearanceOutputDenial> {
        self.node_work.clear();
        self.overlay_work.clear();
        if let Some(nodes) = self.reconstruction_nodes.take() {
            self.order = Default::default();
            let complete = std::mem::take(&mut self.reconstruction_complete);
            return self.lower_reconstruction(
                presentation,
                geometry,
                portal_instances,
                &nodes,
                complete,
            );
        }
        let mut records = self.lower_pending(
            presentation,
            geometry,
            portal_instances,
            AppearanceLoweringPosture::Delta,
        )?;
        records.extend(self.lower_input_refreshes(presentation, geometry, portal_instances)?);
        Ok(records)
    }

    pub(super) fn lower_pending(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        geometry: &UiMountedAppearanceGeometryScope,
        portal_instances: &std::collections::BTreeSet<
            worth_ui_host_contract::UiMountedInstanceIdentity,
        >,
        posture: AppearanceLoweringPosture,
    ) -> Result<Vec<UiAppearanceInspectionRecord>, UiMountedAppearanceOutputDenial> {
        let mut records = Vec::new();
        for key in self.members.take_pending_keys_for_lowering() {
            let (membership, work) = self.members.remove(&key);
            self.selection.record_membership_work(work);
            let Some(membership) = membership else {
                continue;
            };
            match membership {
                UiMountedAppearanceStateMembership::Retained(entry) => {
                    self.restore_entry(entry);
                }
                UiMountedAppearanceStateMembership::PhysicalOnly(physical) => {
                    self.restore_physical_predecessor(
                        UiMountedAppearanceStatePredecessor::PhysicalOnly(physical),
                    );
                }
                UiMountedAppearanceStateMembership::Reserved => {}
                UiMountedAppearanceStateMembership::Staged {
                    attempt,
                    predecessor,
                } => self.lower_staged(
                    attempt,
                    predecessor,
                    presentation,
                    geometry,
                    portal_instances,
                    posture,
                    &mut records,
                )?,
            }
        }
        Ok(records)
    }

    fn lower_staged(
        &mut self,
        attempt: UiAppearanceProjectionAttempt,
        mut predecessor: Option<UiMountedAppearanceStatePredecessor>,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        geometry: &UiMountedAppearanceGeometryScope,
        portal_instances: &std::collections::BTreeSet<
            worth_ui_host_contract::UiMountedInstanceIdentity,
        >,
        posture: AppearanceLoweringPosture,
        records: &mut Vec<UiAppearanceInspectionRecord>,
    ) -> Result<(), UiMountedAppearanceOutputDenial> {
        let context = attempt.context();
        let key = appearance_state_membership::state_key(context);
        let Some(projection) = attempt.projection() else {
            records.push(denial_record(
                context.clone(),
                attempt
                    .denial()
                    .unwrap_or(UiAppearanceInspectionDenial::Basis),
            ));
            self.restore_predecessor(&mut predecessor);
            return Ok(());
        };
        if predecessor
            .as_ref()
            .is_some_and(|previous| previous.session() != context.target().session())
        {
            records.push(denial_record(
                context.clone(),
                UiAppearanceInspectionDenial::MountAffinity,
            ));
            self.restore_predecessor(&mut predecessor);
            return Ok(());
        }
        let mut sidecar = predecessor
            .as_ref()
            .map_or_else(Default::default, |previous| previous.sidecar().clone());
        let predecessor_receipt = sidecar.current_node_receipt();
        let mut input = match context.lower_resolved(
            projection,
            presentation,
            geometry.outline_fringe(context.semantic_surface()),
        ) {
            Ok(input) => input,
            Err(denial) => {
                records.push(denial_record(
                    context.clone(),
                    UiAppearanceInspectionDenial::MountLowering,
                ));
                self.restore_predecessor(&mut predecessor);
                return match geometry_output_denial(denial) {
                    Some(denial) => Err(denial),
                    None => Ok(()),
                };
            }
        };
        input
            .compose_accepted_motion(geometry)
            .map_err(|_| UiMountedAppearanceOutputDenial::NodeLowering)?;
        input.retain_node_owned_families(portal_instances);
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
        let lowering = match posture {
            AppearanceLoweringPosture::Reconstruction if predecessor_receipt.is_some() => {
                sidecar.reconstruct(input)
            }
            _ => sidecar.mount(input),
        };
        let work = match lowering {
            Ok(work) => work,
            Err(_) => {
                records.push(denial_record(
                    context.clone(),
                    UiAppearanceInspectionDenial::MountLowering,
                ));
                self.restore_predecessor(&mut predecessor);
                return Ok(());
            }
        };
        let predecessor_projection = predecessor
            .as_ref()
            .and_then(|previous| previous.semantic())
            .map(|entry| &entry.projection);
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
                self.restore_predecessor(&mut predecessor);
                return Ok(());
            }
        };
        self.node_work.push(
            super::super::appearance_output::UiMountedAppearanceNodeWork {
                predecessor: predecessor_receipt,
                successor: Some(context.node_receipt()),
                work,
            },
        );
        self.restore_entry(UiMountedAppearanceStateEntry {
            key,
            context: context.clone(),
            projection: projection.clone(),
            sidecar,
        });
        records.push(UiAppearanceInspectionRecord::Projection {
            projection: projection.clone(),
            consumers_selected: context.consumers_selected(),
            receipt,
        });
        Ok(())
    }

    fn restore_predecessor(
        &mut self,
        predecessor: &mut Option<UiMountedAppearanceStatePredecessor>,
    ) {
        if let Some(entry) = predecessor.take() {
            self.restore_physical_predecessor(entry);
        }
    }

    pub(super) fn restore_physical_predecessor(
        &mut self,
        predecessor: UiMountedAppearanceStatePredecessor,
    ) {
        let (result, work) = self.members.restore_predecessor(predecessor);
        self.selection.record_membership_work(work);
        result.expect("appearance predecessor retains its exact mounted identity");
    }

    pub(super) fn restore_entry(&mut self, entry: UiMountedAppearanceStateEntry) {
        let (result, work) = self.members.insert_retained(entry);
        self.selection.record_membership_work(work);
        result.expect("appearance membership indexes retain one exact local identity");
    }
}

pub(super) fn denial_record(
    context: crate::runtime::appearance::UiAppearanceAttemptContext,
    denial: UiAppearanceInspectionDenial,
) -> UiAppearanceInspectionRecord {
    UiAppearanceInspectionRecord::Denial {
        context,
        denial,
        receipt: UiAppearanceChangeReceipt::for_denial(),
    }
}

pub(super) fn geometry_output_denial(
    denial: crate::mounting::UiMountedAppearanceLoweringDenial,
) -> Option<UiMountedAppearanceOutputDenial> {
    use crate::mounting::UiMountedAppearanceLoweringDenial as Lowering;
    match denial {
        Lowering::AncestorClip(denial) => {
            Some(UiMountedAppearanceOutputDenial::AncestorClip(denial))
        }
        Lowering::HostGeometryProfileUnavailable => {
            Some(UiMountedAppearanceOutputDenial::HostGeometryProfileUnavailable)
        }
        Lowering::HostGeometrySurfaceUnavailable => {
            Some(UiMountedAppearanceOutputDenial::HostGeometrySurfaceUnavailable)
        }
        Lowering::HostGeometryScale(scale) => {
            Some(UiMountedAppearanceOutputDenial::HostGeometryScale(scale))
        }
        _ => None,
    }
}
