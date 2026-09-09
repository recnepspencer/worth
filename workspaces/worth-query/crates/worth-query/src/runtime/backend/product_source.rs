/// A backend must retain its real owner and Bridge registry before product installation.
#[doc(hidden)]
#[derive(Debug)]
pub enum WorthQueryProductSourceDenial {
    Unsupported,
    SourceNotInstalled,
    Basis(worth_relational::facade::branch::RelationalBranchBasisDenial),
}

impl std::fmt::Display for WorthQueryProductSourceDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "product source admission denied: {self:?}")
    }
}

impl std::error::Error for WorthQueryProductSourceDenial {}
