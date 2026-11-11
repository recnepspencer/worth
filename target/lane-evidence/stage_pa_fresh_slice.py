from pathlib import Path
import subprocess
root='crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/'
paths=[root+'checkpoint_birth_refusal.rs',root+'source_initial_inputs/mod.rs',root+'source_initial_inputs/fresh_admission.rs',root+'source_initial_inputs/fresh_schema.rs',
'crates/worth-relational/src/durability/authority/fresh_source.rs','crates/worth-relational/src/durability/authority/fresh_source/tests.rs','crates/worth-relational/src/durability/authority/checkpoint_complete/tests/empty_schema.rs',
'plans/database-foundation/milestone-3.md','tools/boundary-check/config/src-court-baseline.json','tools/boundary-check/config/certification-construction-callers.txt']
subprocess.run(['git','add','--',*paths],check=True)
# Stage only this slice's mount while keeping active uncommitted query modules in the tree.
parent=root+'mod.rs'
before=subprocess.check_output(['git','show','HEAD:'+parent]).decode('utf-8')
assert 'mod checkpoint_birth_refusal;' not in before
staged=before+'\nmod checkpoint_birth_refusal;\n'
oid=subprocess.check_output(['git','hash-object','-w','--stdin'],input=staged.encode()).decode().strip()
subprocess.run(['git','update-index','--cacheinfo','100644,'+oid+','+parent],check=True)
names=subprocess.check_output(['git','diff','--cached','--name-only']).decode().splitlines()
assert not any(('checkpoint_generation_identity' in p or 'checkpoint_truth_equivalence' in p or 'checkpoint_topology_reidentification' in p) for p in names)
print('Staged exact fresh sources, approved caller entries and fresh-only shared-file hunks; query sources remain unstaged')
