from pathlib import Path
import json,hashlib,re
ROOT=Path(__file__).resolve().parents[1]
FIX=ROOT/'qa/fixtures/host'
records=[]
def function(path,name,patched=False):
    root=ROOT/'.qa/patched-host' if patched else FIX
    s=(root/path).read_text()
    match=re.search(r'^(?:pub(?:\([^\n]*?\))? )?fn '+name+r'\b',s,re.M);assert match,(path,name)
    start=match.start();a=s.index('{',match.end());level=1;b=a+1
    while level:
        level+=(s[b]=='{')-(s[b]=='}');b+=1
    out=s[start:b]
    records.append({'path':path,'function':name,'patched':patched,'sha256':hashlib.sha256(out.encode()).hexdigest()})
    return out+'\n'
base='src-tauri/src/native_core/extensions/'
s='// GENERATED from exact host fixtures by scripts/host-fixtures.py.\n'
s+='pub mod process_install {\nuse serde_json::Value;\nuse std::{collections::HashSet,path::PathBuf};\n'
source=(FIX/(base+'process_install.rs')).read_text();a=source.index('const CAPABILITY_GROUPS');b=source.index('fn valid_capability');s+=source[a:b]
for name in ['valid_capability','component','relative','validate']:s+=function(base+'process_install.rs',name)
s+='}\npub mod launcher {\nuse serde_json::Value;\nuse std::collections::HashSet;\nuse super::process_install;\n'
for name in ['bounded_text','validate']:s+=function(base+'launcher.rs',name)
s+='}\n'
s+='pub mod registry {\nuse serde_json::Value;\nuse std::path::{Path,PathBuf,Component};\n'
s+='''#[derive(Clone)]
pub struct Installed { pub id:String,pub slug:String,pub path:PathBuf,pub resource_path:PathBuf,pub manifest:Value }
pub struct CoreContext {pub entries:Vec<Installed>}
fn collect(ctx:&CoreContext)->Vec<Installed> {ctx.entries.clone()}
fn installed(ctx:&CoreContext,id:&str)->Result<Installed,String> {ctx.entries.iter().find(|e|e.id==id).cloned().ok_or("not installed".into())}
'''
for name in ['platform','compatible','fallback','installed_command','canonical_command_id','icon_path','command_icon_path']:
 s+=function(base+'registry.rs',name,True)
s+='pub(super) fn test_icon_path(e:&Installed)->Option<String> {icon_path(e)}\npub(super) fn test_command_icon_path(e:&Installed,c:&Value)->Option<String> {command_icon_path(e,c)}\n}\n'
(ROOT/'qa/host-functions.rs').write_text(s)
(ROOT/'qa/host-function-hashes.json').write_text(json.dumps(records,indent=2)+'\n')
