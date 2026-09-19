use super::oracle::{adjudicate, removal_expectation, OracleExpectation, OracleRect};
use std::collections::HashMap;

pub(super) mod appearance;
mod attribution;
mod production;

pub(super) use production::{
    application_builder, color, color_token, component, component_identity, establish_allocations,
    execute_frame_with_established_geometry, produce_maximum_overlap, token_identity,
    ProducedMaximumDelta, ProducedUnchanged,
};

pub(super) struct MountedPresentationWorld {
    identity: String,
    version: u16,
    baseline: Box<[OracleRect]>,
    authored_instances: Box<[worth_ui_host_contract::UiMountedInstanceIdentity]>,
    semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    retained: HashMap<
        worth_ui_host_contract::UiMountedInstanceIdentity,
        worth_ui_host_contract::UiMountedSurfaceAppearanceMechanic,
    >,
}

impl MountedPresentationWorld {
    pub(super) fn maximum_overlap(
        transcript: &worth_ui_host_headless::UiHeadlessMountedFrameTranscript,
        authored_instances: Box<[worth_ui_host_contract::UiMountedInstanceIdentity]>,
        semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Self {
        let manifest = include_str!("control_points.toml")
            .parse::<toml::Value>()
            .expect("versioned control-point manifest");
        let identity = text(&manifest, "world_identity").to_owned();
        let version = integer(&manifest, "world_version") as u16;
        let maximum = integer(&manifest, "maximum_surfaces") as usize;
        let controls = manifest["surface"].as_array().expect("surface controls");
        let baseline = (0..maximum)
            .map(|identity| expected_surface(identity, controls))
            .collect::<Vec<_>>();
        assert_exact_rows(transcript, &baseline);
        attribution::assert_exact_attribution(transcript, &authored_instances, semantic_surface);
        let retained = ordered_surfaces(transcript)
            .into_iter()
            .map(|row| (row.node_receipt().mounted_instance(), row.clone()))
            .collect();
        Self {
            identity,
            version,
            baseline: baseline.into_boxed_slice(),
            authored_instances,
            semantic_surface,
            retained,
        }
    }

    pub(super) fn identity(&self) -> &str {
        &self.identity
    }

    pub(super) const fn version(&self) -> u16 {
        self.version
    }

    pub(super) fn baseline(&self) -> &[OracleRect] {
        &self.baseline
    }

    pub(super) fn assert_removal_delta(&mut self, delta: &ProducedMaximumDelta) {
        let count = delta.changed_rows;
        let mut expected_removed = self
            .retained_rows()
            .into_iter()
            .take(count)
            .map(|row| row.node_receipt().mounted_instance())
            .collect::<Vec<_>>();
        expected_removed.sort_unstable();
        let mut expected = removal_expectation(&self.baseline, count);
        let mut candidate = self.removal_candidate(delta);
        for bounds in &mut expected.damage {
            *bounds = bounds.map(|value| value * 1_000);
        }
        expected.damage.sort_unstable();
        candidate.damage.sort_unstable();
        adjudicate(&expected, &candidate).unwrap_or_else(|denial| {
            panic!("appearance removal {count} mismatched: {denial:?}; expected={expected:?}; candidate={candidate:?}")
        });
        assert_exact_cost(delta, count);
        self.apply_changes(&delta.transcript);
        assert_exact_surface_rows(self.retained.values(), &self.baseline[count..]);
        assert_exact_damage(&delta.transcript, &self.baseline[..count]);
        let mut observed_removed = removed_surface_instances(&delta.transcript);
        observed_removed.sort_unstable();
        assert_eq!(observed_removed, expected_removed);
        assert_eq!(
            delta.authored_instances.as_ref(),
            &self.authored_instances[count..]
        );
    }

    pub(super) fn assert_unchanged(&self, unchanged: &ProducedUnchanged) {
        assert_eq!(unchanged.native_work_count, 0);
        assert_eq!(unchanged.cost.delta_rows_carried(), 0);
        assert_eq!(unchanged.cost.draw_list_mutations(), 0);
        assert_eq!(unchanged.cost.order_mutations(), 0);
        assert_eq!(unchanged.cost.logical_damage_regions(), 0);
    }

    pub(super) fn assert_restoration(&mut self, delta: &ProducedMaximumDelta) {
        self.apply_changes(&delta.transcript);
        assert_exact_surface_rows(self.retained.values(), &self.baseline);
        assert_exact_cost(delta, delta.changed_rows);
        assert_exact_damage(&delta.transcript, &self.baseline[..delta.changed_rows]);
        attribution::assert_exact_attribution(
            &delta.transcript,
            &delta.authored_instances[..delta.changed_rows],
            self.semantic_surface,
        );
        assert!(delta.authored_instances[..delta.changed_rows]
            .iter()
            .zip(&self.authored_instances[..delta.changed_rows])
            .all(|(restored, original)| restored != original));
        assert_eq!(
            delta.authored_instances[delta.changed_rows..],
            self.authored_instances[delta.changed_rows..]
        );
    }

    fn apply_changes(
        &mut self,
        transcript: &worth_ui_host_headless::UiHeadlessMountedFrameTranscript,
    ) {
        for fragment in appearance_work(transcript).fragments() {
            if let Some(manifest) = fragment.work().predecessor_manifest() {
                assert!(manifest
                    .mechanic_identities()
                    .iter()
                    .all(|identity| match identity {
                        worth_ui_host_contract::UiMountedAppearanceMechanicIdentity::Surface(
                            instance,
                        ) => self.retained.contains_key(instance),
                        _ => false,
                    }));
            }
            for change in fragment.work().changes() {
                use worth_ui_host_headless::UiHeadlessAppearanceMechanic as Mechanic;
                use worth_ui_host_headless::UiHeadlessAppearanceMechanicChange as Change;
                match change {
                    Change::Insert(Mechanic::Surface(row)) => {
                        assert!(self
                            .retained
                            .insert(row.node_receipt().mounted_instance(), row.clone())
                            .is_none());
                    }
                    Change::Replace {
                        predecessor:
                            worth_ui_host_contract::UiMountedAppearanceMechanicIdentity::Surface(
                                instance,
                            ),
                        successor: Mechanic::Surface(row),
                    } => {
                        assert!(self.retained.contains_key(instance));
                        assert_eq!(*instance, row.node_receipt().mounted_instance());
                        self.retained.insert(*instance, row.clone());
                    }
                    Change::Remove(
                        worth_ui_host_contract::UiMountedAppearanceMechanicIdentity::Surface(
                            instance,
                        ),
                    ) => {
                        assert!(self.retained.remove(instance).is_some());
                    }
                    _ => panic!("maximum-overlap delta contains a non-surface change"),
                }
            }
        }
    }

    fn removal_candidate(&self, delta: &ProducedMaximumDelta) -> OracleExpectation {
        let work = appearance_work(&delta.transcript);
        let removed = removed_surface_instances(&delta.transcript)
            .into_iter()
            .collect::<std::collections::HashSet<_>>();
        let ordered_identities = self
            .retained_rows()
            .into_iter()
            .filter(|row| !removed.contains(&row.node_receipt().mounted_instance()))
            .map(|row| u16::try_from(row.surface_paint_order()).expect("profile order fits"))
            .collect();
        let damage = work
            .fragments()
            .iter()
            .flat_map(|fragment| fragment.work().damage())
            .map(damage_values)
            .collect::<Vec<_>>();
        OracleExpectation {
            owner_delta_count: removed.len(),
            vacated_damage_count: damage.len(),
            damage,
            ordered_identities,
        }
    }

    fn retained_rows(&self) -> Vec<&worth_ui_host_contract::UiMountedSurfaceAppearanceMechanic> {
        let mut rows = self.retained.values().collect::<Vec<_>>();
        rows.sort_by_key(|row| row.surface_paint_order());
        rows
    }
}

fn removed_surface_instances(
    transcript: &worth_ui_host_headless::UiHeadlessMountedFrameTranscript,
) -> Vec<worth_ui_host_contract::UiMountedInstanceIdentity> {
    appearance_work(transcript)
        .fragments()
        .iter()
        .flat_map(|fragment| fragment.work().changes())
        .filter_map(|change| match change {
            worth_ui_host_headless::UiHeadlessAppearanceMechanicChange::Remove(
                worth_ui_host_contract::UiMountedAppearanceMechanicIdentity::Surface(instance),
            ) => Some(*instance),
            _ => None,
        })
        .collect()
}

fn expected_surface(identity: usize, controls: &[toml::Value]) -> OracleRect {
    let order = u16::try_from(identity).expect("maximum surface identity fits the profile");
    let Some(expected) = controls.get(identity) else {
        return OracleRect {
            identity: order,
            bounds: [0, 0, 160, 96],
            rgba: [47, 129, 247, 255],
            order,
        };
    };
    OracleRect {
        identity: order,
        bounds: array4(expected, "x", "y", "width", "height"),
        rgba: rgba(expected),
        order: u16::try_from(integer(expected, "order")).expect("control order fits profile"),
    }
}

fn assert_exact_cost(delta: &ProducedMaximumDelta, count: usize) {
    let count = count as u64;
    assert_eq!(delta.draw_mutations, count);
    assert_eq!(delta.order_mutations, 0);
    assert_eq!(delta.damage_regions, count);
    assert_eq!(delta.delta_rows_carried, count * 3);
}

fn assert_exact_rows(
    transcript: &worth_ui_host_headless::UiHeadlessMountedFrameTranscript,
    expected: &[OracleRect],
) {
    assert_exact_surface_rows(ordered_surfaces(transcript), expected);
}

fn assert_exact_surface_rows<'a>(
    observed: impl IntoIterator<Item = &'a worth_ui_host_contract::UiMountedSurfaceAppearanceMechanic>,
    expected: &[OracleRect],
) {
    let mut observed = observed.into_iter().collect::<Vec<_>>();
    observed.sort_by_key(|row| row.surface_paint_order());
    assert_eq!(observed.len(), expected.len());
    for (row, expected) in observed.into_iter().zip(expected) {
        assert_eq!(
            surface_bounds(row),
            expected.bounds.map(|value| u32::from(value) * 1_000)
        );
        let worth_ui_host_contract::UiMountedSurfacePaint::Fill(
            worth_ui_host_contract::UiMountedSurfaceFill::Solid(color),
        ) = row.paint()
        else {
            panic!("maximum-overlap surface must be a solid fill")
        };
        assert_eq!(color.straight_srgba(), expected.rgba);
        assert_eq!(row.surface_paint_order(), u32::from(expected.order));
    }
}

