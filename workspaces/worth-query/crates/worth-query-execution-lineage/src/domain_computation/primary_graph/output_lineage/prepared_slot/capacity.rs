use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

pub fn denial(
    kind: WorthQueryOutputDemandDenialKind,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        kind,
        "prepared output lineage slot exceeded its request admission",
    )
}
