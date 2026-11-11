//! Principal rosters belong to the installed applications.
use super::*;
use worth_query_decl::facade::application_schema::fragment;
pub struct PrincipalRecords;
impl fragment::Fragment for PrincipalRecords {
    const NAME: &'static str = "principal_records";
    type Bindings = TopologyRecordsBindings;
    type Members = (Principal,);
}

pub struct MinimalPrincipalRecords;
impl fragment::Fragment for MinimalPrincipalRecords {
    const NAME: &'static str = <PrincipalRecords as fragment::Fragment>::NAME;
    type Bindings = TopologyRecordsBindings;
    type Members = (MinimalPrincipal,);
}
pub struct MinimalPrincipal;
impl fragment::Entity for MinimalPrincipal {
    const NAME: &'static str = <Principal as fragment::Entity>::NAME;
    type Aspects = <Principal as fragment::Entity>::Aspects;
    type Relations = ();
}
pub struct PrincipalMappingRecords;
impl fragment::Fragment for PrincipalMappingRecords {
    const NAME: &'static str = <TopologyRecords as fragment::Fragment>::NAME;
    type Bindings = TopologyRecordsBindings;
    type Members = (ExternalPrincipalMapping,);
}
