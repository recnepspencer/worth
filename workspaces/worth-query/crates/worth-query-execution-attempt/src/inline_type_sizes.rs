//! Inline carrier budgets retain layout evidence without constructing owner state.
pub(crate) struct Census {
    exceeded: Vec<(&'static str, usize, usize)>,
}
pub(crate) struct Owner<'a> {
    census: &'a mut Census,
    _file: &'static str,
    _name: &'static str,
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
            _file: file,
            _name: name,
            ceiling,
        };
        owner.field::<T>("");
        owner
    }
}
impl Owner<'_> {
    pub(crate) fn field<T>(&mut self, _field: &'static str) {
        let size = std::mem::size_of::<T>();
        if size > self.ceiling {
            self.census
                .exceeded
                .push((std::any::type_name::<T>(), size, self.ceiling));
        }
    }
}
#[test]
fn attempt_owned_carriers_stay_within_declared_inline_ceilings() {
    let mut census = Census {
        exceeded: Vec::new(),
    };
    crate::domain_computation::application_aftermath::inline_type_budget::collect(&mut census);
    crate::domain_computation::authorization::inline_type_budget::collect(&mut census);
    crate::domain_computation::primary_graph::application_attempt::inline_type_budget::collect(&mut census);
    crate::domain_computation::primary_graph::provider::inline_type_budget::collect(&mut census);
    assert!(
        census.exceeded.is_empty(),
        "inline type ceiling exceeded: {:?}",
        census.exceeded
    );
}
