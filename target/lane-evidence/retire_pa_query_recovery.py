from pathlib import Path
import json
E=Path('target/lane-evidence');s=(E/'pa-query-recovery-public.log').read_text();assert 'EVIDENCE_TEST_COUNTS: 2,0,0' in s and s.rstrip().endswith('exit_code: 0')
plan=(E/'pa-query-recovery-plan.md').read_text();closure=next(p for p in plan.split('\n\n') if p.startswith('Generation case retains'))
p=Path('plans/database-foundation/milestone-3.md');s=p.read_text();h='### C2 - Cost and bounds: backing and work budgets\n';assert h in s;p.write_text(s.replace(h,h+'\nGeneration selection private counter remainder:\n'+closure+'\n\n',1))
R=Path('crates/worth-relational/src');removed=['tests/query/indexes/generation_identity_recovery.rs','tests/domains/cad/topology_identity_survival.rs','tests/domains/cad/mod.rs']
for f in removed:
 p=R/f;assert p.exists(),f
 if f.endswith('cad/mod.rs'):assert p.read_text().strip()=='mod topology_identity_survival;'
 p.unlink()
for f,mount in [('tests/query/indexes/mod.rs','mod generation_identity_recovery;\n'),('tests/domains/mod.rs','mod cad;\n')]:
 p=R/f;s=p.read_text();assert mount in s;p.write_text(s.replace(mount,''))
p=Path('tools/boundary-check/config/src-court-baseline.json');s=p.read_text();p.write_text('\n'.join(line for line in s.split('\n') if not any((R/f).as_posix() in line for f in removed)))
assert (R/'tests/query/indexes/recovery_and_execution_models.rs').exists()
(E/'pa-query-recovery-retirement-ledger.json').write_text(json.dumps({'base':'14b6075657193d0f74ce944beab6124b5dec7b3c','removed':[(R/f).as_posix() for f in removed],'public_cases':json.loads((E/'pa-query-recovery-cases.json').read_text()),'C2_private_claims':['generation_payload_reads delta1','history_inventory_entries delta0'],'legacy_generic_parity':'all original Native roots/child modules remain unchanged and mounted; separate owner/shipping-lookup split pending'},indent=2)+'\n')
print('Retired2qualifiedfunctionalroots/emptycadmount; exactC2counterclosurepinned; legacyNativeparityrootsuntouched')
