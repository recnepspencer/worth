//! Authoritative bootstrap gives each observer an actual record and scoped grant.
use super::super::super::{adoption::*, schema::*};
use super::super::capability::Disclosure;
use worth_query_host::facade::primary_graph::*;
pub(super) const OBSERVER: &str = "observer";
pub(super) const DECOY: &str = "observer-decoy";
pub(super) fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<NumericSchema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    for key in [OBSERVER, DECOY] {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Number::reference(),
                WorthQueryApplicationEntityKey::new(key).unwrap(),
            )
            .field(Key::reference(), key.to_owned())
            .field(Primary::reference(), 17)
            .field(Secondary::reference(), 0)
            .field(ReadEnabled::reference(), true)
            .field(Ceiling::reference(), 1000)
            .field(Calculated::reference(), 0),
        )?;
        let grant_key = format!("{key}-inspection-grant");
        let grant =
            || WorthQueryApplicationEntityKey::<NumericSchema, Grant>::new(&grant_key).unwrap();
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(Grant::reference(), grant())
                .field(GrantIdentity::reference(), grant_key.clone())
                .field(GrantAction::reference(), "inspect".to_owned())
                .field(GrantPurpose::reference(), "audit".to_owned())
                .field(Disclosure::reference(), "calculation".to_owned())
                .field(GrantStatus::reference(), "active".to_owned())
                .field(GrantScopeKey::reference(), key.to_owned())
                .field(GrantNotBefore::reference(), 0)
                .field(GrantNotAfter::reference(), u64::MAX)
                .field(GrantDepth::reference(), 0),
        )?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            GrantResource::reference(),
            format!("{key}-resource"),
            grant(),
            WorthQueryApplicationEntityKey::<NumericSchema, Number>::new(key).unwrap(),
        ))?;
        let principal = || {
            WorthQueryApplicationEntityKey::<NumericSchema, Principal>::new("numeric-operator")
                .unwrap()
        };
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            GrantGrantor::reference(),
            format!("{key}-grantor"),
            principal(),
            grant(),
        ))?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            GrantGrantee::reference(),
            format!("{key}-grantee"),
            principal(),
            grant(),
        ))?;
    }
    Ok(())
}
