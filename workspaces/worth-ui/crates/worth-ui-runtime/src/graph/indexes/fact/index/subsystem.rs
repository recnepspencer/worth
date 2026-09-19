use std::collections::BTreeMap;

use crate::declaration::UiAspectName;
use crate::fact_contract::{
    UiConsumedFactContract, UiProducedFactFamily, UiSubsystemConsumedFactRule,
};
use crate::graph::UiGraphSnapshot;

use super::{canonical_entries, consumer_identity, consumer_key, UiGraphFactIndexEntry};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct UiGraphSubsystemFactIndex {
    host_viewport: Box<[UiGraphFactIndexEntry]>,
    host_device_scale: Box<[UiGraphFactIndexEntry]>,
    pointer_presence_target: Box<[UiGraphFactIndexEntry]>,
    measurement: Box<[UiGraphFactIndexEntry]>,
    query: Box<[UiGraphFactIndexEntry]>,
    committed_scroll_extent: Box<[UiGraphFactIndexEntry]>,
    committed_portal_anchor: Box<[UiGraphFactIndexEntry]>,
    committed_focus: Box<[UiGraphFactIndexEntry]>,
    committed_selection: Box<[UiGraphFactIndexEntry]>,
    committed_motion_track: Box<[UiGraphFactIndexEntry]>,
    committed_command_route: Box<[UiGraphFactIndexEntry]>,
}

impl UiGraphSubsystemFactIndex {
    pub(super) fn entries_for(&self, family: UiProducedFactFamily) -> &[UiGraphFactIndexEntry] {
        match family {
            UiProducedFactFamily::AuthoredSource => &[],
            UiProducedFactFamily::HostViewport => &self.host_viewport,
            UiProducedFactFamily::HostDeviceScale => &self.host_device_scale,
            UiProducedFactFamily::PointerPresenceTarget => &self.pointer_presence_target,
            UiProducedFactFamily::Measurement => &self.measurement,
            UiProducedFactFamily::Query => &self.query,
            UiProducedFactFamily::IntentPosture => &[],
            UiProducedFactFamily::CommittedScrollExtent => &self.committed_scroll_extent,
            UiProducedFactFamily::CommittedPortalAnchor => &self.committed_portal_anchor,
            UiProducedFactFamily::CommittedFocus => &self.committed_focus,
            UiProducedFactFamily::CommittedSelection => &self.committed_selection,
            UiProducedFactFamily::CommittedMotionTrack => &self.committed_motion_track,
            UiProducedFactFamily::CommittedCommandRoute => &self.committed_command_route,
        }
    }
}

pub(super) fn build_subsystem_index(snapshot: &UiGraphSnapshot) -> UiGraphSubsystemFactIndex {
    let mut by_family = BTreeMap::<UiProducedFactFamily, Vec<UiGraphFactIndexEntry>>::new();
    for (aspect, consumers) in snapshot.core_indexes().consumed_aspects().iter() {
        add_subsystem_aspect_entries(snapshot, aspect, consumers, &mut by_family);
    }
    UiGraphSubsystemFactIndex {
        host_viewport: take_family(&mut by_family, UiProducedFactFamily::HostViewport),
        host_device_scale: take_family(&mut by_family, UiProducedFactFamily::HostDeviceScale),
        pointer_presence_target: take_family(
            &mut by_family,
            UiProducedFactFamily::PointerPresenceTarget,
        ),
        measurement: take_family(&mut by_family, UiProducedFactFamily::Measurement),
        query: take_family(&mut by_family, UiProducedFactFamily::Query),
        committed_scroll_extent: take_family(
            &mut by_family,
            UiProducedFactFamily::CommittedScrollExtent,
        ),
        committed_portal_anchor: take_family(
            &mut by_family,
            UiProducedFactFamily::CommittedPortalAnchor,
        ),
        committed_focus: take_family(&mut by_family, UiProducedFactFamily::CommittedFocus),
        committed_selection: take_family(&mut by_family, UiProducedFactFamily::CommittedSelection),
        committed_motion_track: take_family(
            &mut by_family,
            UiProducedFactFamily::CommittedMotionTrack,
        ),
        committed_command_route: take_family(
            &mut by_family,
            UiProducedFactFamily::CommittedCommandRoute,
        ),
    }
}

fn add_subsystem_aspect_entries(
    snapshot: &UiGraphSnapshot,
    aspect: &UiAspectName,
    consumers: &[crate::graph::UiGraphAspectConsumer],
    by_family: &mut BTreeMap<UiProducedFactFamily, Vec<UiGraphFactIndexEntry>>,
) {
    for rule in UiSubsystemConsumedFactRule::all() {
        if rule.affected_aspect_family() != aspect.family() {
            continue;
        }
        let contract = UiConsumedFactContract::declared_aspect(rule.fact_family(), aspect.clone())
            .expect("a matching subsystem rule constructs one consumed-fact contract");
        for consumer in consumers {
            by_family
                .entry(rule.fact_family())
                .or_default()
                .push(UiGraphFactIndexEntry::new(
                    consumer_key(snapshot, consumer.kind()),
                    consumer_identity(consumer.kind()),
                    Some(aspect.clone()),
                    contract.clone(),
                ));
        }
    }
}

fn take_family(
    by_family: &mut BTreeMap<UiProducedFactFamily, Vec<UiGraphFactIndexEntry>>,
    family: UiProducedFactFamily,
) -> Box<[UiGraphFactIndexEntry]> {
    canonical_entries(by_family.remove(&family).unwrap_or_default())
}
