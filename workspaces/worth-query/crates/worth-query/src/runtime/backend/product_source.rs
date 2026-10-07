/// A backend must retain its real owner and Bridge registry before product installation.
#[doc(hidden)]
#[derive(Debug)]
pub enum WorthQueryProductSourceDenial {
    Handle(worth_query_execution::facade::primary_graph::WorthQueryHandleDenial),
    InvalidBranchName(String),
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

impl From<worth_query_execution::facade::primary_graph::WorthQueryHandleDenial>
    for WorthQueryProductSourceDenial
{
    fn from(denial: worth_query_execution::facade::primary_graph::WorthQueryHandleDenial) -> Self {
        Self::Handle(denial)
    }
}
impl From<worth_query_execution::facade::integration::WorthQueryRelationalSourceDenial>
    for WorthQueryProductSourceDenial
{
    fn from(
        denial: worth_query_execution::facade::integration::WorthQueryRelationalSourceDenial,
    ) -> Self {
        match denial {
            worth_query_execution::facade::integration::WorthQueryRelationalSourceDenial::Handle(denial) => Self::Handle(denial),
            worth_query_execution::facade::integration::WorthQueryRelationalSourceDenial::Basis(denial) => Self::Basis(denial),
            worth_query_execution::facade::integration::WorthQueryRelationalSourceDenial::InvalidBranchName(name) => Self::InvalidBranchName(name),
        }
    }
}