fn assert_exact_damage(
    transcript: &worth_ui_host_headless::UiHeadlessMountedFrameTranscript,
    removed: &[OracleRect],
) {
    let observed = appearance_work(transcript)
        .fragments()
        .iter()
        .flat_map(|fragment| fragment.work().damage())
        .map(damage_values)
        .collect::<Vec<_>>();
    let mut observed = observed;
    let mut expected = removed
        .iter()
        .map(|row| row.bounds.map(|value| i32::from(value) * 1_000))
        .collect::<Vec<_>>();
    observed.sort_unstable();
    expected.sort_unstable();
    assert_eq!(observed, expected);
}

pub(super) fn ordered_surfaces(
    transcript: &worth_ui_host_headless::UiHeadlessMountedFrameTranscript,
) -> Vec<&worth_ui_host_contract::UiMountedSurfaceAppearanceMechanic> {
    let mut rows = appearance_work(transcript)
        .fragments()
        .iter()
        .flat_map(|fragment| fragment.work().successor().mechanics())
        .filter_map(|mechanic| match mechanic {
            worth_ui_host_headless::UiHeadlessAppearanceMechanic::Surface(row) => Some(row),
            _ => None,
        })
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| row.surface_paint_order());
    rows
}

fn appearance_work(
    transcript: &worth_ui_host_headless::UiHeadlessMountedFrameTranscript,
) -> &worth_ui_host_headless::UiHeadlessAppearancePresentationTranscript {
    transcript
        .appearance_work()
        .expect("appearance work is required")
}

