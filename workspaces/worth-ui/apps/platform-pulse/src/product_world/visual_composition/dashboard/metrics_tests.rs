use worth_ui::facade::declaration::{ComponentViewportAxisPlacement as Axis, MosaicLayoutCell};

use super::super::dashboard_containers;
use super::super::declared_layout::{assert_near, Declared, Rect, EXTENTS};
use super::super::page::{BREAKPOINT, PAGE, PLATFORM_PULSE_SIDEBAR_WIDTH};
use super::{container, elements, CARD_COLUMNS, CLEARANCE, ROW};

const CARDS: [&str; 4] = ["active_card", "query_card", "error_card", "latency_card"];

/// Each card's least width, read off its content: the wider of its label and
/// its value with the trend 6 points after it, from a 25-point inset; a
/// 6-point clearance; and the 44-point icon 24 points from the card's end.
/// Active services: 25 + 84 = 109. Request volume: 25 + 65 + 6 + 40 = 136.
/// Error rate: 25 + 84 + 6 + 40 = 155. P95 latency: 25 + 93 + 6 + 32 = 156.
const MINIMUMS: [f32; 4] = [183.0, 210.0, 229.0, 230.0];

/// Shares `available` by weight, holding each share below its minimum at
/// the minimum and sharing the rest among the others.
fn held_shares<const N: usize>(available: f32, weights: [f32; N], minimums: [f32; N]) -> [f32; N] {
    let mut held = [false; N];
    loop {
        let fixed = (0..N)
            .filter(|&i| held[i])
            .map(|i| minimums[i])
            .sum::<f32>();
        let weight = (0..N)
            .filter(|&i| !held[i])
            .map(|i| weights[i])
            .sum::<f32>();
        let shares = std::array::from_fn(|i| match held[i] {
            true => minimums[i],
            false => (available - fixed) * weights[i] / weight,
        });
        let short = (0..N).filter(|&i| !held[i] && shares[i] < minimums[i]);
        let short = short.collect::<Vec<_>>();
        if short.is_empty() {
            return shares;
        }
        for i in short {
            held[i] = true;
        }
    }
}

/// Independent oracle. From 1200 points wide the cards share the row left
/// after the 20-point gaps in proportion to their concept widths, each held
/// at its least width. Below it they sit two by two, 20 points apart, each
/// column as wide as the wider least width it holds.
fn expected_cards(width: f32, row: Rect) -> [Rect; 4] {
    let [x, y, across, _] = row;
    if width < 1200.0 {
        let [first, second] = held_shares(
            across - 20.0,
            [1.0, 1.0],
            [MINIMUMS[0].max(MINIMUMS[2]), MINIMUMS[1].max(MINIMUMS[3])],
        );
        let second_x = x + first + 20.0;
        return [
            [x, y, first, 98.0],
            [second_x, y, second, 98.0],
            [x, y + 118.0, first, 98.0],
            [second_x, y + 118.0, second, 98.0],
        ];
    }
    let weights = CARD_COLUMNS.map(|(_, width)| f32::from(width));
    let mut left = x;
    held_shares(across - 60.0, weights, MINIMUMS).map(|extent| {
        let card = [left, y, extent, 98.0];
        left += extent + 20.0;
        card
    })
}

#[test]
fn cards_share_the_row_by_weight_and_hold_their_least_width() {
    for (width, height) in EXTENTS.into_iter().chain([(2560.0, 1440.0)]) {
        let declared = Declared::at(width, height);
        let row = declared.container(ROW);
        for (id, expected) in CARDS.into_iter().zip(expected_cards(width, row)) {
            assert_near(declared.element(id), expected, &format!("{id} at {width}"));
        }
    }
}

/// At the breakpoint the four cards fit the page's content width: the row
/// fills its cell and the last card ends at the gutter.
#[test]
fn four_cards_fit_inside_the_page_at_the_breakpoint() {
    let declared = Declared::at(1200.0, 800.0);
    assert_near(declared.container(ROW), [260.0, 167.0, 916.0, 98.0], "row");
    let last = declared.element("latency_card");
    assert!((last[0] + last[2] - 1176.0).abs() < 1e-3, "{last:?}");
}

