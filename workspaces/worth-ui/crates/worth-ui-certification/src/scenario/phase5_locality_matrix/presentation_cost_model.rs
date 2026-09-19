//! Independent exact model for mounted production and native presentation work.

use worth_ui_host_contract::{
    UiHostPresentationCostReport as NativeCost,
    UiMountedPresentationProductionCost as ProductionCost,
};

use super::case::{Phase5LocalityAxis as Axis, Phase5LocalityCase};

pub(super) fn adjudicate(
    case: Phase5LocalityCase,
    production: ProductionCost,
    native: NativeCost,
) -> Result<(), String> {
    require_production(case, production)?;
    require_native(case, native)
}

fn require_production(case: Phase5LocalityCase, cost: ProductionCost) -> Result<(), String> {
    let paragraphs = case.retained_paragraphs() as u64;
    let reconstructive = case.axis() == Axis::Dpi;
    let completion_fanout = case.axis() == Axis::UploadCompletion;
    let sources = if reconstructive || completion_fanout {
        paragraphs
    } else {
        1
    };
    require_equal(
        "production source instances",
        cost.source_instances(),
        sources,
    )?;
    require_equal(
        "production commands considered",
        cost.commands_considered(),
        if reconstructive || completion_fanout {
            2 * paragraphs
        } else {
            2
        },
    )?;
    require_equal(
        "production command index lookups",
        cost.command_index_lookups(),
        if reconstructive {
            2 * paragraphs
        } else if completion_fanout {
            4 * paragraphs
        } else {
            4
        },
    )?;
    require_equal(
        "production order lookups",
        cost.order_lookups(),
        if reconstructive {
            2 * paragraphs
        } else if completion_fanout {
            4 * paragraphs
        } else {
            4
        },
    )?;
    require_equal(
        "production retained scans",
        cost.retained_command_scans(),
        if reconstructive { 2 * paragraphs } else { 0 },
    )?;
    require_equal(
        "production retained clones",
        cost.retained_command_clones(),
        0,
    )?;
    require_equal(
        "production materialized rows",
        cost.projection_rows_materialized(),
        if reconstructive {
            3 * paragraphs + 3
        } else {
            0
        },
    )
}

fn require_native(case: Phase5LocalityCase, cost: NativeCost) -> Result<(), String> {
    let expected = NativeExpectation::for_case(case);
    for (label, actual, wanted) in [
        ("presented surfaces", cost.presented_surfaces(), 1),
        (
            "translated rows",
            cost.translated_rows(),
            expected.translated_rows,
        ),
        ("translated bytes", cost.translated_bytes(), 0),
        (
            "cache hits",
            cost.native_resource_cache_hits(),
            expected.cache_hits,
        ),
        ("cache misses", cost.native_resource_cache_misses(), 0),
        ("async handoffs", cost.asynchronous_handoffs(), 0),
        ("delta rows", cost.delta_rows_carried(), expected.delta_rows),
        (
            "draw-list mutations",
            cost.draw_list_mutations(),
            expected.draw_mutations,
        ),
        (
            "order mutations",
            cost.order_mutations(),
            expected.order_mutations,
        ),
        (
            "order lookups",
            cost.order_index_lookups(),
            expected.order_lookups,
        ),
        ("order rotations", cost.order_index_rotations(), 0),
        (
            "order high water",
            cost.order_index_high_water(),
            expected.order_high_water,
        ),
        (
            "damage regions",
            cost.logical_damage_regions(),
            expected.damage_regions,
        ),
        ("logical damage pixels", cost.logical_damage_pixels(), 0),
        (
            "retained scans",
            cost.retained_command_scans(),
            expected.retained_scans,
        ),
        ("retained clones", cost.retained_command_clones(), 0),
        (
            "damage probes",
            cost.damage_index_probes(),
            expected.damage_probes,
        ),
        (
            "damage records",
            cost.damage_index_stored_records(),
            expected.damage_records,
        ),
        (
            "damage high water",
            cost.damage_index_high_water(),
            expected.damage_high_water,
        ),
        (
            "damage command checks",
            cost.damage_region_command_checks(),
            expected.damage_checks,
        ),
        (
            "intersecting commands",
            cost.intersecting_commands(),
            expected.commands,
        ),
        (
            "replayed commands",
            cost.replayed_commands(),
            expected.commands,
        ),
        (
            "cleared pixels",
            cost.cleared_pixels(),
            expected.cleared_pixels,
        ),
        ("GPU writes", cost.gpu_writes(), 1),
        ("render passes", cost.render_passes(), 2),
        ("surface copies", cost.surface_copies(), 1),
        ("surface acquisitions", cost.surface_acquisitions(), 1),
        ("queue submissions", cost.queue_submissions(), 1),
        ("presents", cost.presents(), 1),
        (
            "presented pixels",
            cost.presented_pixels(),
            expected.presented_pixels,
        ),
    ] {
        require_equal(label, actual, wanted)?;
    }
    require_equal(
        "order node touches",
        cost.order_index_node_touches(),
        super::retained_order_reference::expected_touches(case),
    )?;
    require_equal(
        "rendered pixels",
        cost.rendered_pixels(),
        expected_rendered_pixels(case),
    )
}

