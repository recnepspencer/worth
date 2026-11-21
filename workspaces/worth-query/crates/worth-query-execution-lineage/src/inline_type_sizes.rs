//! Inline carrier budgets retain layout evidence without constructing owner state.
pub(crate) struct Census {
    exceeded: Vec<(&'static str, usize, usize)>,
}
pub(crate) struct Owner<'a> {
    census: &'a mut Census,
    file: &'static str,
    name: &'static str,
    ceiling: usize,
}
impl Census {
    pub(crate) fn owner<T>(
        &mut self,
        file: &'static str,
        name: &'static str,
        ceiling: usize,
    ) -> Owner<'_> {
        let mut owner = Owner {
            census: self,
            file,
            name,
            ceiling,
        };
        owner.field::<T>("");
        owner
    }
}
impl Owner<'_> {
    pub(crate) fn field<T>(&mut self, field: &'static str) {
        let size = std::mem::size_of::<T>();
        if size > self.ceiling {
            self.census
                .exceeded
                .push((std::any::type_name::<T>(), size, self.ceiling));
        }
    }
}
#[test]
fn lineage_owned_carriers_stay_within_declared_inline_ceilings() {
    let mut census = Census {
        exceeded: Vec::new(),
    };
    crate::domain_computation::application_aftermath::causal_fact::inline_type_budget::collect(&mut census);
    crate::domain_computation::authorization::denial::inline_type_budget::collect(&mut census);
    crate::domain_computation::authorization::operation_scope_binding::inline_type_budget::collect(&mut census);
    crate::domain_computation::product_publication::unpublished::inline_type_budget::collect(&mut census);
    crate::domain_computation::primary_graph::application_attempt::compare_and_commit::commit_outcome::inline_type_budget::collect(&mut census);
    crate::domain_computation::primary_graph::application_attempt::compare_and_commit::commit_receipt::inline_type_budget::collect(&mut census);
    crate::domain_computation::primary_graph::application_attempt::denial::inline_type_budget::collect(&mut census);
    crate::domain_computation::primary_graph::application_attempt::idempotency::inline_type_budget::collect(&mut census);
    crate::domain_computation::primary_graph::application_contribution::computation::inline_type_budget::collect(&mut census);
    crate::domain_computation::primary_graph::application_contribution::producer::demand::denial::inline_type_budget::collect(&mut census);
    crate::domain_computation::primary_graph::application_output_demand::registry::inline_type_budget::collect(&mut census);
    crate::domain_computation::primary_graph::application_output_demand::source_replay::inline_type_budget::collect(&mut census);
    assert!(
        census.exceeded.is_empty(),
        "inline type ceiling exceeded: {:?}",
        census.exceeded
    );
}
