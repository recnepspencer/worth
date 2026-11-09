from pathlib import Path
import subprocess,sys
r=Path.cwd();e=r/'target/lane-evidence';py=sys.executable
code=subprocess.call([py,str(e/'run_evidence.py'),'deferred-geometry-native-round2-check-driver',py,str(e/'pa_merge_owned_cargo.py'),'deferred-geometry-native-round2-check','Cargo.toml','check','-p','worth-relational','--tests','--message-format=json','--keep-going'])
if code:raise SystemExit('Read every Native diagnostic before another round')
raise SystemExit(subprocess.call([py,str(e/'run_evidence.py'),'deferred-geometry-focused-driver',py,str(e/'deferred_geometry_verify.py')]))
