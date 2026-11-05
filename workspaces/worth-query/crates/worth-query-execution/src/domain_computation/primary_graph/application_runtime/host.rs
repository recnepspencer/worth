//! The runtime implements every module's host trait here: it composes the subsystems.
mod application_checkpoint;
mod application_discovery;
mod application_entry;
mod application_resource;
mod authorization;
mod conditional_operation;
mod handler;
mod identity;
mod operation_body;
mod output_demand;
mod output_lineage;
mod principal_currentness_capture;
mod product_activation;
mod resolution;
