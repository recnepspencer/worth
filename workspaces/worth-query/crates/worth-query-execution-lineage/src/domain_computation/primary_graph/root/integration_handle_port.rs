//! Crate-boundary internal port for `WorthQueryPrimaryGraphIntegrationHandle`
//! backing fields. Never exported on `worth-query-execution::facade` or Host.

use std::sync::{Arc, Mutex};

use super::super::aggregate_projection::WorthQueryAggregateProjections;
use super::super::output_lineage::WorthQueryApplicationOutputLineage;
use super::super::schema_layout::WorthQueryPrimaryGraphLayout;
use super::application_publication::ApplicationPublication;
use super::home_retirement::PrimaryGraphSourceCustody;
use super::WorthQueryPrimaryGraphIntegrationHandle;

pub trait PrimaryGraphIntegrationHandleInternalPort {
    fn application_publication_ref(&self) -> Option<&Arc<ApplicationPublication>>;
    fn source_owner_ref(&self) -> &PrimaryGraphSourceCustody;
    fn layout_ref(&self) -> &Arc<WorthQueryPrimaryGraphLayout>;
    fn primary_index_ids_ref(&self) -> &Arc<[worth_relational::facade::indexes::DerivedIndexId]>;
    fn aggregate_projections_ref(&self) -> &Arc<Mutex<WorthQueryAggregateProjections>>;
    fn output_lineage_ref(&self) -> &Arc<Mutex<WorthQueryApplicationOutputLineage>>;
}

impl PrimaryGraphIntegrationHandleInternalPort for WorthQueryPrimaryGraphIntegrationHandle {
    fn application_publication_ref(&self) -> Option<&Arc<ApplicationPublication>> {
        self.application_publication.as_ref()
    }

    fn source_owner_ref(&self) -> &PrimaryGraphSourceCustody {
        &self.source_owner
    }

    fn layout_ref(&self) -> &Arc<WorthQueryPrimaryGraphLayout> {
        &self.layout
    }

    fn primary_index_ids_ref(&self) -> &Arc<[worth_relational::facade::indexes::DerivedIndexId]> {
        &self.primary_index_ids
    }

    fn aggregate_projections_ref(&self) -> &Arc<Mutex<WorthQueryAggregateProjections>> {
        &self.aggregate_projections
    }

    fn output_lineage_ref(&self) -> &Arc<Mutex<WorthQueryApplicationOutputLineage>> {
        &self.output_lineage
    }
}
