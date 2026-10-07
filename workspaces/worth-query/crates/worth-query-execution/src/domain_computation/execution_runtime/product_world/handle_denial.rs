/// Why a retained Query handle cannot enter its application owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryHandleDenial {
    /// The application returned to its home and revoked retained handles.
    Closed,
}

impl std::fmt::Display for WorthQueryHandleDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Query handle denied: {self:?}")
    }
}

impl std::error::Error for WorthQueryHandleDenial {}
