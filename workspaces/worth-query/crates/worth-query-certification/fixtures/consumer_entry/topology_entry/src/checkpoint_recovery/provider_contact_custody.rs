//! Foreign provider work cannot satisfy or contaminate a receiver's observation.
use super::*;
use crate::producer::ProviderContacts;

#[test]
fn installed_provider_contacts_belong_to_their_actual_application_configuration() {
    let _guard = checkpoint_recovery_test_guard();
    let subject_contacts = ProviderContacts::default();
    let foreign_contacts = ProviderContacts::default();
    let subject = support::observed::install(None, &subject_contacts);
    let foreign = support::observed::install(None, &foreign_contacts);
    subject_contacts.reset();
    foreign_contacts.reset();

    // The same authored keys and program occur in both genuine installations.
    // Receiver ownership, rather than a body key or worker position, separates them.
    let (scope, principal) = authenticate(&foreign);
    let request = foreign.request(&principal, &scope);
    let settled = settle(&request, &foreign);
    assert!(settled.work().producer_contact_count() > 0);
    drop(settled);
    assert!(foreign_contacts.count() > 0);
    assert_eq!(subject_contacts.count(), 0);
    let foreign_before = foreign_contacts.count();

    let (scope, principal) = authenticate(&subject);
    let request = subject.request(&principal, &scope);
    let settled = settle(&request, &subject);
    assert!(settled.work().producer_contact_count() > 0);
    drop(settled);
    assert!(subject_contacts.count() > 0);
    assert_eq!(foreign_contacts.count(), foreign_before);
}
