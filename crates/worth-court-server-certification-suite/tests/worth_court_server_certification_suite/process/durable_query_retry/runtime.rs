//! Shared genuine filesystem Home and admitted Server mutation fixture.
use std::sync::{atomic::AtomicUsize, Arc};
use worth_query_host::facade::{
    application_installation::ApplicationBuildIdentity, WorthQueryApplicationHome,
};
use worth_query_testkit_host::facade::host::application_home as home;
pub(super) use worth_testkit_server::application_mutation::{
    assert_contacts, balance, execute, server,
};
pub(super) fn open(
    contacts: Arc<AtomicUsize>,
) -> WorthQueryApplicationHome<home::schema::HomeIdentitySchema, home::HomeProgram> {
    worth_testkit_server::application_mutation::open(contacts, build())
}
pub(super) fn open_at(
    location: std::path::PathBuf,
    contacts: Arc<AtomicUsize>,
    fresh: bool,
) -> WorthQueryApplicationHome<home::schema::HomeIdentitySchema, home::HomeProgram> {
    worth_testkit_server::application_mutation::open_at(location, contacts, fresh, build())
}
fn build() -> ApplicationBuildIdentity {
    ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap()
}
