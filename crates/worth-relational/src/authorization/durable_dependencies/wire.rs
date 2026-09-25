use super::{
    RelationalAuthorizationDependencyDenial as Denial, RelationalAuthorizationDurableDependencies,
    DURABLE_DEPENDENCY_VERSION, MAXIMUM_AUTHORIZATION_DEPENDENCIES,
    MAXIMUM_AUTHORIZATION_DEPENDENCY_BYTES,
};

impl RelationalAuthorizationDurableDependencies {
    /// Opaque versioned wire value suitable for durable text fields.
    pub fn to_wire_string(&self) -> Result<String, Denial> {
        let bytes = self.bounded_wire_bytes()?;
        let alphabet = b"0123456789abcdef";
        let mut wire = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            wire.push(alphabet[(byte >> 4) as usize] as char);
            wire.push(alphabet[(byte & 0x0f) as usize] as char);
        }
        Ok(wire)
    }

    /// Decodes descriptive evidence, never a live authorization permit.
    pub fn from_wire_string(wire: &str) -> Result<Self, Denial> {
        if wire.len() > MAXIMUM_AUTHORIZATION_DEPENDENCY_BYTES * 2 {
            return Err(Denial::ByteBudgetExceeded);
        }
        if wire.len() % 2 != 0 || wire.is_empty() {
            return Err(Denial::MalformedWire);
        }
        let mut bytes = Vec::with_capacity(wire.len() / 2);
        for pair in wire.as_bytes().chunks_exact(2) {
            let high = nibble(pair[0]).ok_or(Denial::MalformedWire)?;
            let low = nibble(pair[1]).ok_or(Denial::MalformedWire)?;
            bytes.push((high << 4) | low);
        }
        let decoded: Self = rmp_serde::from_slice(&bytes).map_err(|_| Denial::MalformedWire)?;
        if decoded.version != DURABLE_DEPENDENCY_VERSION {
            return Err(Denial::UnsupportedVersion);
        }
        let canonical = decoded.bounded_wire_bytes()?;
        if canonical != bytes {
            return Err(Denial::MalformedWire);
        }
        Ok(decoded)
    }

    pub(super) fn bounded_wire_bytes(&self) -> Result<Vec<u8>, Denial> {
        if self.version != DURABLE_DEPENDENCY_VERSION {
            return Err(Denial::UnsupportedVersion);
        }
        if self.dependency_count() > MAXIMUM_AUTHORIZATION_DEPENDENCIES {
            return Err(Denial::DependencyBudgetExceeded);
        }
        let bytes = rmp_serde::to_vec_named(self).map_err(|_| Denial::MalformedWire)?;
        if bytes.len() > MAXIMUM_AUTHORIZATION_DEPENDENCY_BYTES {
            return Err(Denial::ByteBudgetExceeded);
        }
        Ok(bytes)
    }

    fn dependency_count(&self) -> usize {
        self.paths.iter().fold(2usize, |count, path| {
            count
                .saturating_add(1)
                .saturating_add(path.witness.as_ref().map_or(0, Vec::len))
                .saturating_add(path.entities.len())
                .saturating_add(path.relations.len())
                .saturating_add(path.adjacencies.len())
                .saturating_add(path.fields.len())
        })
    }
}

fn nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}
