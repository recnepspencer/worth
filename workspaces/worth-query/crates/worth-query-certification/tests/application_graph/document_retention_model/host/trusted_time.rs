//! A P0-initial host whose trusted time the certification sets, so deadline
//! law is exercised without waiting on the wall clock.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use worth_query_host::facade::application_installation::{
    in_memory_rostered_program_with_authorization_time_source, WorthQueryApplicationProgramRoster,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryRuntimeTimeSource, WorthQueryRuntimeTimeSourceDenial,
};

use super::super::{
    programs::{validated_first_program, validated_second_program, RetentionProgramP0},
    schema::DocumentRetentionSchema,
};
use super::{host_limits, seed_host, DocumentRetentionRuntime};

/// Trusted time that moves only when the certification advances it, and can
/// be made unreadable.
#[derive(Clone)]
pub struct CertificationTrustedTime {
    origin: SystemTime,
    offset_milliseconds: Arc<AtomicU64>,
    unavailable: Arc<AtomicBool>,
}

impl CertificationTrustedTime {
    pub fn new() -> Self {
        Self {
            origin: SystemTime::now(),
            offset_milliseconds: Arc::new(AtomicU64::new(0)),
            unavailable: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn advance(&self, duration: Duration) {
        let milliseconds = u64::try_from(duration.as_millis()).expect("finite test advance");
        self.offset_milliseconds
            .fetch_add(milliseconds, Ordering::SeqCst);
    }

    pub fn set_unavailable(&self, unavailable: bool) {
        self.unavailable.store(unavailable, Ordering::SeqCst);
    }
}

impl WorthQueryRuntimeTimeSource for CertificationTrustedTime {
    fn current_time(&self) -> Result<SystemTime, WorthQueryRuntimeTimeSourceDenial> {
        if self.unavailable.load(Ordering::SeqCst) {
            return Err(WorthQueryRuntimeTimeSourceDenial::Unavailable);
        }
        Ok(self.origin + Duration::from_millis(self.offset_milliseconds.load(Ordering::SeqCst)))
    }
}

/// Publishes the P0-initial host with P1 rostered, on the given trusted time.
pub fn publish_on_first_program_with_trusted_time(
    time: CertificationTrustedTime,
) -> DocumentRetentionRuntime<RetentionProgramP0> {
    in_memory_rostered_program_with_authorization_time_source(
        validated_first_program(),
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
        DocumentRetentionSchema::declaration().expect("the document-retention schema is valid"),
        ((),),
        host_limits(),
        seed_host,
        time,
    )
    .expect("the trusted-time host must install")
}
