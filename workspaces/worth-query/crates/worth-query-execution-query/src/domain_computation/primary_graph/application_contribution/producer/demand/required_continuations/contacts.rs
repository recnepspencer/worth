//! Account for a caller's own executions before successor custody can end.
use super::*;

pub enum ContactAttribution {
    Unowned,
    Caller { reported: usize },
}

impl<Schema: ApplicationSchema> RequiredFreshProgress<Schema> {
    pub fn attribute_to_caller(&mut self, total: &mut usize) {
        assert!(matches!(self.contacts, ContactAttribution::Unowned));
        self.contacts = ContactAttribution::Caller { reported: 0 };
        self.report_caller_contacts(Some(total));
    }

    pub fn report_caller_contacts(&mut self, total: Option<&mut usize>) {
        if let ContactAttribution::Caller { reported } = &mut self.contacts {
            let total = total.expect("caller-owned successor resumes under its caller's custody");
            let contacts = self.successor.producer_contacts();
            *total += contacts
                .checked_sub(*reported)
                .expect("execution contacts never decrease");
            *reported = contacts;
        }
    }
}
