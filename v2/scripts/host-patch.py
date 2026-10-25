"""Build a narrow, UNAPPLIED patch from immutable host fixture bytes."""
from pathlib import Path
import difflib
ROOT=Path(__file__).resolve().parents[1]
FIX=ROOT/'qa/fixtures/host'
changes={}
def edit(file, edits):
    old=(FIX/file).read_text();new=old
    for a,b in edits:
        assert a in new,(file,a[:80]);new=new.replace(a,b)
    changes[file]=(old,new)
f='src/renderer/src/raycast-api/form-runtime.tsx'
edit(f,[
 ('  actionId?: string\n','  actionId?: string\n  focusActionId?: string\n'),
 ("          actionId: typeof node.props?.actionId === 'string' ? node.props.actionId : undefined,", "          actionId: typeof node.props?.actionId === 'string' ? node.props.actionId : undefined,\n          focusActionId: typeof node.props?.focusActionId === 'string' ? node.props.focusActionId : undefined,"),
 ('  onChangeField: (actionId: string, value: unknown) => Promise<void> | void','  onChangeField: (actionId: string, value: unknown, values?: Record<string, unknown>) => Promise<void> | void'),
 ('  const nativeColorWheel = useMemo', '''  const draft = useRef({ values, revision: 0, lastChangedField: '' })
  const wholeForm = (): Record<string, unknown> => ({
    ...draft.current.values,
    _inputRevision: draft.current.revision,
    _lastChangedField: draft.current.lastChangedField,
  })
  const nativeColorWheel = useMemo'''),
 ('    setValues((previous) => ({\n      ...reconcileFormValues(fields, previousFields ? previous : undefined, previousFields),\n      ...wheelDraft,\n    }))', '''    const serverRevision = root.props?.inputRevision
    // Native revisioned forms must never overwrite a newer local draft with a
    // delayed controlled response, including an A/B/A edit sequence.
    if (typeof serverRevision === 'number' && serverRevision < draft.current.revision) return
    setValues((previous) => {
      const next = {
        ...reconcileFormValues(fields, previousFields ? previous : undefined, previousFields),
        ...wheelDraft,
      }
      draft.current.values = next
      return next
    })'''),
 ('  }, [fields, nativeColorWheel, title])','  }, [fields, nativeColorWheel, title, root.props?.inputRevision])'),
 ('  useImperativeHandle(ref, () => ({ getValues: () => values }), [values])','  useImperativeHandle(ref, () => ({ getValues: wholeForm }), [values])'),
 ('    setValues((previous) => ({ ...previous, [field.id]: value }))\n    if (field.actionId) void onChangeField(field.actionId, value)', '''    const next = { ...draft.current.values, [field.id]: value }
    draft.current = { values: next, revision: draft.current.revision + 1, lastChangedField: field.id }
    setValues(next)
    if (field.actionId) void onChangeField(field.actionId, value, wholeForm())'''),
 ('onSubmitForm(values)','onSubmitForm(wholeForm())'),
 ('                    type={inputType}\n                    value={current}', '''                    type={inputType}
                    autoFocus={field.id === root.props?.focusedFieldId}
                    onFocus={() => {
                      if (field.focusActionId) void onInvokeAction(field.focusActionId, wholeForm())
                    }}
                    value={current}'''),
])
f='src/renderer/src/raycast-api/index.tsx'
edit(f,[
 ('onChangeField={(actionId, value) => onInvokeAction(actionId, { value })}', 'onChangeField={(actionId, value, values) => onInvokeAction(actionId, { ...values, value })}'),
 ('                void onInvokeAction(action.id)\n', "                void onInvokeAction(action.id, kind === 'form' ? formRuntimeRef.current?.getValues() : undefined)\n"),
])
f='src-tauri/src/native_core/extensions/registry.rs'
old=(FIX/f).read_text()
a=old.index('pub(super) fn installed_command(');b=old.index('\nfn platform()',a)
new='''pub(super) fn installed_command(ctx: &CoreContext, id: &str, command: &str) -> Option<Installed> {
    // Only a successfully verified active package may supply a declared alias.
    // A same-slug name or prefix alone is never replacement authority.
    let canonical = id.starts_with("com.tezbar.native.");
    let native = collect(ctx).into_iter().find(|entry| {
        entry.id.starts_with("com.tezbar.native.")
            && entry.manifest["healthError"].is_null()
            && if canonical { entry.id == id } else {
                entry.manifest["replaces"].as_array().is_some_and(|ids| ids.iter().any(|alias| alias.as_str() == Some(id)))
            }
            && entry.manifest["commands"].as_array().is_some_and(|commands| commands.iter().any(|c|
                c["name"] == command && c["runtime"] == "rust" && compatible(c, true)))
    });
    if native.is_some() || canonical { return native; }
    let extension = installed(ctx, id).ok()?;
    (compatible(&extension.manifest, false)
        && extension.manifest["commands"].as_array().into_iter().flatten().any(|cmd|
            cmd["name"].as_str() == Some(command) && compatible(cmd, true)))
        .then_some(extension)
}

pub(super) fn canonical_command_id(ctx: &CoreContext, id: &str, command: &str) -> Option<String> {
    installed_command(ctx, id, command)
        .filter(|entry| entry.id.starts_with("com.tezbar.native."))
        .map(|entry| entry.id)
}
'''
new=old[:a]+new+old[b:]
# Keep path canonical for preferences/uninstall; only assets use active version.
new=new.replace('    data_dir: PathBuf,','    data_dir: PathBuf,\n    resource_path: PathBuf,')
new=new.replace('            let mut manifest = if native {','            let mut resource_path = path.clone();\n            let mut manifest = if native {')
new=new.replace('                    Ok(package) => package.manifest,','                    Ok(package) => { resource_path = package.dir; package.manifest },')
new=new.replace('                data_dir: ctx.paths.data_dir.clone(),','                data_dir: ctx.paths.data_dir.clone(),\n                resource_path,')
new=new.replace('        path,\n        manifest: Value::Null,','        resource_path: path.clone(),\n        path,\n        manifest: Value::Null,')
new=new.replace('        extension.path.join(relative),\n        extension.path.join("assets").join(relative),','        extension.resource_path.join(relative),\n        extension.resource_path.join("assets").join(relative),')
# Tests that construct the internal struct must set its separate resource root.
# Actual host tests mostly use collect; assert constructors only production.
changes[f]=(old,new)
f='src-tauri/src/native_core/extensions/native.rs'
edit(f,[
 ('if requested.starts_with("com.tezbar.native.") || v2_runtime::slug(requested).is_some()', 'if requested.starts_with("com.tezbar.native.") || v2_runtime::slug(requested).is_some()\n                || registry::canonical_command_id(ctx, requested, body["commandName"].as_str().unwrap_or("")).is_some()'),
 ('            v2_runtime::canonical_id(v2_runtime::slug(requested).ok_or("Unknown extension")?)','            match registry::canonical_command_id(ctx, requested, command) {\n                Some(id) => id,\n                None => return Ok(None),\n            }'),
])
out=ROOT/'integration/host.patch';out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(''.join(''.join(difflib.unified_diff(a.splitlines(True),b.splitlines(True),fromfile='a/'+f,tofile='b/'+f)) for f,(a,b) in changes.items()))
for f,(_,new) in changes.items():
    p=ROOT/'.qa/patched-host'/f;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(new)
print('Unapplied host patch:',out.relative_to(ROOT))
