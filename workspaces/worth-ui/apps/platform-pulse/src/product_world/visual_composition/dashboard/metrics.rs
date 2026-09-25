//! The summary card row: one flexible layout container whose four cards, and
//! every label, value, and icon inside them, are placed by the row's tracks.
use worth_ui::facade::declaration::{
    ComponentAllocationMeasurementContract, ComponentViewportAxisPlacement,
    ComponentViewportRegion, MosaicLayoutCell, MosaicLayoutContract, MosaicTrack,
};

use super::{surface, text, DashboardContainer, DashboardElement, DashboardGraphic};

const ROW: &str = "metric_row";
const ROW_TOP: u16 = 152;
const CARD_HEIGHT: u16 = 98;
const CARD_GAP: u16 = 16;
/// Each card's origin and width at the canonical 1536-point extent. Each
/// width is both its column's weight and its minimum: a wider row grows the
/// cards in proportion, and a narrower one keeps them at their canonical
/// width and overflows, because a narrower card would slide its icon over
/// its value and trend.
const CARD_COLUMNS: [(u16, u16); 4] = [(266, 253), (535, 248), (799, 237), (1052, 237)];
/// The row keeps the canonical distance to the right edge of the viewport.
const ROW_END: u16 = 1536 - 1289;

/// Which card edge a card's content keeps its canonical distance from.
#[derive(Clone, Copy)]
enum Edge {
    Start,
    Both,
    End,
}

pub(super) fn container(elements: &[DashboardElement]) -> DashboardContainer {
    let columns = CARD_COLUMNS
        .map(|(_, width)| MosaicTrack::flex(width, width).expect("card weights are nonzero"));
    let layout = elements
        .iter()
        .filter(|element| {
            element
                .layout_cell
                .is_some_and(|cell| cell.container == ROW)
        })
        .fold(
            MosaicLayoutContract::columns(columns)
                .expect("the card row declares its columns")
                .with_gaps(CARD_GAP, 0),
            |layout, element| {
                let cell = element.layout_cell.expect("filtered to row members").cell;
                layout
                    .with_member(element.component(), cell)
                    .expect("each card member sits once within the row")
            },
        );
    DashboardContainer {
        id: ROW,
        allocation: ComponentAllocationMeasurementContract::viewport_region(
            ComponentViewportRegion::new(
                ComponentViewportAxisPlacement::stretch_between(CARD_COLUMNS[0].0, ROW_END),
                ComponentViewportAxisPlacement::fixed_from_start(ROW_TOP, CARD_HEIGHT)
                    .expect("the card row has height"),
            ),
        ),
        layout: layout.into(),
    }
}

fn card(id: &'static str, column: u16) -> DashboardElement {
    let (x, width) = CARD_COLUMNS[usize::from(column)];
    surface(
        id,
        [x, ROW_TOP, width, CARD_HEIGHT],
        "raised_surface",
        12,
        true,
        2,
    )
    .in_cell(
        ROW,
        MosaicLayoutCell::at(column, 0),
        ComponentViewportRegion::new(
            ComponentViewportAxisPlacement::stretch_between(0, 0),
            ComponentViewportAxisPlacement::stretch_between(0, 0),
        ),
    )
}

/// Places `element` within its card, keeping its canonical padding from the
/// chosen edge of the card.
fn in_card(element: DashboardElement, column: u16, edge: Edge) -> DashboardElement {
    let (card_x, card_width) = CARD_COLUMNS[usize::from(column)];
    let [x, y, width, height] = element.rect;
    let start = x - card_x;
    let end = card_x + card_width - x - width;
    let horizontal = match edge {
        Edge::Start => ComponentViewportAxisPlacement::fixed_from_start(start, width),
        Edge::Both => Some(ComponentViewportAxisPlacement::stretch_between(start, end)),
        Edge::End => ComponentViewportAxisPlacement::fixed_from_end(end, width),
    }
    .expect("card content has width");
    let vertical = ComponentViewportAxisPlacement::fixed_from_start(y - ROW_TOP, height)
        .expect("card content has height");
    element.in_cell(
        ROW,
        MosaicLayoutCell::at(column, 0),
        ComponentViewportRegion::new(horizontal, vertical),
    )
}

