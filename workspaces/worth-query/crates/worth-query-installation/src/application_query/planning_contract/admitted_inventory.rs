use super::{
    views::{guard_view, ordering_view, predicate_view, result_relation_view, root_step_view},
    WorthQueryReadGraphGuardView, WorthQueryReadGraphOrderingView,
    WorthQueryReadGraphPredicateView, WorthQueryReadGraphRelationView,
};
use crate::application_query::graph_access_contract::WorthQueryInstalledGraphReadMeaning;

#[derive(Debug)]
pub enum WorthQueryPlanningInventoryStop<Stop> {
    Admission(Stop),
    AccountingOverflow,
}

pub struct WorthQueryAdmittedReadGraphPlanningInventory<'a> {
    meaning: &'a WorthQueryInstalledGraphReadMeaning,
    root_step_count: usize,
    relation_count: usize,
    relation_predicate_count: usize,
    predicate_count: usize,
    guard_count: usize,
}

impl<'a> WorthQueryAdmittedReadGraphPlanningInventory<'a> {
    pub(super) fn prepare<Stop>(
        meaning: &'a WorthQueryInstalledGraphReadMeaning,
        admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<Self, WorthQueryPlanningInventoryStop<Stop>> {
        let count_visits = checked_visits(&[
            (meaning.root_paths.len(), 2),
            (meaning.relations.len(), 1),
            (1, 2),
        ])?;
        admit(count_visits, 0).map_err(WorthQueryPlanningInventoryStop::Admission)?;
        let mut root_step_count = 0usize;
        let mut guard_count = 0usize;
        for path in &meaning.root_paths {
            root_step_count = root_step_count
                .checked_add(path.steps().len())
                .ok_or(WorthQueryPlanningInventoryStop::AccountingOverflow)?;
            guard_count = guard_count
                .checked_add(path.guards().len())
                .ok_or(WorthQueryPlanningInventoryStop::AccountingOverflow)?;
        }
        let relation_predicate_count = meaning
            .relations
            .iter()
            .filter(|relation| relation.predicate().is_some())
            .count();
        let relation_count = root_step_count
            .checked_add(meaning.relations.len())
            .ok_or(WorthQueryPlanningInventoryStop::AccountingOverflow)?;
        let predicate_count = meaning
            .predicates
            .len()
            .checked_add(relation_predicate_count)
            .ok_or(WorthQueryPlanningInventoryStop::AccountingOverflow)?;
        Ok(Self {
            meaning,
            root_step_count,
            relation_count,
            relation_predicate_count,
            predicate_count,
            guard_count,
        })
    }

    pub const fn relation_count(&self) -> usize {
        self.relation_count
    }

    pub const fn predicate_count(&self) -> usize {
        self.predicate_count
    }

    pub const fn guard_count(&self) -> usize {
        self.guard_count
    }

    pub fn ordering_count(&self) -> usize {
        self.meaning.ordering.len()
    }

    pub fn relations_admitted<Stop>(
        &self,
        admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<
        impl Iterator<Item = WorthQueryReadGraphRelationView<'a>> + 'a,
        WorthQueryPlanningInventoryStop<Stop>,
    > {
        let visits = checked_visits(&[
            (self.meaning.root_paths.len(), 1),
            (self.root_step_count, 5),
            (self.meaning.relations.len(), 5),
            (1, 2),
        ])?;
        admit(visits, 0).map_err(WorthQueryPlanningInventoryStop::Admission)?;
        let meaning = self.meaning;
        Ok(meaning
            .root_paths
            .iter()
            .flat_map(|path| path.steps().iter().map(root_step_view))
            .chain(meaning.relations.iter().map(result_relation_view)))
    }

    pub fn predicates_admitted<Stop>(
        &self,
        admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<
        impl Iterator<Item = WorthQueryReadGraphPredicateView<'a>> + 'a,
        WorthQueryPlanningInventoryStop<Stop>,
    > {
        let visits = checked_visits(&[
            (self.meaning.predicates.len(), 4),
            (self.meaning.relations.len(), 1),
            (self.relation_predicate_count, 4),
            (1, 2),
        ])?;
        admit(visits, 0).map_err(WorthQueryPlanningInventoryStop::Admission)?;
        let meaning = self.meaning;
        Ok(meaning
            .predicates
            .iter()
            .chain(
                meaning
                    .relations
                    .iter()
                    .filter_map(|relation| relation.predicate()),
            )
            .map(predicate_view))
    }

    pub fn guards_admitted<Stop>(
        &self,
        admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<
        impl Iterator<Item = WorthQueryReadGraphGuardView<'a>> + 'a,
        WorthQueryPlanningInventoryStop<Stop>,
    > {
        let visits = checked_visits(&[
            (self.meaning.root_paths.len(), 1),
            (self.guard_count, 7),
            (1, 2),
        ])?;
        admit(visits, 0).map_err(WorthQueryPlanningInventoryStop::Admission)?;
        let meaning = self.meaning;
        Ok(meaning
            .root_paths
            .iter()
            .flat_map(|path| path.guards().iter())
            .map(guard_view))
    }

    pub fn orderings_admitted<Stop>(
        &self,
        admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<
        impl Iterator<Item = WorthQueryReadGraphOrderingView<'a>> + 'a,
        WorthQueryPlanningInventoryStop<Stop>,
    > {
        let visits = checked_visits(&[(self.meaning.ordering.len(), 6), (1, 1)])?;
        admit(visits, 0).map_err(WorthQueryPlanningInventoryStop::Admission)?;
        let meaning = self.meaning;
        Ok(meaning.ordering.iter().map(ordering_view))
    }
}

fn checked_visits<Stop>(
    parts: &[(usize, u64)],
) -> Result<u64, WorthQueryPlanningInventoryStop<Stop>> {
    parts.iter().try_fold(0u64, |total, (count, per_row)| {
        let count = u64::try_from(*count)
            .map_err(|_| WorthQueryPlanningInventoryStop::AccountingOverflow)?;
        total
            .checked_add(
                count
                    .checked_mul(*per_row)
                    .ok_or(WorthQueryPlanningInventoryStop::AccountingOverflow)?,
            )
            .ok_or(WorthQueryPlanningInventoryStop::AccountingOverflow)
    })
}
