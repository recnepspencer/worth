"""Pinned driver artifact receipt through actual sccache misses and hits."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("--driver",type=Path,required=True)
    parser.add_argument("--correction-suffix",action="store_true")
    args=parser.parse_args()
    driver=args.driver.resolve()
    root=driver.parents[2]/"driver-journey"/str(time.time_ns())
    root.mkdir(parents=True)
    print(f"driver artifacts: {root}",flush=True)
    env=os.environ.copy()
    env.pop("RUSTC_BOOTSTRAP",None)
    (root/"lib.rs").write_text('include!("value.rs");pub fn current()->u32{VALUE}\n')
    (root/"value.rs").write_text("pub const VALUE:u32=17;\n")
    command=["sccache",str(driver),"--crate-name=audited","--crate-type=lib","--edition=2024","--emit=link,metadata,dep-info","--out-dir",str(root),str(root/"lib.rs")]
    notes=[]
    if not args.correction_suffix:
        for label in ("miss","hit","changed-input"):
            if label=="changed-input":(root/"value.rs").write_text("pub const VALUE:u32=23;\n")
            start=time.monotonic()
            result=subprocess.run(command,env=env,capture_output=True,text=True)
            (root/(label+".log")).write_text(result.stdout+result.stderr)
            assert result.returncode==0,result.stderr
            dep=(root/"audited.d").read_text()
            line=next(x for x in dep.splitlines() if x.startswith("# worth-expanded-audit="))
            note=json.loads(line.split("=",1)[1])
            for artifact in note["artifacts"]:
                assert hashlib.sha256((root/artifact["name"]).read_bytes()).hexdigest()==artifact["sha256"]
            assert {x["name"] for x in note["artifacts"]}=={"libaudited.rmeta","libaudited.rlib"},note
            notes.append(note)
            print(label,round(time.monotonic()-start,3),note["invocation"],flush=True)
        assert notes[0]==notes[1],"unchanged request did not return its cached audit/artifact binding"
        assert notes[2]["invocation"]!=notes[1]["invocation"] and notes[2]["artifacts"]!=notes[1]["artifacts"]
        (root/"lib.rs").write_text('pub extern "C" fn ordinary()->u32{17}\n')
        result=subprocess.run(command,env=env,capture_output=True,text=True)
        assert result.returncode==0,result.stderr
        (root/"lib.rs").write_text('macro_rules! native {()=>{unsafe{core::arch::asm!("nop");}}}pub fn forbidden(){native!();}\n')
        result=subprocess.run(command,env=env,capture_output=True,text=True)
        (root/"expanded-native-refusal.log").write_text(result.stdout+result.stderr)
        assert result.returncode!=0 and "expanded native assembly requires qualification" in result.stderr,result.stderr
        result=subprocess.run(command[1:]+["--extern","unknown="+str(root/"unknown.dll")],env=env,capture_output=True,text=True)
        assert result.returncode!=0 and "qualified unit dependency owner" in result.stderr,result.stderr
        result=subprocess.run(command[1:]+["-L",str(root)],env=env,capture_output=True,text=True)
        assert result.returncode!=0 and "qualified unit dependency owner" in result.stderr,result.stderr
        response=root/"hidden.args"
        response.write_text("--extern unknown="+str(root/"unknown.dll"))
        result=subprocess.run(command[1:]+["@"+str(response)],env=env,capture_output=True,text=True)
        assert result.returncode!=0 and "response-file compiler context requires qualification" in result.stderr,result.stderr
        result=subprocess.run(command[1:]+["-Zcodegen-backend="+str(root/"unknown.dll")],env=env,capture_output=True,text=True)
        assert result.returncode!=0 and "unqualified compiler argument" in result.stderr,result.stderr
    # A real, controlled native macro demonstrates the implicit-resolution door.
    sysroot=Path(subprocess.check_output(["rustc","--print","sysroot"],text=True).strip())
    compiler=sysroot/"bin/rustc.exe"
    linker=sysroot/"lib/rustlib/x86_64-pc-windows-msvc/bin/rust-lld.exe"
    marker=root/"native-marker.txt"
    macro=root/"native_macro.rs"
    macro.write_text('extern crate proc_macro;#[proc_macro]pub fn emit(input:proc_macro::TokenStream)->proc_macro::TokenStream{std::fs::write('+json.dumps(str(marker))+',"executed").unwrap();input}')
    result=subprocess.run([str(compiler),"--edition=2024","--crate-type=proc-macro","--crate-name=qualified_probe","-C","linker="+str(linker),"--out-dir",str(root),str(macro)],env=env,capture_output=True,text=True)
    (root/"native-macro-build.log").write_text(result.stdout+result.stderr)
    assert result.returncode==0,result.stderr
    control=[str(compiler)]+command[2:]+["-L",str(root)]
    for index,source in enumerate(['extern crate qualified_probe;qualified_probe::emit!();pub fn ordinary(){}','macro_rules! dependency{()=>{extern crate qualified_probe;}}dependency!();qualified_probe::emit!();pub fn ordinary(){}']):
        (root/"lib.rs").write_text(source)
        result=subprocess.run(control,env=env,capture_output=True,text=True)
        (root/("native-control-"+str(index)+".log")).write_text(result.stdout+result.stderr)
        assert result.returncode==0 and marker.read_text()=="executed",result.stderr
        marker.unlink()
        result=subprocess.run(command[1:]+["-L",str(root)],env=env,capture_output=True,text=True)
        (root/("implicit-refusal-"+str(index)+".log")).write_text(result.stdout+result.stderr)
        assert result.returncode!=0 and "qualified unit dependency owner" in result.stderr,result.stderr
        assert not marker.exists(),"unqualified native macro ran before refusal"
    (root/"results.json").write_text(json.dumps(dict(cache=notes,native_control_effect=True,pre_effect_refusals=2),indent=2))
    print("PASS native macro control and pre-effect implicit/macro-generated crate-load refusals" if args.correction_suffix else "PASS actual cache hit, changed dependency miss, same-compilation returned artifacts, expanded-native and external-load refusals",flush=True)


if __name__=="__main__":
    main()
