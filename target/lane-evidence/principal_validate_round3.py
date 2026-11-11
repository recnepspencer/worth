from pathlib import Path
import subprocess,sys
r=Path.cwd();e=r/'target/lane-evidence';py=sys.executable
code=subprocess.call([py,str(e/'run_evidence.py'),'principal-native-final-check-driver',py,str(e/'pa_merge_owned_cargo.py'),'principal-native-final-check','Cargo.toml','check','-p','worth-relational','--tests','--message-format=json','--keep-going'])
if code:raise SystemExit('Read all Native diagnostics; focused tests have not launched')
raise SystemExit(subprocess.call([py,str(e/'run_evidence.py'),'principal-final-focused-driver',py,str(e/'principal_verify.py')]))
