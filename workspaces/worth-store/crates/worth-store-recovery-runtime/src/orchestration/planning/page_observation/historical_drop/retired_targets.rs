//! Targets the ordered history wrote and a later ordinary edge retired.

use worth_store_recovery_physics::PhysicalRedoTarget;

use super::super::{AbsentTarget, SelectedFrontier};
use super::ordered_retirements::OrderedRetirements;

/// Splits the targets into the observations of the retired ones and the ones
/// left to allocation truth.
///
/// A target is retired only when the selected root already allocated it,
/// Store's own reading of the ordered retirements finds its last image
/// emptied, and physics mints the witness for that image. Store decides
/// before physics is asked, and neither reading stands in for the other.
pub(super) fn classify<'target, Observation>(
    targets: Vec<AbsentTarget<'target>>,
    frontier: SelectedFrontier,
    store: &OrderedRetirements,
    mut physics: impl FnMut(&PhysicalRedoTarget) -> Option<Observation>,
) -> (Vec<Observation>, Vec<AbsentTarget<'target>>) {
    let mut retired = Vec::new();
    let mut absent = Vec::new();
    for target in targets {
        let last = target.last();
        let observed = (target.allocated_under(frontier) && store.emptied(last))
            .then(|| physics(last))
            .flatten();
        match observed {
            Some(observation) => retired.push(observation),
            None => absent.push(target),
        }
    }
    (retired, absent)
}

#[cfg(test)]
mod tests {
    use super::super::retirement_fixture::{image, record, retirement, step, PAGE};
    use super::*;
    use worth_store_recovery_physics::PhysicalRedoTargetIdentity;

    const ALLOCATED: SelectedFrontier = SelectedFrontier {
        next_page: PAGE + 1,
        next_extent: 0,
    };

    /// Store's reading of a page whose last image holds records 1 and 2,
    /// retired by an ordinary edge only when `bound`.
    fn store(bound: bool) -> (OrderedRetirements, [PhysicalRedoTarget; 2]) {
        let (older, first) = image(1, 1);
        let (written, last) = image(2, 2);
        let retiring = retirement(&[1, 2]);
        let members = [(step(1), &older), (step(2), &written), (step(3), &retiring)];
        let edges = [step(1), step(2), step(3)];
        let index = OrderedRetirements::index(
            members.into_iter(),
            edges.into_iter().take(if bound { 3 } else { 2 }),
            [record(9)].into_iter(),
        );
        (index, [first, last])
    }

    fn page<'a>(images: &'a [PhysicalRedoTarget; 2]) -> Vec<AbsentTarget<'a>> {
        vec![AbsentTarget::inline_page(images.iter().collect()).unwrap()]
    }

    #[test]
    fn a_retired_target_is_classified_by_its_last_image() {
        let (index, images) = store(true);
        let (retired, absent) = classify(page(&images), ALLOCATED, &index, |target| {
            Some(target.identity())
        });
        assert_eq!(retired, vec![images[1].identity()]);
        assert!(absent.is_empty());
        assert!(matches!(
            images[1].identity(),
            PhysicalRedoTargetIdentity::InlinePage { generation: 2, .. }
        ));
    }

    #[test]
    fn store_denies_a_target_physics_would_accept() {
        let (index, images) = store(false);
        let mut asked = 0;
        let (retired, absent) = classify(page(&images), ALLOCATED, &index, |target| {
            asked += 1;
            Some(target.identity())
        });
        assert!(retired.is_empty());
        assert_eq!(absent.len(), 1);
        assert_eq!(asked, 0, "physics is asked only after Store agrees");
    }

    #[test]
    fn a_target_above_the_selected_frontier_is_never_retired() {
        let (index, images) = store(true);
        let fresh = SelectedFrontier {
            next_page: PAGE,
            next_extent: u64::MAX,
        };
        let (retired, absent) = classify(page(&images), fresh, &index, |target| {
            Some(target.identity())
        });
        assert!(retired.is_empty());
        assert_eq!(absent.len(), 1);
    }

    #[test]
    fn a_target_physics_denies_is_left_to_allocation_truth() {
        let (index, images) = store(true);
        let (retired, absent) = classify(page(&images), ALLOCATED, &index, |_| None::<()>);
        assert!(retired.is_empty());
        assert_eq!(absent[0].first().identity(), images[0].identity());
    }
}
