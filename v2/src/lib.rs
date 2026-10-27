pub mod domain;
use domain::{Domain, CODES, UNITS};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};
use tezbar_extension_sdk::{BoxFuture, CoreResult, Extension, ExtensionContext};

const COMMANDS: &[&str] = &["base-converter", "code-converter", "byte-converter"];
#[derive(Default)]
struct Fence {
    epoch: u64,
    input: u64,
    prepared: Option<Retry>,
}
type Fences = Arc<Mutex<HashMap<String, Arc<Mutex<Fence>>>>>;
#[derive(Default)]
pub struct Converter {
    fences: Fences,
    next: AtomicU64,
}
struct Cleanup {
    fences: Fences,
    session: String,
    fence: Arc<Mutex<Fence>>,
}
impl Drop for Cleanup {
    fn drop(&mut self) {
        let mut map = self.fences.lock().unwrap();
        if map
            .get(&self.session)
            .is_some_and(|f| Arc::ptr_eq(f, &self.fence))
        {
            map.remove(&self.session);
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
struct State {
    nonce: u64,
    epoch: u64,
    input: u64,
    domain: Domain,
    retry: Option<Retry>,
    status: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
struct Retry {
    method: String,
    params: Value,
}
fn pref(payload: &Value, key: &str, default: bool) -> bool {
    match &payload["preferences"][key] {
        Value::Bool(v) => *v,
        Value::String(v) => v == "true",
        _ => default,
    }
}
fn sid(p: &Value) -> Result<&str, String> {
    p["sessionId"]
        .as_str()
        .filter(|s| !s.is_empty() && s.len() <= 256)
        .ok_or("Invalid sessionId".into())
}
fn ack(value: Value) -> Result<(), String> {
    let obj = value
        .as_object()
        .ok_or("Host did not return an acknowledgement object")?;
    if obj.get("ok") != Some(&Value::Bool(true))
        || obj.keys().any(|k| k != "ok" && k != "message")
        || obj.get("message").is_some_and(|m| !m.is_string())
    {
        return Err(
            "Invalid host acknowledgement (expected only ok:true and optional string message)"
                .into(),
        );
    }
    Ok(())
}
impl Converter {
    fn view(&self, command: &str, payload: &Value, s: &State) -> Value {
        let mut actions = Vec::new();
        let mut children = Vec::new();
        let id = |kind: &str, key: &str| format!("{}:{}:{kind}:{key}", s.nonce, s.epoch);
        let action = |key: &str, title: String, number: Option<&str>| {
            let mut a = json!({"id":id("action",key),"title":title,"kind":"action"});
            if let Some(key) = number {
                a["shortcut"] = json!({"modifiers":["cmd"],"key":key});
            }
            a
        };
        let copy = |key: &str, title: String, number: Option<&str>| {
            action(&format!("copy-{key}"), title, number)
        };
        let d = &s.domain;
        let mut field = |key: &str, title: &str, placeholder: &str| {
            children.push(json!({"type":"Form.TextField","props":{"id":key,"title":title,"value":d.values.get(key).cloned().unwrap_or_default(),"placeholder":placeholder,"actionId":id("edit",key),"focusActionId":id("focus",key),"error":if d.source==key {d.error.clone()}else{None}}}));
        };
        match command {
            "base-converter" => {
                actions.push(copy(
                    "focused",
                    format!(
                        "Copy Base {}",
                        if d.focused == "input" {
                            d.from_base.to_string()
                        } else {
                            d.focused.strip_prefix("base").unwrap_or("10").into()
                        }
                    ),
                    None,
                ));
                for (key, title, shortcut) in [
                    ("base10", "Copy Base 10", "1"),
                    ("base2", "Copy Base 2", "2"),
                    ("base4", "Copy Base 4", "3"),
                    ("base8", "Copy Base 8", "4"),
                    ("base16", "Copy Base 16", "5"),
                    ("base32", "Copy Base 32", "6"),
                ] {
                    actions.push(copy(key, title.into(), Some(shortcut)));
                }
                if pref(payload, "advanceView", true) {
                    actions.push(copy("output", "Copy Output".into(), Some("o")));
                }
                for b in [10, 2, 4, 8, 16, 32] {
                    field(&format!("base{b}"), &b.to_string(), "");
                }
                if pref(payload, "advanceView", true) {
                    let mut advanced = vec![];
                    for (key, title, b) in [
                        ("fromBase", "From Base", d.from_base),
                        ("toBase", "To Base", d.to_base),
                    ] {
                        advanced.push(json!({"type":"Form.Dropdown","props":{"id":key,"title":title,"value":b.to_string(),"actionId":id("edit",key)},"children":(2..=36).map(|b|json!({"type":"Form.Dropdown.Item","props":{"title":b.to_string(),"value":b.to_string()}})).collect::<Vec<_>>()}));
                    }
                    advanced.push(json!({"type":"Form.TextField","props":{"id":"input","title":"Input","value":d.values.get("input"),"actionId":id("edit","input"),"focusActionId":id("focus","input"),"error":if d.source=="input" {d.error.clone()}else{None}}}));
                    advanced.push(json!({"type":"Form.Description","props":{"title":"Output","text":d.values.get("output").cloned().unwrap_or_default()+"\n"}}));
                    let separator = json!({"type":"Form.Separator"});
                    if payload["preferences"]["advanceViewLocation"] == "Top" {
                        advanced.push(separator);
                        advanced.extend(children);
                        children = advanced;
                    } else {
                        children.push(separator);
                        children.extend(advanced);
                    }
                }
            }
            "byte-converter" => {
                if let Some(best) = &d.best {
                    actions.push(copy("best", format!("Copy {best}"), None));
                }
                let focus = format!(
                    "{} {}",
                    d.values.get(&d.focused).cloned().unwrap_or_default(),
                    d.focused
                );
                let mut a = copy("focused", format!("Copy {focus}"), Some("enter"));
                a["shortcut"]["modifiers"] = json!(["cmd", "shift"]);
                actions.push(a);
                for (i, (key, _)) in UNITS.iter().enumerate() {
                    actions.push(copy(
                        key,
                        format!(
                            "Copy {} {key}",
                            d.values.get(*key).cloned().unwrap_or_default()
                        ),
                        Some(&(i + 1).to_string()),
                    ));
                    field(key, key, "0");
                }
            }
            _ => {
                for (i, (key, title)) in CODES.iter().enumerate() {
                    let action_title = if *key == "entity" {
                        "Entity"
                    } else if *key == "utf8" {
                        "Utf-8"
                    } else if *key == "ascii" {
                        "Ascii"
                    } else {
                        title
                    };
                    actions.push(copy(
                        key,
                        format!("Copy {action_title}"),
                        Some(&(i + 1).to_string()),
                    ));
                    field(
                        key,
                        title,
                        match *key {
                            "native" => "String",
                            "unicode" => "\\u0031",
                            "base64" => "MQ==",
                            "utf8" => "\\xE4\\xBD\\xA0\\xE5\\xA5\\xBD",
                            "ascii" => "Number",
                            "hex" => "0x0",
                            "decimal" => "0",
                            "url" => "https%3A%2F%2Fwww.raycast.com",
                            _ => "&#xXXXX;",
                        },
                    );
                }
            }
        }
        if command == "code-converter" {
            for child in &mut children {
                let info = match child["props"]["id"].as_str() {
                    Some("utf8") => Some("Letters and numbers will not be transcode to UTF-8"),
                    Some("ascii") => Some("Only ASCII greater than 13 is supported"),
                    Some("url") => Some("URLs encoded with encodeURIComponent"),
                    Some("entity") => Some("HTML entity encoded with &#xXXXX;"),
                    _ => None,
                };
                if let Some(info) = info {
                    child["props"]["info"] = json!(info);
                }
            }
        }
        if command != "byte-converter" {
            let mut a = action("clear", "Clear All".into(), None);
            a["shortcut"] = json!({"modifiers":["shift","cmd"],"key":"backspace"});
            actions.push(a);
        }
        if command != "byte-converter" {
            let mut a = action("command-settings", "Configure Command".into(), None);
            a["shortcut"] = json!({"modifiers":["shift","cmd"],"key":","});
            actions.push(a);
        }
        let mut a = action("extension-settings", "Configure Extension".into(), None);
        a["shortcut"] = json!({"modifiers":["opt","cmd"],"key":","});
        actions.push(a);
        if s.retry.is_some() {
            actions.push(action(
                "retry",
                "Retry — may repeat the previous host effect".into(),
                None,
            ));
        }
        if let Some(status) = &s.status {
            children
                .push(json!({"type":"Form.Description","props":{"title":"Status","text":status}}));
        }
        if d.error.is_some() {
            // Never permit copying old converted output after a failed edit.
            actions.retain(|a| !a["id"].as_str().unwrap().contains(":action:copy-"));
        }
        json!({"ok":true,"mode":"view","title":match command {"base-converter"=>"Base Converter","byte-converter"=>"Byte Converter",_=>"Code Converter"},"root":{"type":"Form","props":{"focusedFieldId":d.focused,"inputRevision":s.input},"children":children},"actions":actions,"refreshMs":null})
    }
    fn edit(&self, command: &str, p: &Value, s: &mut State, key: &str, value: &str) {
        let result = if key == "fromBase" || key == "toBase" {
            s.domain.choose_base(key, value)
        } else {
            s.domain
                .edit(command, key, value, pref(p, "asciiCommaSeparated", true))
        };
        if let Err(e) = result {
            s.domain.error = Some(e);
            s.domain.source = key.into();
            s.domain.raw = value.chars().take(domain::MAX_INPUT).collect();
            s.domain.values.insert(key.into(), s.domain.raw.clone());
        }
        s.retry = None;
        s.status = None;
    }
    async fn autofill(&self, ctx: &ExtensionContext, command: &str, p: &Value, s: &mut State) {
        s.retry = None;
        s.status = None;
        match ctx.host("host.clipboard.read", json!({})).await {
            Ok(Value::String(text)) => {
                if !text.is_empty() {
                    if let Err(e) = s.domain.autopaste(
                        command,
                        &text,
                        pref(p, "advanceView", true),
                        pref(p, "asciiCommaSeparated", true),
                    ) {
                        s.domain.error = Some(e);
                    }
                }
            }
            result => {
                s.retry = Some(Retry {
                    method: "host.clipboard.read".into(),
                    params: json!({}),
                });
                s.status=Some(match result {Err(e)=>format!("Clipboard auto-fill failed: {e}. Retry or edit a field to continue."),_=>"Clipboard auto-fill failed: host must return a string. Retry or edit a field to continue.".into()});
            }
        }
    }
    async fn effect(
        &self,
        ctx: &ExtensionContext,
        fence: &Arc<Mutex<Fence>>,
        s: &mut State,
        effect: Retry,
    ) {
        // Save exact prepared bytes before await. Cancellation keeps the authority
        // consumed; SDK staging cannot re-authorize the previous action ID.
        fence.lock().unwrap().prepared = Some(effect.clone());
        let result = ctx
            .host(&effect.method, effect.params.clone())
            .await
            .and_then(ack);
        match result {
            Ok(()) => {
                fence.lock().unwrap().prepared = None;
                s.retry = None;
                s.status = Some("Done".into());
            }
            Err(e) => {
                s.retry = Some(effect);
                s.status=Some(format!("{e}. The host effect may already have happened. Retry deliberately; it may repeat it."));
            }
        }
    }
}
impl Extension for Converter {
    fn id(&self) -> &'static str {
        "com.tezbar.native.converter"
    }
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
    fn commands(&self) -> &'static [&'static str] {
        COMMANDS
    }
    fn handle<'a>(
        &'a self,
        ctx: &'a ExtensionContext,
        command: &'a str,
        operation: &'a str,
        p: &'a Value,
        state: &'a mut Value,
    ) -> BoxFuture<'a, CoreResult> {
        Box::pin(async move {
            let session = sid(p)?.to_owned();
            if operation == "extension:dispose-session" {
                self.fences.lock().unwrap().remove(&session);
                return Ok(json!({"ok":true}));
            }
            if operation == "extension:run-command" {
                let fence = Arc::new(Mutex::new(Fence::default()));
                {
                    let mut map = self.fences.lock().unwrap();
                    if map.len() >= 128 || map.contains_key(&session) {
                        return Err("Session authority capacity/duplicate".into());
                    }
                    map.insert(session.clone(), fence.clone());
                }
                let cleanup = Cleanup {
                    fences: self.fences.clone(),
                    session: session.clone(),
                    fence: fence.clone(),
                };
                ctx.spawn(async move {
                    let _cleanup = cleanup;
                    std::future::pending::<()>().await;
                })?;
                let nonce = self.next.fetch_add(1, Ordering::SeqCst) + 1;
                let comma = pref(p, "asciiCommaSeparated", true);
                let mut s = State {
                    nonce,
                    epoch: 1,
                    input: 0,
                    domain: Domain::new(command, comma),
                    retry: None,
                    status: None,
                };
                fence.lock().unwrap().epoch = 1;
                if pref(p, "autoPaste", true) {
                    self.autofill(ctx, command, p, &mut s).await;
                }
                let view = self.view(command, p, &s);
                *state = serde_json::to_value(s).map_err(|e| e.to_string())?;
                return Ok(view);
            }
            let fence = self
                .fences
                .lock()
                .unwrap()
                .get(&session)
                .cloned()
                .ok_or("Session authority expired")?;
            let mut s: State =
                serde_json::from_value(state.clone()).map_err(|_| "No committed session state")?;
            if operation == "extension:refresh-session" {
                // A failed/cancelled effect invalidates old actions even when SDK discards staged state.
                s.epoch = {
                    let mut f = fence.lock().unwrap();
                    f.epoch += 1;
                    f.epoch
                };
                s.retry = fence.lock().unwrap().prepared.clone().or_else(|| {
                    s.retry
                        .clone()
                        .filter(|r| r.method == "host.clipboard.read")
                });
                if s.retry
                    .as_ref()
                    .is_some_and(|r| r.method != "host.clipboard.read")
                {
                    s.status=Some("Previous host operation was interrupted. It may already have happened. Retry deliberately; exact prepared bytes will be reused.".into());
                }
                let view = self.view(command, p, &s);
                *state = serde_json::to_value(s).unwrap();
                return Ok(view);
            }
            if operation == "extension:search-text-changed" {
                fence.lock().unwrap().prepared = None;
                s.epoch = {
                    let mut f = fence.lock().unwrap();
                    f.epoch += 1;
                    f.epoch
                };
                let raw = p["searchText"].as_str().ok_or("Missing searchText")?;
                let key = match command {
                    "base-converter" => "base10",
                    "byte-converter" => "bits",
                    _ => "native",
                };
                self.edit(command, p, &mut s, key, raw);
            } else if operation == "extension:invoke-action" {
                let action_id = p["actionId"].as_str().ok_or("Missing actionId")?;
                let mut parts = action_id.splitn(4, ':');
                let nonce = parts
                    .next()
                    .and_then(|v| v.parse::<u64>().ok())
                    .ok_or("Invalid action nonce")?;
                let epoch = parts
                    .next()
                    .and_then(|v| v.parse::<u64>().ok())
                    .ok_or("Invalid action revision")?;
                let kind = parts.next().ok_or("Invalid action kind")?;
                let key = parts.next().ok_or("Invalid action key")?;
                if nonce != s.nonce {
                    return Err("Foreign/stale action".into());
                }
                let fv = &p["formValues"];
                let input_rev = fv["_inputRevision"].as_u64();
                {
                    let mut f = fence.lock().unwrap();
                    if let ("edit", Some(rev)) = (kind, input_rev) {
                        if rev <= f.input {
                            return Err("Delayed field revision rejected".into());
                        }
                        f.input = rev;
                        s.input = rev;
                    } else if epoch != s.epoch || epoch != f.epoch {
                        return Err(
                            "Stale or consumed action. Refresh the view to continue.".into()
                        );
                    }
                    f.epoch += 1;
                    s.epoch = f.epoch;
                } // One-use authority is consumed before any parse failure or await.
                match kind {
                    "edit" => {
                        fence.lock().unwrap().prepared = None;
                        let raw = fv["value"]
                            .as_str()
                            .or_else(|| fv[key].as_str())
                            .ok_or("Field value must be a string")?;
                        self.edit(command, p, &mut s, key, raw);
                    }
                    "focus" => {
                        if !s.domain.values.contains_key(key) || key == "output" {
                            return Err("Invalid focused field".into());
                        }
                        s.domain.focused = key.into();
                    }
                    "action" => {
                        // The host patch sends the latest whole form, including the last
                        // edited field. Apply it before preparing a copy, even if its
                        // queued field event has not arrived yet.
                        if let (Some(rev), Some(field)) =
                            (input_rev, fv["_lastChangedField"].as_str())
                        {
                            let apply = {
                                let mut f = fence.lock().unwrap();
                                if rev > f.input {
                                    f.input = rev;
                                    s.input = rev;
                                    true
                                } else {
                                    false
                                }
                            };
                            if apply {
                                fence.lock().unwrap().prepared = None;
                                let raw = fv[field].as_str().ok_or("Latest draft missing")?;
                                self.edit(command, p, &mut s, field, raw);
                            }
                        }
                        if let Some(key) = key.strip_prefix("copy-") {
                            if s.domain.error.is_some() {
                                return Err(
                                    "Conversion failed; edit the draft before copying".into()
                                );
                            }
                            let d = &s.domain;
                            let key = if key == "focused" {
                                d.focused.as_str()
                            } else {
                                key
                            };
                            let text = if key == "best" {
                                d.best.clone().ok_or("No best unit")?
                            } else if command == "byte-converter" {
                                if !UNITS.iter().any(|(k, _)| *k == key) {
                                    return Err("Invalid copy field".into());
                                }
                                format!("{} {key}", d.values.get(key).ok_or("Missing value")?)
                            } else {
                                d.values.get(key).cloned().ok_or("Invalid copy field")?
                            };
                            self.effect(
                                ctx,
                                &fence,
                                &mut s,
                                Retry {
                                    method: "host.clipboard.write".into(),
                                    params: json!({"text":text}),
                                },
                            )
                            .await;
                        } else {
                            match key {
                                "clear" if command != "byte-converter" => {
                                    fence.lock().unwrap().prepared = None;
                                    let (from, to, focused) = (
                                        s.domain.from_base,
                                        s.domain.to_base,
                                        s.domain.focused.clone(),
                                    );
                                    s.domain =
                                        Domain::new(command, pref(p, "asciiCommaSeparated", true));
                                    if command == "base-converter" {
                                        s.domain.choose_base("fromBase", &from.to_string())?;
                                        s.domain.choose_base("toBase", &to.to_string())?;
                                        s.domain.focused = focused;
                                    }
                                    s.retry = None;
                                    s.status = None;
                                }
                                "command-settings" | "extension-settings" => {
                                    self.effect(
                                        ctx,
                                        &fence,
                                        &mut s,
                                        Retry {
                                            method: "host.settings.open".into(),
                                            params: json!({"surface":"settings-extensions"}),
                                        },
                                    )
                                    .await
                                }
                                "retry" => {
                                    let retry = s.retry.take().ok_or("No prepared retry")?;
                                    if retry.method == "host.clipboard.read" {
                                        self.autofill(ctx, command, p, &mut s).await;
                                    } else {
                                        self.effect(ctx, &fence, &mut s, retry).await;
                                    }
                                }
                                _ => return Err("Unknown action".into()),
                            }
                        }
                    }
                    _ => return Err("Unknown action kind".into()),
                }
            } else {
                return Err("Unsupported operation".into());
            }
            let view = self.view(command, p, &s);
            *state = serde_json::to_value(s).map_err(|e| e.to_string())?;
            Ok(view)
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn acknowledgements_are_closed() {
        for v in [
            Value::Null,
            json!(true),
            json!({"ok":false}),
            json!({"ok":true,"error":"x"}),
            json!({"ok":true,"message":null}),
            json!({"ok":true,"extra":1}),
        ] {
            assert!(ack(v).is_err());
        }
        assert!(ack(json!({"ok":true})).is_ok());
        assert!(ack(json!({"ok":true,"message":"yes"})).is_ok());
    }
}
