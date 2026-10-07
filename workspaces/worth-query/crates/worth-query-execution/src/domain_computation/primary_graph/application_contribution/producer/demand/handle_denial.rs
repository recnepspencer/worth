use super::*;

impl From<crate::facade::primary_graph::WorthQueryHandleDenial> for WorthQueryOutputDemandDenial {
    fn from(denial: crate::facade::primary_graph::WorthQueryHandleDenial) -> Self {
        Self::new(
            WorthQueryOutputDemandDenialKind::Handle(denial),
            "application handle",
        )
    }
}
