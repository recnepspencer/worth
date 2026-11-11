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
        println!(
            "TYPE_SIZE|{}|{}|{}|{}|{}|{}",
            self.file,
            self.name,
            field,
            std::any::type_name::<T>(),
            size,
            self.ceiling
        );
        if size > self.ceiling {
            self.census
                .exceeded
                .push((std::any::type_name::<T>(), size, self.ceiling));
        }
    }
}
#[test]
fn owned_carriers_stay_within_declared_inline_ceilings() {
    let mut census = Census {
        exceeded: Vec::new(),
    };
    crate::inline_type_budget::collect(&mut census);
    assert!(
        census.exceeded.is_empty(),
        "inline type ceiling exceeded: {:?}",
        census.exceeded
    );
}
