from pathlib import Path
import json
base=Path('crates/worth-relational/src')
changed=[]
def edit(rel,old,new):
 p=base/rel;s=p.read_text();assert old in s,(rel,old);p.write_text(s.replace(old,new,1));changed.append(p.as_posix())
def opening(s,start):
 depth=0
 for i in range(start,len(s)):
  c=s[i]
  if c in '([':depth+=1
  elif c in ')]':depth-=1
  elif c=='{' and depth==0:return i
 raise AssertionError('missing block')
def ending(s,start):
 depth=0
 for i in range(start,len(s)):
  if s[i]=='{':depth+=1
  elif s[i]=='}':
   depth-=1
   if depth==0:return i
 raise AssertionError('missing end')
def collapse(rel,anchor):
 p=base/rel;s=p.read_text();a=s.index(anchor);b=opening(s,a);end=ending(s,b)
 inner=b+1
 while s[inner].isspace():inner+=1
 assert s[inner:inner+3]=='if ',(rel,s[inner:inner+30])
 ib=opening(s,inner);ie=ending(s,ib)
 assert not s[ie+1:end].strip(),(rel,s[ie+1:end])
 s=s[:a]+'if '+s[a+3:b].strip()+'\n && '+s[inner+3:ib].strip()+' {'+s[ib+1:ie]+'}'+s[end+1:]
 p.write_text(s);changed.append(p.as_posix())
edit('storage/partition/adjacency.rs','/// Borrow memberships one at a time; the receiver owns admission per item.\n\n','/// Borrow memberships one at a time; the receiver owns admission per item.\n')
edit('storage/residency/region_capture.rs','history_frontiers: history_frontiers,','history_frontiers,')
edit('branch/root_readmission/request.rs','require_schema_authority(&schema, &envelope, &descriptor)?','require_schema_authority(&schema, envelope, &descriptor)?')
edit('durability/authority/checkpoint_image.rs','.iter()\n                .cloned()\n                .map(|partition| {','.iter()\n                .map(|partition| {')
edit('validation/preparation_invariant_authority.rs','            &selected,','            selected,')
edit('storage/residency/record_directory/routing.rs','regions.len() == 0','regions.is_empty()')
edit('storage/residency/required_symbols.rs','self.full.then(|| self.owned.as_deref()).flatten()','self.full.then_some(self.owned.as_deref()).flatten()')
edit('publication/ownership/corruption/coherent.rs','let Some(text) = (fields.get(&FieldKey::new("payload")?)?).raw_text() else {\n        return None;\n    };','let text = (fields.get(&FieldKey::new("payload")?)?).raw_text()?;')
edit('publication/ownership/record.rs','let Some(payload) = (payload).raw_text() else {\n        return None;\n    };','let payload = payload.raw_text()?;')
edit('storage/substrate/digest_index/construction.rs','    lane: &crate::storage::substrate::maintenance::ReconstructionLane','    _lane: &crate::storage::substrate::maintenance::ReconstructionLane')
p=base/'storage/substrate/digest_index/construction.rs';p.write_text(p.read_text().replace('build_sorted(lane,','build_sorted(_lane,'))
edit('history/data/canonical_commit_envelope/required_output_origin/header_work.rs','ACTIVE.update(|observation| {\n            observation.map(|mut o| {','ACTIVE.set(ACTIVE.get().map(|mut o| {')
edit('history/data/canonical_commit_envelope/required_output_origin/header_work.rs','                o\n            })\n        });','                o\n        }));')
edit('branch/model_ownership/graph_selection.rs','if !map.contains_key(&edge.relation_id) {\n        read.retain()?;\n        map.insert(edge.relation_id, edge);\n    }','if let std::collections::btree_map::Entry::Vacant(entry) = map.entry(edge.relation_id) {\n        read.retain()?;\n        entry.insert(edge);\n    }')
edit('branch/model_ownership/graph_selection/transfers.rs','if !transferred.contains_key(&entity) {\n                                read.retain()?;\n                                transferred.insert(entity, owner);\n                            }','if let std::collections::btree_map::Entry::Vacant(entry) = transferred.entry(entity) {\n                                read.retain()?;\n                                entry.insert(owner);\n                            }')
for rel,anchor in [
('branch/model_ownership/graph_selection/transfer_history.rs','if let crate::history::data::CanonicalCommitAuthorityKind::OwnedGraphSelection(graph)'),
('branch/model_ownership/graph_selection/transfers.rs','if read.kind(entity)?.is_some()'),
('branch/model_ownership/graph_selection/transfers.rs','if !entities.selected_entities().contains(&entity)'),
('branch/model_ownership/graph_selection.rs','if let Some(now) = read.relation(edge.relation_id)?'),
('branch/model_ownership/graph_selection.rs','if let Some(&owner) = transferred.get(&edge.target)'),
('durability/derived_index_artifacts.rs','if let Entries::RelationJoin(index) = &generation.entries'),
('history/data/canonical_commit_envelope/required_output_origin/group_selection.rs','if self.producer_cascade_retirement.is_none()'),
('indexes/authority/maintenance/refresh_preparation.rs','if let (Some(prior), Some(basis)) = (&prior, &basis)'),
('indexes/authority/maintenance/reads.rs','if let Some(record) = record_metadata::relation(execution, Some(view), *id, work)?'),
('indexes/data/join_leg_state.rs','if let Some((shared, external)) = self.relations.remove(&relation)'),
('publication/ownership/completion.rs','if changed(&row.value.origin.meaning)'),
('runtime/state/subsystems/history_catalog_recovery/source_binding.rs','if let Some(resident) = existing.resident_envelope()'),
('runtime/state/subsystems/history_catalog_recovery/source_binding.rs','if let Some(existing) = &expected'),
('storage/residency/lease.rs','if let Some(certificate) = certificate'),
('validation/engine/engine/proposed_pass.rs','if request.should_execute_anything()')]:collapse(rel,anchor)
Path('target/lane-evidence/pa-integration-lint-files.json').write_text(json.dumps(sorted(set(changed)),indent=2)+'\n')
print('corrected lint files',len(set(changed)))
