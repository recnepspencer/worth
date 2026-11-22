//! The closed set of sub-identities a namespace derives.

/// One derived identity of an action, query or feature namespace.
///
/// Its segment is appended to the root: `root.input`, `root.binding`, ...
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ApplicationIdentityRole {
    /// Structured input value binding.
    Input,
    /// Application request value binding of a query.
    Request,
    /// Query parameter value binding.
    Parameters,
    /// Mutation or query binding.
    Binding,
    /// Application query type.
    Query,
    /// Mutation handler.
    Handler,
    /// Idempotency key.
    Idempotency,
    /// Domain denial value binding.
    Denial,
    /// Result value binding.
    Result,
    /// Authored operation body.
    Body,
    /// Action delivery completion.
    Completion,
    /// Query result projection.
    Projection,
}

pub(super) const ROLE_COUNT: usize = ApplicationIdentityRole::ALL.len();

impl ApplicationIdentityRole {
    pub const ALL: [Self; 12] = [
        Self::Input,
        Self::Request,
        Self::Parameters,
        Self::Binding,
        Self::Query,
        Self::Handler,
        Self::Idempotency,
        Self::Denial,
        Self::Result,
        Self::Body,
        Self::Completion,
        Self::Projection,
    ];

    /// The segment appended to the namespace root.
    pub const fn segment(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Request => "request",
            Self::Parameters => "parameters",
            Self::Binding => "binding",
            Self::Query => "query",
            Self::Handler => "handler",
            Self::Idempotency => "idempotency",
            Self::Denial => "denial",
            Self::Result => "result",
            Self::Body => "body",
            Self::Completion => "completion",
            Self::Projection => "projection",
        }
    }

    pub(super) const fn index(self) -> usize {
        self as usize
    }

    /// Bytes of the longest segment, so a root admits every role derivation.
    pub(super) const fn longest_segment() -> usize {
        let mut longest = 0;
        let mut index = 0;
        while index < ROLE_COUNT {
            let length = Self::ALL[index].segment().len();
            if length > longest {
                longest = length;
            }
            index += 1;
        }
        longest
    }
}
