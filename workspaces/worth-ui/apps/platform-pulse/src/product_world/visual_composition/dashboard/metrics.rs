//! The summary card row: one flexible layout container whose four cards, and
//! every label, value, and icon inside them, are placed by the row's tracks.
//!
//! From the breakpoint up the cards share one row; below it they sit two by
//! two. Each card's text keeps its concept inset from the card's start and
//! its icon from the card's end. A card is never narrower than the width that
//! keeps its icon clear of its text, so a narrow row overflows instead of
//! sliding the icon over a value.
use worth_ui::facade::declaration::{
    ComponentViewportAxisPlacement as Axis, MosaicLayoutCell, MosaicLayoutContract, MosaicTrack,
};

use super::frame::{Edge, Frame};
use super::page::{self, CARD_ROW_HEIGHT, GAP};
use super::{surface, text, ContainerTracks, DashboardElement, DashboardGraphic, StackedTracks};

pub(super) const ROW: &str = "metric_row";
const ROW_TOP: u16 = 152;
/// Each card's origin and width in the concept. Each width is its column's
/// weight, so a wider row grows the cards in proportion.
pub(super) const CARD_COLUMNS: [(u16, u16); 4] = [(266, 253), (535, 248), (799, 237), (1052, 237)];
/// How far the label and the value stand from the card's start.
const TEXT_INSET: u16 = 25;
/// The space between a value and the trend that follows it.
const TREND_GAP: u16 = 6;
/// The least space a card keeps between its text and its icon.
pub(super) const CLEARANCE: u16 = 6;

