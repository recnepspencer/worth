use worth_ui::facade::declaration::ComponentViewportAxisPlacement as Axis;

use super::super::dashboard_containers;
use super::super::declared_layout::{assert_near, Declared, Rect, EXTENTS};
use super::{container, elements, CARD_COLUMNS, ROW};

const CARDS: [&str; 4] = ["active_card", "query_card", "error_card", "latency_card"];

/// Independent oracle: the cards share the row width left after the 20-point
/// gaps in proportion to their concept widths, and a row too narrow for that
/// holds every card at its concept width and overflows.
fn expected_cards(row: Rect) -> [Rect; 4] {
    let weights = CARD_COLUMNS.map(|(_, width)| f32::from(width));
    let share = ((row[2] - 60.0) / weights.iter().sum::<f32>()).max(1.0);
    let mut x = row[0];
    weights.map(|weight| {
        let card = [x, row[1], weight * share, row[3]];
        x += weight * share + 20.0;
        card
    })
}

#[test]
fn cards_share_the_row_by_their_concept_widths() {
    for (width, height) in EXTENTS
        .into_iter()
        .chain([(800.0, 600.0), (2560.0, 1440.0)])
    {
        let declared = Declared::at(width, height);
        let row = declared.container(ROW);
        for (id, expected) in CARDS.into_iter().zip(expected_cards(row)) {
            assert_near(declared.element(id), expected, &format!("{id} at {width}"));
        }
    }
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
            match element.placement.layout_cell().unwrap().region.horizontal() {
                Axis::FixedFromEnd { .. } => {
                    let end = card[0] + card[2] - placed[0] - placed[2];
                    assert!((end - from_end).abs() < 1e-3, "{at} end");
                }
                Axis::StretchBetween { .. } => {
                    assert!(
                        (placed[0] - card[0] - from_start).abs() < 1e-3,
                        "{at} start"
                    );
                    let end = card[0] + card[2] - placed[0] - placed[2];
                    assert!((end - from_end).abs() < 1e-3, "{at} end");
                }
                Axis::FixedFromStart { .. } => {
                    assert!(
                        (placed[0] - card[0] - from_start).abs() < 1e-3,
                        "{at} start"
                    );
                }
            }
        }
    }
}

/// Content anchored to a card's end never moves closer to content anchored
/// to its start than the concept draws it.
#[test]
fn card_content_never_crowds_closer_than_the_concept() {
    for width in [800.0, 1_200.0, 1_535.0, 1_537.0, 2_048.0, 2_560.0] {
        let resized = Declared::at(width, 1024.0);
        for start in elements() {
            for end in elements() {
                let (Some(start_cell), Some(end_cell)) =
                    (start.placement.layout_cell(), end.placement.layout_cell())
                else {
                    continue;
                };
                if start_cell.cell != end_cell.cell
                    || !matches!(start_cell.region.horizontal(), Axis::FixedFromStart { .. })
                    || !matches!(end_cell.region.horizontal(), Axis::FixedFromEnd { .. })
                {
                    continue;
                }
                let placed = resized.element(start.id);
                let gap = resized.element(end.id)[0] - (placed[0] + placed[2]);
                let concept = f32::from(end.rect[0]) - f32::from(start.rect[0] + start.rect[2]);
                assert!(
                    gap >= concept - 1e-3,
                    "{} crowds {} at {width}",
                    end.id,
                    start.id
                );
            }
        }
    }
}

#[test]
fn the_row_declares_exactly_the_card_elements_as_members() {
    assert_eq!(container().id, ROW);
    let row = dashboard_containers()
        .into_iter()
        .find(|container| container.id == ROW)
        .expect("the card row is a container");
    let layout = row.layout.fallback();
    assert_eq!(layout.members().count(), elements().len());
    for element in elements() {
        assert_eq!(
            layout.member_cell(&element.component()),
            element.placement.layout_cell().map(|cell| cell.cell)
        );
    }
    assert!(row.layout.variants().next().is_none());
}
