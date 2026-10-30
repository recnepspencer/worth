use super::{GeometryDomain, ReadFamily, ReadVertex};
use worth_query::facade::{domain, foundation};

pub type SettledProjection = domain::WorthQuerySettledDomainProjection<
    GeometryDomain,
    ReadVertex,
    ReadFamily,
    foundation::ObservationLaneWitness,
>;
