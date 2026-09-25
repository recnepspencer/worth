//! The traffic chart panel. The chart fills the panel inside its padding, and
//! each axis is a layout container whose fixed label tracks alternate with
//! equal flexible spacers, so every label stays on its gridline as the chart
//! resizes.
use worth_ui::facade::declaration::{
    ComponentViewportAxisPlacement, ComponentViewportRegion, MosaicLayoutCell,
    MosaicLayoutContract, MosaicTrack,
};

use super::frame::Edge;
use super::page::Panel;
use super::{
    surface, text, ContainerTracks, DashboardElement, DashboardGraphic, DashboardLayoutCell,
    DashboardPlacement,
};

const X_AXIS: &str = "chart_x_axis";
const Y_AXIS: &str = "chart_y_axis";
/// Where the concept draws the chart's plot area; its gridlines divide it
/// into six columns and three rows.
const GRID: [u16; 4] = [331, 347, 721, 216];
const X_LABELS: [(&str, &str); 7] = [
    ("chart_x_0", "12 AM"),
    ("chart_x_1", "4 AM"),
    ("chart_x_2", "8 AM"),
    ("chart_x_3", "12 PM"),
    ("chart_x_4", "4 PM"),
    ("chart_x_5", "8 PM"),
    ("chart_x_6", "12 AM"),
];
/// Each time label is centered under its vertical gridline.
const X_LABEL_WIDTH: u16 = 44;
const X_LABEL_TOP: u16 = 572;
const X_LABEL_HEIGHT: u16 = 23;
const Y_LABELS: [(&str, &str); 4] = [
    ("chart_y_0", "300K"),
    ("chart_y_1", "200K"),
    ("chart_y_2", "100K"),
    ("chart_y_3", "0"),
];
/// Each volume label keeps its concept offset above its horizontal gridline.
const Y_LABEL_LEAD: u16 = 9;
const Y_LABEL_LEFT: u16 = 290;
const Y_LABEL_WIDTH: u16 = 37;
const Y_LABEL_HEIGHT: u16 = 21;

/// The two axis containers, placed in the chart panel.
pub(super) fn containers() -> [ContainerTracks; 2] {
    let [x, y, width, height] = GRID;
    let frame = Panel::Chart.frame();
    [
        ContainerTracks {
            id: X_AXIS,
            placement: frame.placement(
                [
                    x - X_LABEL_WIDTH / 2,
                    X_LABEL_TOP,
                    width + X_LABEL_WIDTH,
                    X_LABEL_HEIGHT,
                ],
                Edge::Both,
                Edge::End,
            ),
            tracks: MosaicLayoutContract::columns(labels_between_spacers(
                X_LABELS.len(),
                X_LABEL_WIDTH,
            ))
            .expect("the time axis declares its columns"),
            scroll_panel: None,
        },
        ContainerTracks {
            id: Y_AXIS,
            placement: frame.placement(
                [
                    Y_LABEL_LEFT,
                    y - Y_LABEL_LEAD,
                    Y_LABEL_WIDTH,
                    height + Y_LABEL_HEIGHT,
                ],
                Edge::Start,
                Edge::Both,
            ),
            tracks: MosaicLayoutContract::rows(labels_between_spacers(
                Y_LABELS.len(),
                Y_LABEL_HEIGHT,
            ))
            .expect("the volume axis declares its rows"),
            scroll_panel: None,
        },
    ]
}

/// `count` fixed label tracks with an equal flexible spacer between each
/// pair. Across `extent + label` points, label `k` then starts at
/// `k * extent / (count - 1)`.
fn labels_between_spacers(count: usize, label: u16) -> Vec<MosaicTrack> {
    let label = MosaicTrack::fixed(label).expect("a label track has extent");
    let spacer = MosaicTrack::flex(1, 0).expect("a spacer has weight");
    let mut tracks = vec![label];
    for _ in 1..count {
        tracks.extend([spacer, label]);
    }
    tracks
}

/// Places an axis label in its label track, filling it.
fn on_axis(
    element: DashboardElement,
    container: &'static str,
    cell: MosaicLayoutCell,
) -> DashboardElement {
    let fill = ComponentViewportAxisPlacement::stretch_between(0, 0);
    DashboardElement {
        placement: DashboardPlacement::Cell(DashboardLayoutCell {
            container,
            cell,
            region: ComponentViewportRegion::new(fill, fill),
        }),
        ..element
    }
}

pub(super) fn elements() -> Vec<DashboardElement> {
    let frame = Panel::Chart.frame();
    let [grid_x, grid_y, grid_width, grid_height] = GRID;
    let chart = |id, color, paint_order, graphic| {
        frame.place(
            surface(id, GRID, color, 12, false, paint_order).graphic(graphic),
            Edge::Both,
            Edge::Both,
        )
    };
    let mut elements = vec![
        frame.place(
            surface(
                "service_stage",
                [266, 269, 812, 348],
                "raised_surface",
                12,
                true,
                2,
            ),
            Edge::Both,
            Edge::Both,
        ),
        frame.place(
            text(
                "service_body",
                "Traffic over time",
                [291, 288, 310, 35],
                18,
                true,
                "primary_text",
            ),
            Edge::Start,
            Edge::Start,
        ),
        frame.place(
            surface(
                "confirmation_target",
                [930, 283, 130, 34],
                "raised_surface",
                10,
                true,
                3,
            )
            .action("period"),
            Edge::End,
            Edge::Start,
        ),
        frame.place(
            text(
                "confirmation_label",
                "Last 24 hours",
                [938, 283, 92, 34],
                12,
                false,
                "primary_text",
            )
            .centered(),
            Edge::End,
            Edge::Start,
        ),
        frame.place(
            surface(
                "period_chevron",
                [1033, 296, 10, 6],
                "secondary_text",
                12,
                false,
                4,
            )
            .graphic(DashboardGraphic::Chevron),
            Edge::End,
            Edge::Start,
        ),
        chart("chart_grid", "grid", 3, DashboardGraphic::ChartGrid),
        chart("chart_area", "chart_fill", 4, DashboardGraphic::TrafficArea),
        chart(
            "query_accent",
            "principal_accent",
            5,
            DashboardGraphic::TrafficLine,
        ),
    ];
    let step = |extent: u16, count: usize, index: usize| {
        let steps = u16::try_from(count - 1).expect("an axis has few labels");
        let index = u16::try_from(index).expect("an axis has few labels");
        extent * index / steps
    };
    for (index, (id, label)) in Y_LABELS.into_iter().enumerate() {
        let top = grid_y - Y_LABEL_LEAD + step(grid_height, Y_LABELS.len(), index);
        let rect = [Y_LABEL_LEFT, top, Y_LABEL_WIDTH, Y_LABEL_HEIGHT];
        let cell = MosaicLayoutCell::at(0, 2 * index as u16);
        elements.push(on_axis(
            text(id, label, rect, 12, false, "secondary_text"),
            Y_AXIS,
            cell,
        ));
    }
    for (index, (id, label)) in X_LABELS.into_iter().enumerate() {
        let left = grid_x - X_LABEL_WIDTH / 2 + step(grid_width, X_LABELS.len(), index);
        let rect = [left, X_LABEL_TOP, X_LABEL_WIDTH, X_LABEL_HEIGHT];
        let cell = MosaicLayoutCell::at(2 * index as u16, 0);
        elements.push(on_axis(
            text(id, label, rect, 12, false, "secondary_text").centered(),
            X_AXIS,
            cell,
        ));
    }
    elements
}
