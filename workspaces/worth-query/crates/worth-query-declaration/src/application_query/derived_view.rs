use super::ApplicationQueryReference;

/// One named, disposable projection of an installed application query.
/// Query owns retention and invalidation; this contract grants no read or
/// publication authority to the projection.
pub struct ApplicationDerivedViewDefinition<Schema, Query, Parameters, QueryResult, Scope> {
    name: &'static str,
    query: ApplicationQueryReference<Schema, Query, Parameters, QueryResult, Scope>,
    limits: ApplicationDerivedViewLimits,
}

impl<Schema, Query, Parameters, QueryResult, Scope>
    ApplicationDerivedViewDefinition<Schema, Query, Parameters, QueryResult, Scope>
{
    pub const fn new(
        name: &'static str,
        query: ApplicationQueryReference<Schema, Query, Parameters, QueryResult, Scope>,
        limits: ApplicationDerivedViewLimits,
    ) -> Self {
        Self {
            name,
            query,
            limits,
        }
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub const fn query(
        &self,
    ) -> &ApplicationQueryReference<Schema, Query, Parameters, QueryResult, Scope> {
        &self.query
    }

    pub const fn limits(&self) -> ApplicationDerivedViewLimits {
        self.limits
    }
}

/// Finite per-view entry and retained-payload ceilings. A zero bound is
/// rejected when the runtime registers the view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationDerivedViewLimits {
    maximum_entries: usize,
    maximum_retained_bytes: usize,
}

impl ApplicationDerivedViewLimits {
    pub const fn bounded(maximum_entries: usize, maximum_retained_bytes: usize) -> Self {
        Self {
            maximum_entries,
            maximum_retained_bytes,
        }
    }

    pub const fn maximum_entries(self) -> usize {
        self.maximum_entries
    }

    pub const fn maximum_retained_bytes(self) -> usize {
        self.maximum_retained_bytes
    }
}
