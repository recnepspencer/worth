//! Genuine Directory callback, pending, transport and terminal-ACK journeys.
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::primary_graph::{
    WorthQueryInboundAdmissionDenial as Denial, WorthQueryInboundPermanentDenialKind as Conflict,
    WorthQueryInboundReceiptPosture as Posture,
};
mod fixture;
use fixture::*;
mod acknowledgement;
mod index_repair;
mod reopening;
mod transport;

mod empty_original;
