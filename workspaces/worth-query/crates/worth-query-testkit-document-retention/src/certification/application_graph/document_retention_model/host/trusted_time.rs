//! A P0-initial host whose trusted time the certification sets, so deadline
//! law is exercised without waiting on the wall clock.

use std::{sync::Arc, time::Duration};
pub use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
use worth_query_host::facade::primary_graph::WorthQueryRuntimeClock;
use worth_query_host::facade::runtime::time::{system_time_sampler, ControlledTime};

pub use worth_query_host::facade::application_installation::{
    in_memory_rostered_program_with_authorization_time_source, WorthQueryApplicationProgramRoster,
};

pub use super::super::{
    programs::{validated_first_program, validated_second_program, RetentionProgramP0},
    schema::DocumentRetentionSchema,
};
pub use super::{host_limits, seed_host, DocumentRetentionRuntime};

/// Trusted time that moves only when the certification advances it, and can
/// be made unreadable.
#[derive(Clone)]
pub struct CertificationTrustedTime {
    control: Arc<ControlledTime>,
    pub admission_clock:
        worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock,
}

impl Default for CertificationTrustedTime {
    fn default() -> Self {
        Self::new(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    }
}

impl CertificationTrustedTime {
    pub fn new(
        admission_clock: worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock,
    ) -> Self {
        Self {
            control: Arc::new(ControlledTime::at_unix_nanos(
                system_time_sampler().wall().unwrap().unix_nanos(),
            )),
            admission_clock,
        }
    }

    pub fn advance(&self, duration: Duration) {
        self.control.advance(duration);
    }

    pub fn set_unavailable(&self, unavailable: bool) {
        self.control.set_wall_available(!unavailable);
    }
}

/// Publishes the P0-initial host with P1 rostered, on the given trusted time.
pub fn publish_on_first_program_with_trusted_time(
    time: impl Into<WorthQueryRuntimeClock>,
) -> DocumentRetentionRuntime<RetentionProgramP0> {
    let time = time.into();
    let initial = validated_first_program();
    let roster = super::with_lifecycle_programs(
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
        initial.identity(),
        initial.revision(),
    );
    in_memory_rostered_program_with_authorization_time_source(
        initial,
        roster,
        DocumentRetentionSchema::declaration().expect("the document-retention schema is valid"),
        ((),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host_limits(time.admission_clock().clone()),
        seed_host,
        time,
    )
    .expect("the trusted-time host must install")
}

impl From<CertificationTrustedTime> for WorthQueryRuntimeClock {
    fn from(source: CertificationTrustedTime) -> Self {
        Self::from(source.control).with_admission_clock(source.admission_clock)
    }
}
