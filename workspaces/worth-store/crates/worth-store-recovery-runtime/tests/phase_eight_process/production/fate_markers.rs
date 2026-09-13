use std::collections::BTreeMap;
use std::process::Output;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct IndexedRecoveryFate {
    pub(super) idempotency: [u8; 32],
    pub(super) fate: String,
}

pub(super) fn indexed_fate_tags(
    actual: &[IndexedRecoveryFate],
) -> Result<BTreeMap<[u8; 32], u8>, String> {
    let mut observed = BTreeMap::new();
    for fate in actual {
        let tag = fate_tag(&fate.fate)?;
        if observed.insert(fate.idempotency, tag).is_some() {
            return Err("recovery emitted a duplicate fate identity".to_owned());
        }
    }
    Ok(observed)
}

pub fn persisted_fate_tags(output: &Output) -> Result<BTreeMap<[u8; 32], u8>, String> {
    indexed_fate_tags(&parse_indexed_recovery_fates(output))
}

fn fate_tag(fate: &str) -> Result<u8, String> {
    match fate {
        "AcknowledgedDurable" => Ok(1),
        "DurableUnacknowledged" => Ok(2),
        "ProvenNoEffect" => Ok(3),
        "Indeterminate" => Ok(4),
        other => Err(format!("unknown recovery fate {other}")),
    }
}

pub(super) fn parse_indexed_recovery_fates(output: &Output) -> Vec<IndexedRecoveryFate> {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    stderr
        .lines()
        .chain(stdout.lines())
        .filter(|line| line.starts_with("C8_RECOVERY_FATE "))
        .map(parse_indexed_recovery_fate)
        .collect()
}

fn parse_indexed_recovery_fate(line: &str) -> IndexedRecoveryFate {
    let mut fields = line.split_whitespace();
    assert_eq!(fields.next(), Some("C8_RECOVERY_FATE"));
    let idempotency = fields.next().expect("indexed fate identity");
    let (name, idempotency) = idempotency
        .split_once('=')
        .expect("indexed fate identity uses name=value");
    assert_eq!(name, "idempotency");
    let fate = fields.next().expect("indexed fate classification");
    let (name, fate) = fate
        .split_once('=')
        .expect("indexed fate classification uses name=value");
    assert_eq!(name, "fate");
    assert!(fields.next().is_none());
    IndexedRecoveryFate {
        idempotency: decode_hex_32(idempotency),
        fate: fate.to_owned(),
    }
}

fn decode_hex_32(value: &str) -> [u8; 32] {
    assert_eq!(value.len(), 64);
    let mut bytes = [0; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
            .expect("indexed fate identity is hexadecimal");
    }
    bytes
}
