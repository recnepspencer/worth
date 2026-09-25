//! The summary card row: one flexible layout container whose four cards, and
//! every label, value, and icon inside them, are placed by the row's tracks.
use worth_ui::facade::declaration::{MosaicLayoutCell, MosaicLayoutContract, MosaicTrack};

use super::frame::{Edge, Frame};
use super::page::{self, CARD_ROW_HEIGHT, GAP};
use super::{surface, text, ContainerTracks, DashboardElement, DashboardGraphic};

pub(super) const ROW: &str = "metric_row";
const ROW_TOP: u16 = 152;
/// Each card's origin and width in the concept. Each width is both its
/// column's weight and its minimum: a wider row grows the cards in
/// proportion, and a narrower one keeps them at their concept width and
/// overflows, because a narrower card would slide its icon over its value
/// and trend.
pub(super) const CARD_COLUMNS: [(u16, u16); 4] = [(266, 253), (535, 248), (799, 237), (1052, 237)];

pub(super) fn container() -> ContainerTracks {
    let columns = CARD_COLUMNS
        .map(|(_, width)| MosaicTrack::flex(width, width).expect("card weights are nonzero"));
    ContainerTracks {
        id: ROW,
        placement: page::card_row(),
        tracks: MosaicLayoutContract::columns(columns)
            .expect("the card row declares its columns")
            .with_gaps(GAP, 0),
        scroll_panel: None,
    }
}

/// The box of the card in `column`, as the concept draws it.
const fn card_frame(column: u16) -> Frame {
    let (x, width) = CARD_COLUMNS[column as usize];
    Frame::cell(
        ROW,
        MosaicLayoutCell::at(column, 0),
        [x, ROW_TOP, width, CARD_ROW_HEIGHT],
    )
}

fn card(id: &'static str, column: u16) -> DashboardElement {
    let (x, width) = CARD_COLUMNS[usize::from(column)];
    card_frame(column).place(
        surface(
            id,
            [x, ROW_TOP, width, CARD_ROW_HEIGHT],
            "raised_surface",
            12,
            true,
            2,
        ),
        Edge::Both,
        Edge::Both,
    )
}

/// Places `element` within its card, keeping its concept padding from the
/// chosen edge of the card and from the card's top.
fn in_card(element: DashboardElement, column: u16, edge: Edge) -> DashboardElement {
    card_frame(column).place(element, edge, Edge::Start)
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
