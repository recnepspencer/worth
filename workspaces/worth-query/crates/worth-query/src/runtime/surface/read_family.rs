use crate::identity::hash_parts;

use super::WorthQueryReadGraph;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryReadFamilyAdmission {
    KernelOnly,
}

impl WorthQueryReadFamilyAdmission {
    fn digest_component(&self) -> String {
        match self {
            Self::KernelOnly => "admission:kernel_only".to_string(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryReadFamily {
    family_name: String,
    family_digest: String,
    admission: WorthQueryReadFamilyAdmission,
    read_graph: WorthQueryReadGraph,
}

impl WorthQueryReadFamily {
    pub fn family_name(&self) -> &str {
        &self.family_name
    }

    pub fn family_digest(&self) -> &str {
        &self.family_digest
    }

    pub fn admission(&self) -> &WorthQueryReadFamilyAdmission {
        &self.admission
    }

    pub fn read_graph(&self) -> &WorthQueryReadGraph {
        &self.read_graph
    }

    pub(in crate::runtime) fn new_kernel_only(
        family_name: impl Into<String>,
        read_graph: WorthQueryReadGraph,
    ) -> Self {
        Self::new(
            family_name,
            WorthQueryReadFamilyAdmission::KernelOnly,
            read_graph,
        )
    }

    fn new(
        family_name: impl Into<String>,
        admission: WorthQueryReadFamilyAdmission,
        read_graph: WorthQueryReadGraph,
    ) -> Self {
        let family_name = family_name.into();
        let family_digest = hash_parts(&[
            "worth_query_read_family_v1".to_string(),
            format!("family_name:{family_name}"),
            admission.digest_component(),
            format!("read_graph:{}", read_graph.digest()),
        ]);
        Self {
            family_name,
            family_digest,
            admission,
            read_graph,
        }
    }
}
#[cfg(test)]
#[path = "read_family_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
