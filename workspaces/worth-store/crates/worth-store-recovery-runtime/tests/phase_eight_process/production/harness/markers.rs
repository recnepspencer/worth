use std::process::Output;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryRuntimeMarker {
    pub store: [u8; 16],
    pub runtime: u64,
    pub root_generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryFateMarker {
    pub acknowledged: u64,
    pub durable_unacknowledged: u64,
    pub proven_no_effect: u64,
    pub indeterminate: u64,
}

impl RecoveryFateMarker {
    pub fn total(self) -> u64 {
        self.acknowledged
            .saturating_add(self.durable_unacknowledged)
            .saturating_add(self.proven_no_effect)
            .saturating_add(self.indeterminate)
    }
}

pub fn parse_recovery_runtime_marker(output: &Output) -> RecoveryRuntimeMarker {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stderr
        .lines()
        .chain(stdout.lines())
        .find(|line| line.starts_with("C8_RECOVERY_RUNTIME "))
        .unwrap_or_else(|| {
            panic!(
                "production recovery runtime marker missing\nstdout:\n{stdout}\nstderr:\n{stderr}"
            )
        });
    let mut fields = line.split_whitespace();
    assert_eq!(fields.next(), Some("C8_RECOVERY_RUNTIME"));
    let store_hex = fields.next().expect("recovery Store marker");
    let runtime = fields
        .next()
        .expect("recovery runtime marker")
        .parse()
        .expect("numeric runtime marker");
    let root_generation = fields
        .next()
        .expect("recovery root generation marker")
        .parse()
        .expect("numeric root generation marker");
    assert!(fields.next().is_none());
    RecoveryRuntimeMarker {
        store: decode_hex_store(store_hex),
        runtime,
        root_generation,
    }
}

pub fn parse_recovery_fate_marker(output: &Output) -> RecoveryFateMarker {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stderr
        .lines()
        .chain(stdout.lines())
        .find(|line| line.starts_with("C8_RECOVERY_FATES "))
        .unwrap_or_else(|| {
            panic!("production recovery fate marker missing\nstdout:\n{stdout}\nstderr:\n{stderr}")
        });
    let mut fields = line.split_whitespace();
    assert_eq!(fields.next(), Some("C8_RECOVERY_FATES"));
    let acknowledged = parse_marker_field(fields.next(), "acknowledged");
    let durable_unacknowledged = parse_marker_field(fields.next(), "durable_unacknowledged");
    let proven_no_effect = parse_marker_field(fields.next(), "proven_no_effect");
    let indeterminate = parse_marker_field(fields.next(), "indeterminate");
    assert!(fields.next().is_none());
    RecoveryFateMarker {
        acknowledged,
        durable_unacknowledged,
        proven_no_effect,
        indeterminate,
    }
}

fn parse_marker_field(field: Option<&str>, expected_name: &str) -> u64 {
    let field = field.expect("recovery fate marker field");
    let (name, value) = field
        .split_once('=')
        .expect("recovery fate marker uses name=value fields");
    assert_eq!(name, expected_name);
    value.parse().expect("numeric recovery fate marker")
}

fn decode_hex_store(value: &str) -> [u8; 16] {
    assert_eq!(value.len(), 32);
    let mut bytes = [0; 16];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).expect("hex Store marker");
    }
    bytes
}