/// One summary card's content. Each text box is as wide as its text set in
/// the bundled fonts at weight 500, with room to spare: Roboto 12 for the
/// label, Gelasio 28 for the value, and Roboto 14 for the trend, whose arrow
/// falls back to another face.
struct Metric {
    card: &'static str,
    label: (&'static str, &'static str, u16),
    value: (&'static str, &'static str, u16),
    trend: (&'static str, &'static str, u16, &'static str),
    icon_back: (&'static str, &'static str),
    icon: (&'static str, &'static str, DashboardGraphic),
}

const METRICS: [Metric; 4] = [
    Metric {
        card: "active_card",
        label: ("active_label", "Active services", 84),
        value: ("active_value", "24", 35),
        trend: ("active_trend", "↑ 2%", 32, "positive"),
        icon_back: ("active_icon_back", "mint_pale"),
        icon: ("active_icon", "positive", DashboardGraphic::Server),
    },
    Metric {
        card: "query_card",
        label: ("query_label", "Request volume", 89),
        value: ("request_volume", "1.2M", 65),
        trend: ("query_trend", "↑ 12%", 40, "positive"),
        icon_back: ("query_icon_back", "lavender_pale"),
        icon: ("query_icon", "principal_accent", DashboardGraphic::Bars),
    },
    Metric {
        card: "error_card",
        label: ("error_label", "Error rate", 53),
        value: ("error_value", "0.28%", 84),
        trend: ("error_trend", "↓ 35%", 40, "positive"),
        icon_back: ("error_icon_back", "coral_pale"),
        icon: ("error_icon", "negative", DashboardGraphic::Warning),
    },
    Metric {
        card: "latency_card",
        label: ("latency_label", "P95 latency", 66),
        value: ("latency_value", "186 ms", 93),
        trend: ("latency_trend", "↓ 8%", 32, "caution"),
        icon_back: ("latency_icon_back", "amber_pale"),
        icon: ("latency_icon", "caution", DashboardGraphic::Clock),
    },
];

pub(super) fn container() -> ContainerTracks {
    let minimums = [0, 1, 2, 3].map(card_minimum);
    let columns = CARD_COLUMNS
        .iter()
        .zip(minimums)
        .map(|(&(_, width), minimum)| {
            MosaicTrack::flex(width, minimum).expect("card weights are nonzero")
        });
    let pair = |first: u16, second: u16| {
        MosaicTrack::flex(
            1,
            minimums[usize::from(first)].max(minimums[usize::from(second)]),
        )
        .expect("a stacked column has weight")
    };
    let row = || MosaicTrack::fixed(CARD_ROW_HEIGHT).expect("a card row has height");
    ContainerTracks {
        id: ROW,
        placement: page::card_row(),
        tracks: MosaicLayoutContract::columns(columns)
            .expect("the card row declares its columns")
            .with_gaps(GAP, 0),
        stacked: Some(StackedTracks {
            tracks: MosaicLayoutContract::grid([pair(0, 2), pair(1, 3)], [row(), row()])
                .expect("the stacked cards declare their tracks")
                .with_gaps(GAP, GAP),
            cell: |cell| MosaicLayoutCell::at(cell.column() % 2, cell.column() / 2),
        }),
        scroll_owner: None,
    }
}

/// The narrowest the card in `column` can be: its text at its inset from
/// the start, its icon at its inset from the end, and the clearance between.
fn card_minimum(column: u16) -> u16 {
    let (x, width) = CARD_COLUMNS[usize::from(column)];
    let (mut from_start, mut from_end) = (0, 0);
    for element in elements() {
        let Some(placed) = element.placement.layout_cell() else {
            continue;
        };
        if placed.cell != MosaicLayoutCell::at(column, 0) {
            continue;
        }
        let [left, _, extent, _] = element.rect;
        match placed.region.horizontal() {
            Axis::FixedFromStart { .. } => from_start = from_start.max(left + extent - x),
            Axis::FixedFromEnd { .. } => from_end = from_end.max(x + width - left),
            Axis::StretchBetween { .. } => {}
        }
    }
    from_start + CLEARANCE + from_end
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

/// One card's surface, filling its cell, and its content: the label, value,
/// and trend kept from the card's start, and the icon kept from its end.
fn card(column: u16, metric: &Metric) -> [DashboardElement; 6] {
    let (x, width) = CARD_COLUMNS[usize::from(column)];
    let frame = card_frame(column);
    let start = |element| frame.place(element, Edge::Start, Edge::Start);
    let end = |element| frame.place(element, Edge::End, Edge::Start);
    let (label, label_text, label_width) = metric.label;
    let (value, value_text, value_width) = metric.value;
    let (trend, trend_text, trend_width, trend_color) = metric.trend;
    let (icon_back, icon_back_color) = metric.icon_back;
    let (icon, icon_color, graphic) = metric.icon;
    let text_x = x + TEXT_INSET;
    let icon_x = x + width - 68;
    [
        frame.place(
            surface(
                metric.card,
                [x, ROW_TOP, width, CARD_ROW_HEIGHT],
                "raised_surface",
                12,
                true,
                2,
            ),
            Edge::Both,
            Edge::Both,
        ),
        start(text(
            label,
            label_text,
            [text_x, 171, label_width, 21],
            12,
            false,
            "primary_text",
        )),
        start(text(
            value,
            value_text,
            [text_x, 197, value_width, 40],
            28,
            true,
            "primary_text",
        )),
        start(text(
            trend,
            trend_text,
            [text_x + value_width + TREND_GAP, 208, trend_width, 26],
            14,
            false,
            trend_color,
        )),
        end(surface(
            icon_back,
            [icon_x, 171, 44, 44],
            icon_back_color,
            32,
            false,
            3,
        )),
        end(surface(icon, [icon_x + 13, 184, 18, 18], icon_color, 12, false, 4).graphic(graphic)),
    ]
}

pub(super) fn elements() -> Vec<DashboardElement> {
    METRICS
        .iter()
        .zip(0..)
        .flat_map(|(metric, column)| card(column, metric))
        .collect()
}

#[cfg(test)]
#[path = "metrics_tests.rs"]
mod tests;