#[derive(Clone, Copy)]
struct NativeExpectation {
    commands: u64,
    cache_hits: u64,
    translated_rows: u64,
    delta_rows: u64,
    draw_mutations: u64,
    order_mutations: u64,
    order_lookups: u64,
    order_high_water: u64,
    damage_regions: u64,
    retained_scans: u64,
    damage_probes: u64,
    damage_records: u64,
    damage_high_water: u64,
    damage_checks: u64,
    cleared_pixels: u64,
    presented_pixels: u64,
}

impl NativeExpectation {
    fn for_case(case: Phase5LocalityCase) -> Self {
        let paragraphs = case.retained_paragraphs() as u64;
        match case.axis() {
            Axis::Width => Self::local(paragraphs, 4, 3, 7, 6, 54_720, 38_016),
            Axis::Dpi => Self {
                commands: 23 * paragraphs,
                cache_hits: 0,
                translated_rows: 23 * paragraphs,
                delta_rows: 0,
                draw_mutations: 23 * paragraphs,
                order_mutations: 23 * paragraphs,
                order_lookups: 0,
                order_high_water: 0,
                damage_regions: 0,
                retained_scans: 23 * paragraphs,
                damage_probes: 0,
                damage_records: 23 * paragraphs,
                damage_high_water: 23 * paragraphs,
                damage_checks: 0,
                cleared_pixels: 54_000,
                presented_pixels: 54_000,
            },
            Axis::UploadCompletion => Self {
                translated_rows: 3 * paragraphs,
                delta_rows: 7 * paragraphs,
                draw_mutations: 2 * paragraphs,
                ..Self::local(
                    paragraphs,
                    2 * paragraphs,
                    3 * paragraphs,
                    7 * paragraphs,
                    4 * paragraphs - 1,
                    7_776,
                    34_560,
                )
            },
            Axis::PinRelease => {
                let commands = 2 * paragraphs - 2;
                Self {
                    commands,
                    cache_hits: commands,
                    translated_rows: 3,
                    delta_rows: 7,
                    draw_mutations: 2,
                    order_mutations: 2,
                    order_lookups: 2 * paragraphs + 6,
                    order_high_water: 2 * paragraphs,
                    damage_regions: 1,
                    retained_scans: 0,
                    damage_probes: (4 * paragraphs).saturating_sub(5),
                    damage_records: commands,
                    damage_high_water: 2 * paragraphs,
                    damage_checks: commands,
                    cleared_pixels: 7_776,
                    presented_pixels: 34_560,
                }
            }
            Axis::Content | Axis::PaintBoundary | Axis::AtlasMiss => Self::local(
                paragraphs,
                2 * paragraphs,
                3,
                7,
                4 * paragraphs - 1,
                7_776,
                34_560,
            ),
        }
    }

    fn local(
        paragraphs: u64,
        commands: u64,
        translated_rows: u64,
        delta_rows: u64,
        damage_probes: u64,
        cleared_pixels: u64,
        presented_pixels: u64,
    ) -> Self {
        Self {
            commands,
            cache_hits: commands,
            translated_rows,
            delta_rows,
            draw_mutations: 2,
            order_mutations: 0,
            order_lookups: commands,
            order_high_water: 2 * paragraphs,
            damage_regions: if commands == 4 { 2 } else { 1 },
            retained_scans: 0,
            damage_probes,
            damage_records: 2 * paragraphs,
            damage_high_water: 2 * paragraphs,
            damage_checks: commands,
            cleared_pixels,
            presented_pixels,
        }
    }
}

fn expected_rendered_pixels(case: Phase5LocalityCase) -> u64 {
    let paragraphs = case.retained_paragraphs() as u64;
    match case.axis() {
        Axis::Content => 2_551 * paragraphs - 476,
        Axis::Width => 9_282,
        Axis::PaintBoundary => 2_551 * paragraphs,
        Axis::Dpi => 3_402 * paragraphs,
        Axis::AtlasMiss => 2_551 * paragraphs - 51,
        Axis::UploadCompletion => 2_500 * paragraphs,
        Axis::PinRelease => 2_551 * paragraphs.saturating_sub(1),
    }
}

fn require_equal(label: &str, actual: u64, expected: u64) -> Result<(), String> {
    (actual == expected)
        .then_some(())
        .ok_or_else(|| format!("presentation cost {label}: expected {expected}, observed {actual}"))
}
