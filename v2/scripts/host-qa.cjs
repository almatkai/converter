const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..');const ts=require(path.join(root,'.qa/v1/node_modules/typescript'));
const Ajv=require(path.join(root,'.qa/schema-tools/node_modules/ajv/dist/2020'));const formats=require(path.join(root,'.qa/schema-tools/node_modules/ajv-formats'));
const ajv=new Ajv({strict:false,allErrors:true});formats(ajv);
const schema=JSON.parse(fs.readFileSync(path.join(root,'qa/fixtures/host/schemas/extension-v3.schema.json')));const validate=ajv.compile(schema);
const manifest=JSON.parse(fs.readFileSync(path.join(root,'extension.json')));assert(validate(manifest),JSON.stringify(validate.errors));
for(const [key,value] of [['schemaVersion',2],['runtime','javascript'],['id','raycast.converter'],['capabilities',['process.exec']],['artifacts',{'made-up':{}}],['extra','x']]){const bad=structuredClone(manifest);bad[key]=value;assert(!validate(bad),key);}
const deny=()=>{throw new Error('QA producer denied');};
function loader(file,extras=''){
 let slots=[],i=0,effects=[];
 const react={useState(v){let k=i++;if(!(k in slots))slots[k]=typeof v==='function'?v():v;return [slots[k],n=>slots[k]=typeof n==='function'?n(slots[k]):n];},useRef(v){return react.useState({current:v})[0];},useMemo:fn=>fn(),useCallback:fn=>fn,useEffect:fn=>effects.push(fn),useImperativeHandle(ref,fn){ref.current=fn();},forwardRef:fn=>fn};react.default=react;
 const jsx=(type,props)=>({type,props});
 const mock=(name)=>new Proxy({},{get:(_t,k)=>name+'.'+String(k)});
 const module={exports:{}};
 const requireMock=name=>{
  if(name==='react')return react;
  if(name==='react/jsx-runtime')return {jsx,jsxs:jsx,Fragment:'Fragment'};
  const imports={
   '../../ui/primitives':mock('primitive'),'../../ui/Markdown':{Markdown:'Markdown'},'./detail-runtime':{NativeColorWheel:'NativeColorWheel',parseNativeColorWheel:()=>null,MetadataItem:'MetadataItem',MetadataSidebar:'MetadataSidebar',DetailRuntime:'DetailRuntime'},
   '../../colorConverter':{buildColorConversionResults:()=>[]},'./action-runtime':{Action:'Action',ActionPanel:'ActionPanel',ActionPanelOverlay:'ActionPanelOverlay'},'./action-runtime-registry':{ActionRegistryContext:{Provider:'Registry'}},'./form-runtime':{FormRuntime:'FormRuntime'},'./grid-runtime':{GridRuntime:'GridRuntime'},'./list-runtime':{ListRuntime:'ListRuntime'},'./hooks':{setRuntimeContext:()=>{},NavigationContext:{Provider:'Navigation'}}};
  if(name in imports)return imports[name];throw new Error('QA unknown host import denied '+name);
 };
 const compiled=ts.transpileModule(fs.readFileSync(file,'utf8')+'\n'+extras,{compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2022,jsx:ts.JsxEmit.ReactJSX,esModuleInterop:true}}).outputText;
 const context=vm.createContext({module,exports:module.exports,require:requireMock,console:{log:deny},window:{addEventListener:()=>{},removeEventListener:()=>{},tezbar:new Proxy({},{get:()=>deny})},fetch:deny,setTimeout:deny,process:undefined},{codeGeneration:{strings:false,wasm:false}});
 new vm.Script(compiled,{filename:file}).runInContext(context,{timeout:1000});
 return {exports:module.exports,render(fn){i=0;const n=fn();const run=effects;effects=[];for(const f of run)f();return n;}};
}
function walk(n){if(!n||typeof n!=='object')return [];if(Array.isArray(n))return n.flatMap(walk);return [n,...walk(n.props?.children)];}
const views=JSON.parse(fs.readFileSync(path.join(root,'.qa/process/views.json')));
const formPath='src/renderer/src/raycast-api/form-runtime.tsx';
const pinned=loader(path.join(root,'qa/fixtures/host',formPath),'export {collectFormItems,collectOptionData};');
const patched=loader(path.join(root,'.qa/patched-host',formPath),'export {collectFormItems,collectOptionData};');
let count=0;
for(const v of views){const original=pinned.exports.collectFormItems(v.root),changed=patched.exports.collectFormItems(v.root);assert(original.length>0);assert.equal(original.length,changed.length);const fields=changed.filter(x=>x.type==='field').map(x=>x.field);assert(fields.every(f=>f.actionId));if(v.title==='Base Converter'&&fields.some(f=>f.type==='Form.Dropdown'))assert(fields.filter(f=>f.type==='Form.Dropdown').every(f=>f.options.length===35));count++;}
assert.equal(pinned.exports.collectFormItems({type:'Unknown',children:[]}).length,0);
assert.equal(pinned.exports.collectFormItems({type:'Form.TextField',props:{id:' ',title:12}})[0].field.id,'Form.TextField:0');
// Execute actual patched FormRuntime callbacks, not a handwritten state mirror.
const initial=structuredClone(views.find(v=>v.title==='Code Converter').root);const ref={current:null};let changes=[],submits=[],focus=[];
const props={root:initial,title:'Code Converter',onBack(){},onSubmitForm:v=>submits.push(v),onChangeField:(id,value,values)=>changes.push({id,value,values}),onInvokeAction:(...v)=>focus.push(v),onOpenActions(){}};
let tree=patched.render(()=>patched.exports.FormRuntime(props,ref));
let input=walk(tree).find(n=>n.type==='input'&&n.props.id.includes('native'));
input.props.onChange({target:{value:'A'}});input.props.onChange({target:{value:'B'}});input.props.onChange({target:{value:'A'}});
assert.equal(changes[2].values._inputRevision,3);assert.equal(changes[2].values.native,'A');assert.equal(changes[2].values._lastChangedField,'native');assert(Object.keys(changes[2].values).length>=11);
input.props.onFocus();assert.equal(focus.length,1);assert(focus[0][0].includes(':focus:native'));
tree.props.onSubmit({preventDefault(){}});assert.equal(submits[0].native,'A');assert.equal(ref.current.getValues()._inputRevision,3);
const delayed=structuredClone(initial);delayed.props.inputRevision=2;delayed.children.find(n=>n.props?.id==='native').props.value='B';props.root=delayed;
patched.render(()=>patched.exports.FormRuntime(props,ref));tree=patched.render(()=>patched.exports.FormRuntime(props,ref));input=walk(tree).find(n=>n.type==='input'&&n.props.id.includes('native'));assert.equal(input.props.value,'A');
// Actual List parsers and negative nodes. The product itself uses Form only.
const list=loader(path.join(root,'qa/fixtures/host/src/renderer/src/raycast-api/list-runtime.tsx'),'export {parseListRows,parseListAccessory};').exports;
assert.equal(list.parseListRows({type:'List',children:[{type:'List.Item',props:{id:'a',title:'A'}},{type:'Form.TextField',props:{title:'ignored'}}]}).length,1);
for(const n of [null,{},'x',{type:'Form.Dropdown'}])assert.equal(list.parseListAccessory(n),null);
assert.equal(list.parseListAccessory({type:'List.Dropdown',children:[{type:'List.Dropdown.Item',props:{title:'One',value:'1'}}]}).options[0].value,'1');
// Actual surface overlay must submit every form action with current fields.
const surface=loader(path.join(root,'.qa/patched-host/src/renderer/src/raycast-api/index.tsx'),'export {rootKind};');
assert.equal(surface.exports.rootKind(initial),'form');assert.equal(surface.exports.rootKind({type:'List'}),'list');
let invoked=[];const sp={sessionId:'fixture',title:'Code Converter',extensionId:manifest.id,commandName:'code-converter',root:initial,actions:[{id:'primary',title:'First'},{id:'secondary',title:'Second'}],onBack(){},onSearchTextChanged(){},onLoadMore(){},onInvokeAction:(...v)=>invoked.push(v)};
tree=surface.render(()=>surface.exports.ExtensionRuntimeSurface(sp));let form=walk(tree).find(n=>n.type==='FormRuntime');form.props.ref.current={getValues:()=>({native:'fresh',_inputRevision:7})};form.props.onOpenActions();tree=surface.render(()=>surface.exports.ExtensionRuntimeSurface(sp));const overlay=walk(tree).find(n=>n.type==='ActionPanelOverlay');overlay.props.onExecute(sp.actions[1]);assert.equal(invoked[0][1].native,'fresh');assert.equal(invoked[0][1]._inputRevision,7);
console.log(JSON.stringify({schema:'actual draft2020-12',schemaNegativeCases:6,actualFormViews:count,delayedDraft:'A/B/A preserved',wholeFormActions:true,listParsers:true,hostBuild:'not-run'}));
