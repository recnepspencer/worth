//! Authoritative bootstrap gives each observer an actual record and scoped grant.
use super::super::super::{adoption::*, schema::*};
use super::super::capability::Disclosure;
use worth_query_host::facade::primary_graph::*;
pub(super) const OBSERVER: &str = "observer";
pub(super) const DECOY: &str = "observer-decoy";
pub(in super::super) fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<NumericSchema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    for key in [OBSERVER, DECOY] {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Number::reference(),
                WorthQueryApplicationEntityKey::new(key).unwrap(),
            )
            .field(Key::at("Key"), key.to_owned())
            .field(Primary::at("Primary"), Counter(17))
            .field(Secondary::at("Secondary"), 0)
            .field(ReadEnabled::at("ReadEnabled"), true)
            .field(Ceiling::at("Ceiling"), 1000)
            .field(Calculated::at("Calculated"), 0),
        )?;
        let grant_key = format!("{key}-inspection-grant");
        let grant =
            || WorthQueryApplicationEntityKey::<NumericSchema, Grant>::new(&grant_key).unwrap();
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(Grant::reference(), grant())
                .field(GrantIdentity::at("GrantIdentity"), grant_key.clone())
                .field(GrantAction::at("GrantAction"), "inspect".to_owned())
                .field(GrantPurpose::at("GrantPurpose"), "audit".to_owned())
                .field(Disclosure::at("Disclosure"), "calculation".to_owned())
                .field(GrantStatus::at("GrantStatus"), "active".to_owned())
                .field(GrantScopeKey::at("GrantScopeKey"), key.to_owned())
                .field(GrantNotBefore::at("GrantNotBefore"), 0)
                .field(GrantNotAfter::at("GrantNotAfter"), u64::MAX)
                .field(GrantDepth::at("GrantDepth"), 0),
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
