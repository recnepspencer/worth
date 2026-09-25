use worth_ui::facade::declaration::ComponentViewportAxisPlacement as Axis;

use super::{container, elements, CARD_COLUMNS, CARD_GAP, ROW_END};

/// Independent oracle: weighted columns share the row width left after the
/// gaps, then each element's region resolves inside its column. Every
/// column's minimum is its weight, so a share below one point per weight
/// holds every column at its minimum instead.
fn expected_rects(viewport_width: f32) -> Vec<[f32; 4]> {
    let row_x = f32::from(CARD_COLUMNS[0].0);
    let row_width = viewport_width - row_x - f32::from(ROW_END);
    let weights = CARD_COLUMNS.map(|(_, width)| f32::from(width));
    let share = ((row_width - 3.0 * f32::from(CARD_GAP)) / weights.iter().sum::<f32>()).max(1.0);
    let resolve = |axis: Axis, available: f32| match axis {
        Axis::FixedFromStart {
            start_logical_points,
            extent_logical_points,
        } => (
            f32::from(start_logical_points),
            f32::from(extent_logical_points),
        ),
        Axis::StretchBetween {
            start_logical_points,
            end_logical_points,
        } => (
            f32::from(start_logical_points),
            available - f32::from(start_logical_points) - f32::from(end_logical_points),
        ),
        Axis::FixedFromEnd {
            end_logical_points,
            extent_logical_points,
        } => (
            available - f32::from(end_logical_points) - f32::from(extent_logical_points),
            f32::from(extent_logical_points),
        ),
    };
    elements()
        .into_iter()
        .map(|element| {
            let cell = element
                .layout_cell
                .expect("every card element is a row member");
            let column = usize::from(cell.cell.column());
            let column_x = row_x
                + weights[..column].iter().sum::<f32>() * share
                + column as f32 * f32::from(CARD_GAP);
            let (x, width) = resolve(cell.region.horizontal(), weights[column] * share);
            let (y, height) = resolve(cell.region.vertical(), 98.0);
            [column_x + x, 152.0 + y, width, height]
        })
        .collect()
}

fn rect_of(rects: &[[f32; 4]], id: &str) -> [f32; 4] {
    let index = elements()
        .iter()
        .position(|element| element.id == id)
        .unwrap();
    rects[index]
}

#[test]
fn the_canonical_extent_reproduces_every_authored_rectangle() {
    for (element, expected) in elements().into_iter().zip(expected_rects(1536.0)) {
        let authored = element.rect.map(f32::from);
        assert_eq!(expected, authored, "{} moved at 1536", element.id);
    }
}

#[test]
fn wider_rows_grow_cards_while_content_keeps_its_padding() {
    let wide = expected_rects(2_048.0);
    let card = rect_of(&wide, "latency_card");
    let label = rect_of(&wide, "latency_label");
    let value = rect_of(&wide, "latency_value");
    let icon = rect_of(&wide, "latency_icon_back");
    assert!(card[2] > 237.0);
    let padding = |measured: f32, authored: f32| (measured - authored).abs() < 1e-3;
    assert!(padding(label[0] - card[0], 25.0));
    assert!(padding(card[0] + card[2] - (label[0] + label[2]), 17.0));
    assert!(padding(value[0] - card[0], 25.0));
    assert!(padding(card[0] + card[2] - (icon[0] + icon[2]), 24.0));
    assert_eq!([value[2], icon[2]], [115.0, 44.0]);
}

#[test]
fn narrower_rows_keep_canonical_cards_and_overflow() {
    for width in [800.0, 1_200.0, 1_535.0] {
        for (element, expected) in elements().into_iter().zip(expected_rects(width)) {
            let authored = element.rect.map(f32::from);
            assert_eq!(expected, authored, "{} moved at {width}", element.id);
        }
    }
}

/// Content anchored to a card's end never moves closer to content anchored
/// to its start than the canonical frame places it.
#[test]
fn card_content_never_crowds_closer_than_canonical() {
    let canonical = expected_rects(1536.0);
    for width in [800.0, 1_200.0, 1_535.0, 1_537.0, 2_048.0, 2_560.0] {
        let resized = expected_rects(width);
        for (start_index, start) in elements().iter().enumerate() {
            for (end_index, end) in elements().iter().enumerate() {
                let (Some(start_cell), Some(end_cell)) = (start.layout_cell, end.layout_cell)
                else {
                    continue;
                };
                let start_anchored =
                    !matches!(start_cell.region.horizontal(), Axis::FixedFromEnd { .. })
                        && start_cell.region.horizontal() != Axis::stretch_between(0, 0);
                if start_cell.cell != end_cell.cell
                    || !start_anchored
                    || !matches!(end_cell.region.horizontal(), Axis::FixedFromEnd { .. })
                {
                    continue;
                }
                let gap = |rects: &[[f32; 4]]| {
                    rects[end_index][0] - (rects[start_index][0] + rects[start_index][2])
                };
                assert!(
                    gap(&resized) >= gap(&canonical) - 1e-3,
                    "{} crowds {} at {width}",
                    end.id,
                    start.id
                );
            }
        }
    }
}

#[test]
fn the_row_declares_every_card_element_as_a_member() {
    let row = container(&elements());
    let layout = row.layout.fallback();
    for element in elements() {
        assert_eq!(
            layout.member_cell(&element.component()),
            element.layout_cell.map(|cell| cell.cell)
        );
    }
    assert!(row.layout.variants().next().is_none());
}
