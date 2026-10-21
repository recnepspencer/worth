use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

pub(in crate::domain_computation::primary_graph::output_lineage) fn denial(
    kind: WorthQueryOutputDemandDenialKind,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        kind,
        "prepared output lineage slot exceeded its request admission",
    )
}
