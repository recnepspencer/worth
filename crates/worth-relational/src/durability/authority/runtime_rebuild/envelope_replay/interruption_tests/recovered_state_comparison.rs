use super::{observable_state, RelationalRuntime};

// Independent owners issue distinct owner-bound visibility proofs and storage
// allocation identities. Check each current root's complete content/proof tuple,
// then canonicalize those identities by first occurrence, preserving aliasing.
pub(super) fn comparable_recovered_state(target: &RelationalRuntime) -> String {
    for cell in target.history.branch_cells_snapshot() {
        let root = target
            .history
            .branch_cell(&cell.branch_id)
            .unwrap()
            .root()
            .unwrap();
        assert!(root.is_complete(&target.services.symbols.interner_snapshot()));
    }
    let state = observable_state(target)
        .replace(
            &format!("runtime_instance_id: {},", target.runtime_instance_id()),
            "runtime_instance_id: <recovered owner>,",
        )
        .replace(
            &format!("relational/{}/", target.runtime_instance_id()),
            "relational/<recovered owner>/",
        );
    let mut allocations = std::collections::BTreeMap::new();
    let mut proofs = std::collections::BTreeMap::new();
    let mut owners = std::collections::BTreeMap::new();
    let mut previous = "";
    let mut normalized = Vec::new();
    let state = canonicalize_unordered_fields(&canonicalize_retired_roots(
        &state.lines().map(str::to_owned).collect::<Vec<_>>(),
    ))
    .join("\n");
    let mut lines = state.lines();
    while let Some(line) = lines.next() {
        let indent = &line[..line.len() - line.trim_start().len()];
        let field = line.trim();
        let proof = if field == "visibility: RelationalBranchVisibilityCommitment(" {
            Some(("visibility", "),"))
        } else if field == "visibility_commitment: [" {
            Some(("visibility_commitment", "],"))
        } else {
            None
        };
        if let Some((name, end)) = proof {
            let mut value = String::new();
            for part in lines.by_ref() {
                if part.trim() == end {
                    break;
                }
                if part.trim() != "[" && part.trim() != "]," {
                    value.push_str(part.trim());
                }
            }
            let next = proofs.len();
            let id = *proofs.entry(value).or_insert(next);
            normalized.push(format!("{indent}{name}: <owner proof {id}>,"));
        } else if let Some(id) = field
            .strip_prefix("owner_identity: ")
            .and_then(|id| id.strip_suffix(','))
            .and_then(|id| id.parse::<u64>().ok())
        {
            let next = owners.len();
            let owner = *owners.entry(id).or_insert(next);
            normalized.push(format!("{indent}owner_identity: <owner {owner}>,"));
        } else if previous.trim() == "Allocation {" {
            let id = field
                .strip_prefix("id: ")
                .and_then(|id| id.strip_suffix(','))
                .and_then(|id| id.parse::<u64>().ok())
                .expect("storage allocation debug has its identity first");
            let next = allocations.len();
            let canonical = *allocations.entry(id).or_insert(next);
            normalized.push(format!("{indent}id: {canonical},"));
        } else {
            normalized.push(line.to_owned());
        }
        previous = line;
    }
    normalized.join("\n")
}

// Retention keys its retired roots by native Arc address. Their values carry
// the complete root and retirement generation; compare that unordered map by
// value, retaining every entry and every field instead of its address order.
fn canonicalize_retired_roots(lines: &[String]) -> Vec<String> {
    let mut output = Vec::new();
    let mut identities = std::collections::BTreeMap::new();
    let mut index = 0;
    while index < lines.len() {
        let line = &lines[index];
        output.push(line.clone());
        index += 1;
        if line.trim() != "retired_roots: {" {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let mut entries = Vec::new();
        while index < lines.len() && lines[index].len() - lines[index].trim_start().len() > indent {
            let address = lines[index].trim().split(':').next().unwrap().to_owned();
            let mut entry = vec![lines[index].clone()];
            index += 1;
            while index < lines.len() {
                let next = &lines[index];
                let entry_end =
                    next.len() - next.trim_start().len() == indent + 4 && next.trim() == "},";
                entry.push(next.clone());
                index += 1;
                if entry_end {
                    break;
                }
            }
            let generation = entry
                .iter()
                .find_map(|line| {
                    (line.len() - line.trim_start().len() == indent + 8)
                        .then(|| {
                            line.trim()
                                .strip_prefix("generation: ")
                                .and_then(|generation| generation.strip_suffix(','))
                        })
                        .flatten()
                })
                .expect("a retired root carries its retirement generation");
            let identity = format!("<retired root generation {generation}>");
            identities.insert(address, identity.clone());
            entry[0] = format!(
                "{}{}: RelationalRetiredBranchRoot {{",
                " ".repeat(indent + 4),
                identity
            );
            entries.push(entry);
        }
        entries.sort();
        output.extend(entries.into_iter().flatten());
    }
    // The by-commit index carries the same native keys: preserve that alias.
    for line in &mut output {
        if let Some(identity) = line
            .trim()
            .strip_suffix(',')
            .and_then(|value| identities.get(value))
        {
            *line = format!(
                "{}{identity},",
                &line[..line.len() - line.trim_start().len()]
            );
        }
    }
    output
}

// Rust Debug maps and sets use anonymous braces; their traversal order is not
// observable meaning. Canonicalize their complete entries, never Vec order.
fn canonicalize_unordered_fields(lines: &[String]) -> Vec<String> {
    let mut output = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let line = &lines[index];
        output.push(line.clone());
        index += 1;
        if !line.trim().ends_with(": {") {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let mut entries = Vec::new();
        while index < lines.len() && lines[index].len() - lines[index].trim_start().len() > indent {
            let mut entry = Vec::new();
            while index < lines.len() {
                let next = &lines[index];
                let entry_end = next.len() - next.trim_start().len() == indent + 4
                    && next.trim().ends_with(',');
                entry.push(next.clone());
                index += 1;
                if entry_end {
                    break;
                }
            }
            entries.push(canonicalize_unordered_fields(&entry));
        }
        entries.sort();
        output.extend(entries.into_iter().flatten());
    }
    output
}