/// Every card element keeps its concept distance from its card's top and
/// from the card edge it is anchored to, at every width.
#[test]
fn card_content_keeps_its_concept_padding() {
    for width in [800.0, 1_200.0, 1_536.0, 2_048.0] {
        let declared = Declared::at(width, 1024.0);
        for element in elements() {
            let column = usize::from(element.placement.layout_cell().unwrap().cell.column());
            let (concept_x, concept_width) = CARD_COLUMNS[column];
            let card = declared.element(CARDS[column]);
            let placed = declared.element(element.id);
            let [x, y, w, _] = element.rect.map(f32::from);
            let from_start = x - f32::from(concept_x);
            let from_end = f32::from(concept_x + concept_width) - x - w;
            let at = format!("{} at {width}", element.id);
            assert!((placed[1] - card[1] - (y - 152.0)).abs() < 1e-3, "{at} top");
            let start = || {
                assert!(
                    (placed[0] - card[0] - from_start).abs() < 1e-3,
                    "{at} start"
                );
            };
            let end = || {
                let end = card[0] + card[2] - placed[0] - placed[2];
                assert!((end - from_end).abs() < 1e-3, "{at} end");
            };
            match element.placement.layout_cell().unwrap().region.horizontal() {
                Axis::FixedFromEnd { .. } => end(),
                Axis::StretchBetween { .. } => {
                    start();
                    end();
                }
                Axis::FixedFromStart { .. } => start(),
            }
        }
    }
}

/// Content anchored to a card's end never comes closer than the clearance
/// to content anchored to its start. At the breakpoint the three cards held
/// at their least width stand exactly the clearance apart.
#[test]
fn card_icons_keep_their_clearance_from_the_text() {
    for width in [800.0, 1_120.0, 1_199.0, 1_200.0, 1_201.0, 1_536.0, 2_560.0] {
        let declared = Declared::at(width, 1024.0);
        let mut tightest = [f32::INFINITY; 4];
        for start in elements() {
            for end in elements() {
                let (start_cell, end_cell) = (
                    start.placement.layout_cell().unwrap(),
                    end.placement.layout_cell().unwrap(),
                );
                if start_cell.cell != end_cell.cell
                    || !matches!(start_cell.region.horizontal(), Axis::FixedFromStart { .. })
                    || !matches!(end_cell.region.horizontal(), Axis::FixedFromEnd { .. })
                {
                    continue;
                }
                let placed = declared.element(start.id);
                let gap = declared.element(end.id)[0] - (placed[0] + placed[2]);
                let column = usize::from(start_cell.cell.column());
                tightest[column] = tightest[column].min(gap);
            }
        }
        for (card, gap) in CARDS.into_iter().zip(tightest) {
            assert!(
                gap >= f32::from(CLEARANCE) - 1e-3,
                "{card} crowds its text to {gap} at {width}"
            );
        }
        if width == 1_200.0 {
            for gap in &tightest[1..] {
                assert!((gap - f32::from(CLEARANCE)).abs() < 1e-3, "{tightest:?}");
            }
        }
    }
}

/// A container's minimum does not reach the page it sits in, so the page
/// must reserve room for the cards itself. Stacked, the row's least width
/// fits the page's single column at its minimum. From the breakpoint, the
/// row's least width fits the page's content at the breakpoint.
#[test]
fn the_card_row_minimums_fit_the_page_that_carries_them() {
    let layouts = |id| {
        let container = dashboard_containers()
            .into_iter()
            .find(|container| container.id == id)
            .expect("a dashboard container");
        let wide = container.layout.select(f32::from(BREAKPOINT)).clone();
        (container.layout.fallback().clone(), wide)
    };
    let (row_stacked, row_wide) = layouts(ROW);
    let (page_stacked, page_wide) = layouts(PAGE);
    let gutters = |page: &worth_ui::facade::declaration::MosaicLayoutContract| {
        2 * u32::from(page.inline_padding_logical_points())
    };
    assert!(
        row_stacked.minimum_width_logical_points()
            <= page_stacked.minimum_width_logical_points() - gutters(&page_stacked),
        "the stacked cards outgrow the stacked page column"
    );
    let content = u32::from(BREAKPOINT - PLATFORM_PULSE_SIDEBAR_WIDTH) - gutters(&page_wide);
    assert!(
        row_wide.minimum_width_logical_points() <= content,
        "the cards outgrow the page at the breakpoint"
    );
}

#[test]
fn the_row_declares_exactly_the_card_elements_as_members() {
    assert_eq!(container().id, ROW);
    let row = dashboard_containers()
        .into_iter()
        .find(|container| container.id == ROW)
        .expect("the card row is a container");
    let stacked = row.layout.fallback();
    let [(interval, wide)] = row.layout.variants().collect::<Vec<_>>()[..] else {
        panic!("the card row declares one wide layout");
    };
    assert_eq!(interval.min_logical_points(), 1200);
    assert_eq!(interval.max_logical_points(), None);
    assert_eq!(stacked.members().count(), elements().len());
    assert_eq!(wide.members().count(), elements().len());
    for element in elements() {
        let cell = element.placement.layout_cell().unwrap().cell;
        assert_eq!(wide.member_cell(&element.component()), Some(cell));
        assert_eq!(
            stacked.member_cell(&element.component()),
            Some(MosaicLayoutCell::at(cell.column() % 2, cell.column() / 2))
        );
    }
}
