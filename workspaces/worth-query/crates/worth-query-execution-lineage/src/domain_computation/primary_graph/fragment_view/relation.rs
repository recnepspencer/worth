//! Fragment traversal delegates to the original tracked adjacency read.
use super::{FragmentDenial, ObservedEntity, Reading, View};
use std::marker::PhantomData;
use worth_query_declaration::facade::application_schema::fragment::{
    Entity, Fragment, In, Relation,
};
impl<F: Fragment> View<'_, F, Reading<'_>> {
    #[inline(always)]
    pub fn relations_from<E, R, EP, RP>(
        &mut self,
        from: &ObservedEntity<E>,
        _: R,
    ) -> Result<Vec<ObservedEntity<R::To>>, FragmentDenial>
    where
        E: Entity,
        R: Relation + In<F::Members, (EP, RP), (F, E)>,
    {
        let definition =
            self.definition
                .relation::<E, R>()
                .ok_or(FragmentDenial::MemberNotInstalled {
                    entity: E::NAME,
                    member: R::NAME,
                })?;
        Ok(typed_targets(
            self.access.port.relations_from(&from.erased, definition)?,
        ))
    }
}
fn typed_targets<E>(targets: Vec<super::ErasedObserved>) -> Vec<ObservedEntity<E>> {
    targets
        .into_iter()
        .map(|erased| ObservedEntity {
            erased,
            marker: PhantomData,
        })
        .collect()
}