pub(super) fn elements() -> Vec<DashboardElement> {
    vec![
        card("active_card", 0),
        in_card(
            text(
                "active_label",
                "Active services",
                [291, 171, 211, 21],
                12,
                false,
                "primary_text",
            ),
            0,
            Edge::Both,
        ),
        in_card(
            text(
                "active_value",
                "24",
                [291, 197, 115, 40],
                28,
                true,
                "primary_text",
            ),
            0,
            Edge::Start,
        ),
        in_card(
            text(
                "active_trend",
                "↑ 2%",
                [356, 208, 80, 26],
                14,
                false,
                "positive",
            ),
            0,
            Edge::Start,
        ),
        in_card(
            surface(
                "active_icon_back",
                [451, 171, 44, 44],
                "mint_pale",
                32,
                false,
                3,
            ),
            0,
            Edge::End,
        ),
        in_card(
            surface("active_icon", [464, 184, 18, 18], "positive", 12, false, 4)
                .graphic(DashboardGraphic::Server),
            0,
            Edge::End,
        ),
        card("query_card", 1),
        in_card(
            text(
                "query_label",
                "Request volume",
                [560, 171, 206, 21],
                12,
                false,
                "primary_text",
            ),
            1,
            Edge::Both,
        ),
        in_card(
            text(
                "request_volume",
                "1.2M",
                [560, 197, 115, 40],
                28,
                true,
                "primary_text",
            ),
            1,
            Edge::Start,
        ),
        in_card(
            text(
                "query_trend",
                "↑ 12%",
                [651, 208, 80, 26],
                14,
                false,
                "positive",
            ),
            1,
            Edge::Start,
        ),
        in_card(
            surface(
                "query_icon_back",
                [715, 171, 44, 44],
                "lavender_pale",
                32,
                false,
                3,
            ),
            1,
            Edge::End,
        ),
        in_card(
            surface(
                "query_icon",
                [728, 184, 18, 18],
                "principal_accent",
                12,
                false,
                4,
            )
            .graphic(DashboardGraphic::Bars),
            1,
            Edge::End,
        ),
        card("error_card", 2),
        in_card(
            text(
                "error_label",
                "Error rate",
                [824, 171, 195, 21],
                12,
                false,
                "primary_text",
            ),
            2,
            Edge::Both,
        ),
        in_card(
            text(
                "error_value",
                "0.28%",
                [824, 197, 115, 40],
                28,
                true,
                "primary_text",
            ),
            2,
            Edge::Start,
        ),
        in_card(
            text(
                "error_trend",
                "↓ 35%",
                [925, 208, 80, 26],
                14,
                false,
                "positive",
            ),
            2,
            Edge::Start,
        ),
        in_card(
            surface(
                "error_icon_back",
                [968, 171, 44, 44],
                "coral_pale",
                32,
                false,
                3,
            ),
            2,
            Edge::End,
        ),
        in_card(
            surface("error_icon", [981, 184, 18, 18], "negative", 12, false, 4)
                .graphic(DashboardGraphic::Warning),
            2,
            Edge::End,
        ),
        card("latency_card", 3),
        in_card(
            text(
                "latency_label",
                "P95 latency",
                [1077, 171, 195, 21],
                12,
                false,
                "primary_text",
            ),
            3,
            Edge::Both,
        ),
        in_card(
            text(
                "latency_value",
                "186 ms",
                [1077, 197, 115, 40],
                28,
                true,
                "primary_text",
            ),
            3,
            Edge::Start,
        ),
        in_card(
            text(
                "latency_trend",
                "↓ 8%",
                [1168, 208, 80, 26],
                14,
                false,
                "caution",
            ),
            3,
            Edge::Start,
        ),
        in_card(
            surface(
                "latency_icon_back",
                [1221, 171, 44, 44],
                "amber_pale",
                32,
                false,
                3,
            ),
            3,
            Edge::End,
        ),
        in_card(
            surface("latency_icon", [1234, 184, 18, 18], "caution", 12, false, 4)
                .graphic(DashboardGraphic::Clock),
            3,
            Edge::End,
        ),
    ]
}

#[cfg(test)]
#[path = "metrics_tests.rs"]
mod tests;
