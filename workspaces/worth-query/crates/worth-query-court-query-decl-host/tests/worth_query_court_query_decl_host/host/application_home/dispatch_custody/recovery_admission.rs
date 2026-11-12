//! Real original-intent admission and durable recovery obligations across Home ownership.
use super::*;
use worth_query_host::facade::application_contribution::WorthQueryAdvancementPhase;
use worth_query_host::facade::primary_graph::*;
use worth_query_testkit_host::facade::host::dispatch_home::recoverable::{
    RecoverableNotice, RecoverableNoticeIntent,
};
use worth_query_testkit_host::facade::host::dispatch_home::schema::Account;
mod fixture;
use fixture::*;
mod lifecycle;
mod progression;
mod refusal;
