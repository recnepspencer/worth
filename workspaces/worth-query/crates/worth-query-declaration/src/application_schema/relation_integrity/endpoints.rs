#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationRelationCrossContextPolicy {
    AllowExplicit,
    SchemaControlled,
    Forbid,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationRelationEndpoints {
    pub self_edges_allowed: bool,
    pub cross_context_policy: ApplicationRelationCrossContextPolicy,
}

impl ApplicationRelationEndpoints {
    pub const fn new(
        self_edges_allowed: bool,
        cross_context_policy: ApplicationRelationCrossContextPolicy,
    ) -> Self {
        Self {
            self_edges_allowed,
            cross_context_policy,
        }
    }

    pub const fn same_context(self_edges_allowed: bool) -> Self {
        Self::new(
            self_edges_allowed,
            ApplicationRelationCrossContextPolicy::Forbid,
        )
    }
}
