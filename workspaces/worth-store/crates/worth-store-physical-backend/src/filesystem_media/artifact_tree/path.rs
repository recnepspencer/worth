const MAX_ARTIFACT_TREE_DEPTH: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::filesystem_media) enum ArtifactTreeRoot {
    Families,
    Staging,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactTreeDirectory {
    pub(in crate::filesystem_media) root: ArtifactTreeRoot,
    pub(in crate::filesystem_media) components: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactTreeFile {
    pub(in crate::filesystem_media) directory: ArtifactTreeDirectory,
    pub(in crate::filesystem_media) file_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactTreePathDenial {
    EmptyComponent,
    SpecialComponent,
    EmbeddedSeparator,
    AlternateDataStream,
    NonPortableComponent,
    ReservedDeviceName,
    ExcessiveDepth,
}

impl ArtifactTreeDirectory {
    pub(crate) fn validate_file_component(component: &str) -> Result<(), ArtifactTreePathDenial> {
        validate_component(component)
    }
    /// Retained path backing, excluding the inline directory value.
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        let slots = self
            .components
            .capacity()
            .checked_mul(std::mem::size_of::<String>())?;
        self.components
            .iter()
            .try_fold(u64::try_from(slots).ok()?, |total, component| {
                total.checked_add(u64::try_from(component.capacity()).ok()?)
            })
    }

    pub fn families() -> Self {
        Self {
            root: ArtifactTreeRoot::Families,
            components: Vec::new(),
        }
    }

    pub fn staging() -> Self {
        Self {
            root: ArtifactTreeRoot::Staging,
            components: Vec::new(),
        }
    }

    pub fn child(&self, component: &str) -> Result<Self, ArtifactTreePathDenial> {
        validate_component(component)?;
        if self.components.len() >= MAX_ARTIFACT_TREE_DEPTH {
            return Err(ArtifactTreePathDenial::ExcessiveDepth);
        }
        let mut components = self.components.clone();
        components.push(component.to_owned());
        Ok(Self {
            root: self.root,
            components,
        })
    }

    pub fn file(&self, component: &str) -> Result<ArtifactTreeFile, ArtifactTreePathDenial> {
        validate_component(component)?;
        Ok(ArtifactTreeFile {
            directory: self.clone(),
            file_name: component.to_owned(),
        })
    }

    pub(in crate::filesystem_media) fn coordination_key(&self) -> String {
        let root = match self.root {
            ArtifactTreeRoot::Families => "families",
            ArtifactTreeRoot::Staging => "staging",
        };
        if self.components.is_empty() {
            return root.to_owned();
        }
        format!("{root}/{}", self.components.join("/"))
    }
}

impl ArtifactTreeFile {
    /// Retained directory and filename backing, excluding the inline file value.
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        self.directory
            .owned_heap_bytes()?
            .checked_add(u64::try_from(self.file_name.capacity()).ok()?)
    }

    pub(in crate::filesystem_media) fn coordination_key(&self) -> String {
        format!("{}/{}", self.directory.coordination_key(), self.file_name)
    }
}

#[cfg(test)]
mod owned_heap_tests {
    use super::ArtifactTreeDirectory;

    #[test]
    fn nested_file_path_charge_includes_owned_component_and_name_backing() {
        let root = ArtifactTreeDirectory::families();
        assert_eq!(root.owned_heap_bytes(), Some(0));
        let nested = root
            .child("records")
            .unwrap()
            .child("segment-manifests")
            .unwrap();
        let file = nested.file("manifest-0000000000000001.frame").unwrap();
        assert!(
            nested.owned_heap_bytes().unwrap()
                >= "records".len() as u64 + "segment-manifests".len() as u64
        );
        let content_lower_bound = 2 * std::mem::size_of::<String>()
            + "records".len()
            + "segment-manifests".len()
            + "manifest-0000000000000001.frame".len();
        assert!(file.owned_heap_bytes().unwrap() >= content_lower_bound as u64);
        assert!(file.owned_heap_bytes().unwrap() > root.owned_heap_bytes().unwrap());
    }
}

pub(in crate::filesystem_media) fn validate_component(
    component: &str,
) -> Result<(), ArtifactTreePathDenial> {
    if component.is_empty() {
        return Err(ArtifactTreePathDenial::EmptyComponent);
    }
    if matches!(component, "." | "..") {
        return Err(ArtifactTreePathDenial::SpecialComponent);
    }
    if component.contains(['/', '\\']) {
        return Err(ArtifactTreePathDenial::EmbeddedSeparator);
    }
    if component.contains(':') {
        return Err(ArtifactTreePathDenial::AlternateDataStream);
    }
    if component.ends_with(['.', ' ']) || component.chars().any(char::is_control) {
        return Err(ArtifactTreePathDenial::NonPortableComponent);
    }
    let stem = component.split('.').next().unwrap_or(component);
    if [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ]
    .iter()
    .any(|device| stem.eq_ignore_ascii_case(device))
    {
        return Err(ArtifactTreePathDenial::ReservedDeviceName);
    }
    Ok(())
}
