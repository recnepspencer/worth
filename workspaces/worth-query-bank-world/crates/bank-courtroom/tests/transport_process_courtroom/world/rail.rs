use std::time::Duration;

use bank_external_rail::test_control::{select_fault, FaultScript};
use bank_external_rail::{
    inquire_admission_count, inquire_completed_effect_count, inquire_dispatch_contact_count,
};

use super::TransportProcessWorld;

impl TransportProcessWorld {
    pub async fn rail_admission_count(&self) -> u64 {
        inquire_admission_count(self.rail.local_addr(), Duration::from_secs(5))
            .await
            .expect("the rail should report its own admission count")
    }

    pub async fn rail_contact_count(&self) -> u64 {
        inquire_dispatch_contact_count(self.rail.local_addr(), Duration::from_secs(5))
            .await
            .expect("the rail should report every dispatch frame it received")
    }

    pub async fn rail_completed_effect_count(&self) -> u64 {
        inquire_completed_effect_count(self.rail.local_addr(), Duration::from_secs(5))
            .await
            .expect("the rail should report its own completed effect count")
    }

    pub async fn select_rail_fault(&self, fault: FaultScript) {
        select_fault(self.rail.test_control_addr(), fault, Duration::from_secs(5))
            .await
            .expect("the rail should accept a selected fault posture");
    }
}