fn surface_bounds(row: &worth_ui_host_contract::UiMountedSurfaceAppearanceMechanic) -> [u32; 4] {
    let bounds = row.bounds();
    [
        u32::try_from(bounds.x()).expect("profile x is nonnegative"),
        u32::try_from(bounds.y()).expect("profile y is nonnegative"),
        bounds.width(),
        bounds.height(),
    ]
}

fn damage_values(bounds: &worth_ui_host_contract::UiAppearanceDamageRegion) -> [i32; 4] {
    [
        bounds.x(),
        bounds.y(),
        i32::try_from(bounds.width()).expect("profile damage width fits"),
        i32::try_from(bounds.height()).expect("profile damage height fits"),
    ]
}

fn text<'a>(value: &'a toml::Value, key: &str) -> &'a str {
    value[key]
        .as_str()
        .unwrap_or_else(|| panic!("{key} string"))
}

fn integer(value: &toml::Value, key: &str) -> u64 {
    value[key]
        .as_integer()
        .and_then(|value| u64::try_from(value).ok())
        .unwrap_or_else(|| panic!("{key} unsigned integer"))
}

fn array4(value: &toml::Value, a: &str, b: &str, c: &str, d: &str) -> [u16; 4] {
    [a, b, c, d].map(|key| integer(value, key) as u16)
}

fn rgba(value: &toml::Value) -> [u8; 4] {
    let channels = value["rgba"].as_array().expect("rgba array");
    std::array::from_fn(|index| channels[index].as_integer().unwrap() as u8)
}
