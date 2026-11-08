from pathlib import Path
import sys
r=Path.cwd();sys.path[:0]=[str(r/'scripts'),str(r/'scripts/test_cost'),str(r/'scripts/ci')]
from cargo_capture import CargoRunner
from application_build import application_build_environment
from run_lane import child_environment
from query_gate_environment import environment
env=application_build_environment({**child_environment(),**environment(r)});runner=CargoRunner(r,env)
name='checkpoint_recovery::computation_partition::oracle::absence::serial::checkpoint_absence_rebuilds_every_owner_call_like_fresh'
result=runner([sys.executable,str(r/'target/lane-evidence/principal_after_measure.py'),str(r/'target/lane-evidence/principal-final-focused/topology/build/binaries.json'),'worth_query_topology_entry',name],r,r/'target/lane-evidence/principal-after-measurement.log')
raise SystemExit(result.returncode)
