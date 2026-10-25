// GENERATED from exact host fixtures by scripts/host-fixtures.py.
pub mod process_install {
use serde_json::Value;
use std::{collections::HashSet,path::PathBuf};
const CAPABILITY_GROUPS: &[&str] = &[
    "clipboard",
    "shell",
    "selection",
    "window",
    "dialog",
    "screen",
    "apps",
    "settings",
    "ai",
    "storage",
    "background",
    "tray",
    "notifications",
    "safety",
    "launcher",
];

/// Closed service allowlist shared by manifest validation and runtime grants.
pub(super) const METHODS: &[&str] = &[
    "host.clipboard.read",
    "host.clipboard.write",
    "host.clipboard.writeImage",
    "host.clipboard.writeFiles",
    "host.clipboard.paste",
    "host.clipboard.pasteText",
    "host.shell.open",
    "host.shell.openWithApp",
    "host.shell.reveal",
    "host.selection.read",
    "host.window.visibility",
    "host.window.suppressBlur",
    "host.dialog.confirm",
    "host.screen.capture",
    "host.apps.list",
    "host.settings.open",
    "host.ai.ask",
    "host.background.retain",
    "host.background.release",
    "host.tray.set",
    "host.tray.remove",
    "host.notifications.show",
    "host.notifications.remove",
    "host.safety.authorize",
    "host.safety.record",
    "host.launcher.publish",
];
fn valid_capability(cap: &str) -> bool {
    CAPABILITY_GROUPS.contains(&cap)
        || METHODS
            .iter()
            .any(|m| *m == cap || m.strip_prefix("host.") == Some(cap))
}
pub(super) fn component(s: &str) -> Result<(), String> {
    if s.is_empty()
        || s.len() > 200
        || !s.as_bytes()[0].is_ascii_alphanumeric()
        || !s
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
    {
        return Err("Invalid package component".into());
    }
    Ok(())
}
pub(super) fn relative(s: &str) -> Result<PathBuf, String> {
    // Reject cross-platform path syntax even when verifying on Unix.
    if s.is_empty()
        || s.len() > 1024
        || s.contains(['\\', ':', '\0'])
        || s.chars().any(|c| c.is_control())
        || s.starts_with('/')
        || s.split('/')
            .any(|p| p.is_empty() || p == "." || p == ".." || p.ends_with([' ', '.']))
    {
        return Err(format!("Unsafe package path: {s}"));
    }
    for part in s.split('/') {
        let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        if [
            "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
            "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
        ]
        .contains(&stem.as_str())
        {
            return Err("Reserved package filename".into());
        }
    }
    Ok(PathBuf::from(s))
}
pub(super) fn validate(manifest: &Value) -> Result<(), String> {
    if manifest["schemaVersion"] != 3
        || manifest["runtime"] != "rust"
        || manifest["execution"] != "process"
        || manifest["protocolVersion"] != 1
    {
        return Err("Extension needs a schema-3 process package (protocol 1); schema-2 markers need upgrade".into());
    }
    let slug = manifest["slug"].as_str().ok_or("Missing slug")?;
    component(slug)?;
    if manifest["id"] != format!("com.tezbar.native.{slug}") {
        return Err("Package identity mismatch".into());
    }
    let version = manifest["version"].as_str().ok_or("Missing version")?;
    component(version)?;
    if let Some(min) = manifest["minHostVersion"].as_str() {
        let min = semver::Version::parse(min).map_err(|_| "Invalid minimum host version")?;
        let host = semver::Version::parse(env!("CARGO_PKG_VERSION")).map_err(|e| e.to_string())?;
        if host < min {
            return Err("Package requires a newer Tezbar host".into());
        }
    }
    let caps = manifest["capabilities"]
        .as_array()
        .ok_or("Missing capabilities")?;
    if caps
        .iter()
        .any(|c| !c.as_str().is_some_and(|s| valid_capability(s)))
    {
        return Err("Unknown capability".into());
    }
    let commands = manifest["commands"].as_array().ok_or("Missing commands")?;
    let mut names = HashSet::new();
    for c in commands {
        let name = c["name"].as_str().ok_or("Missing command name")?;
        component(name)?;
        if !names.insert(name) {
            return Err("Duplicate command".into());
        }
        if c.get("concurrency")
            .is_some_and(|v| v != "serialize" && v != "reject-while-busy")
        {
            return Err("Invalid concurrency policy".into());
        }
        if c.get("timeoutMs")
            .is_some_and(|v| !v.as_u64().is_some_and(|n| (1..=300000).contains(&n)))
        {
            return Err("Invalid command deadline".into());
        }
        if c.get("refreshMs")
            .is_some_and(|v| !v.is_null() && !v.as_u64().is_some_and(|n| n >= 500))
        {
            return Err("Invalid refresh interval".into());
        }
    }
    super::launcher::validate(manifest)?;
    if let Some(paths) = manifest.get("migrationPaths") {
        let paths = paths.as_object().ok_or("Invalid migration paths")?;
        if paths.len() > 32 {
            return Err("Too many migration paths".into());
        }
        for (key, path) in paths {
            component(key)?;
            relative(path.as_str().ok_or("Invalid migration path")?)?;
        }
    }
    if let Some(probe) = manifest.get("startupProbe") {
        component(probe["key"].as_str().ok_or("Startup probe requires key")?)?;
        let paths = probe["migrationPaths"]
            .as_array()
            .ok_or("Startup probe requires migration path keys")?;
        if paths.is_empty()
            || paths.iter().any(|p| {
                p.as_str()
                    .is_none_or(|k| manifest["migrationPaths"].get(k).is_none())
            })
        {
            return Err("Startup probe references undeclared migration path".into());
        }
        if !caps
            .iter()
            .any(|v| v == "background" || v == "background.retain" || v == "host.background.retain")
        {
            return Err("Startup probe requires background capability".into());
        }
    }
    Ok(())
}
}
pub mod launcher {
use serde_json::Value;
use std::collections::HashSet;
use super::process_install;
fn bounded_text<'a>(v: &'a Value, name: &str, max: usize) -> Result<&'a str, String> {
    v[name]
        .as_str()
        .filter(|s| !s.is_empty() && s.len() <= max && !s.chars().any(char::is_control))
        .ok_or_else(|| format!("Invalid launcher {name}"))
}
pub(super) fn validate(m: &Value) -> Result<(), String> {
    let ds = match m.get("launcherContributions") {
        Some(ds) => ds
            .as_array()
            .filter(|a| a.len() <= 8)
            .ok_or("Too many launcher contributions")?
            .as_slice(),
        None => &[],
    };
    let mut ids = HashSet::new();
    for d in ds {
        let id = bounded_text(d, "id", 128)?;
        process_install::component(id)?;
        if !ids.insert(id) {
            return Err("Duplicate launcher contribution".into());
        }
        let command = bounded_text(d, "commandName", 128)?;
        let cmd = m["commands"]
            .as_array()
            .and_then(|a| a.iter().find(|c| c["name"] == command))
            .ok_or("Launcher must target its own command")?;
        let surfaces = d["surfaces"]
            .as_array()
            .filter(|a| !a.is_empty() && a.len() <= 2)
            .ok_or("Invalid launcher surfaces")?;
        if surfaces.iter().any(|s| s != "query" && s != "argument") {
            return Err("Invalid launcher surface".into());
        }
        if surfaces.iter().any(|s| s == "argument") {
            let name = bounded_text(d, "argumentName", 128)?;
            if !cmd["arguments"]
                .as_array()
                .is_some_and(|a| a.iter().any(|a| a["name"] == name))
            {
                return Err("Launcher argument must belong to its command".into());
            }
        }
        if d["limit"].as_u64().is_none_or(|n| !(1..=32).contains(&n))
            || d["timeoutMs"]
                .as_u64()
                .is_none_or(|n| !(50..=3000).contains(&n))
        {
            return Err("Invalid launcher bounds".into());
        }
        let selectors = d["selectors"]
            .as_object()
            .ok_or("Missing launcher selectors")?;
        if selectors
            .keys()
            .any(|k| !["terms", "digitRun"].contains(&k.as_str()))
        {
            return Err("Unknown launcher selector".into());
        }
        if let Some(terms) = selectors.get("terms") {
            let terms = terms
                .as_array()
                .filter(|a| a.len() <= 32)
                .ok_or("Invalid launcher terms")?;
            if terms.iter().any(|t| {
                t.as_str().is_none_or(|s| {
                    s.is_empty() || s.len() > 128 || s.chars().any(char::is_control)
                })
            }) {
                return Err("Invalid launcher term".into());
            }
        }
        if let Some(n) = selectors.get("digitRun") {
            let min = n["min"].as_u64().ok_or("Invalid digit selector")?;
            let max = n["max"].as_u64().ok_or("Invalid digit selector")?;
            if !(1..=16).contains(&min) || !(min..=16).contains(&max) {
                return Err("Invalid digit selector".into());
            }
        }
        if d["index"] == true
            && !m["capabilities"].as_array().is_some_and(|a| {
                a.iter().any(|v| {
                    v == "launcher" || v == "launcher.publish" || v == "host.launcher.publish"
                })
            })
        {
            return Err("Launcher index requires publish capability".into());
        }
        if d.get("index").is_some_and(|v| !v.is_boolean()) {
            return Err("Invalid launcher index policy".into());
        }
        if let Some(paths) = d.get("bootstrapMigrationPaths") {
            if d["index"] != true
                || paths.as_array().is_none_or(|a| {
                    a.is_empty()
                        || a.len() > 8
                        || a.iter().any(|p| {
                            p.as_str()
                                .is_none_or(|k| m["migrationPaths"].get(k).is_none())
                        })
                })
            {
                return Err("Invalid launcher bootstrap grants".into());
            }
        }
    }
    for cmd in m["commands"].as_array().into_iter().flatten() {
        if let Some(l) = cmd.get("launcher") {
            if let Some(id) = l.get("id") {
                let command = cmd["name"].as_str().unwrap_or("");
                let valid = std::iter::once(&m["id"])
                    .chain(m["replaces"].as_array().into_iter().flatten())
                    .filter_map(Value::as_str)
                    .any(|ext| id == &format!("extcmd:{ext}:{command}"));
                if !valid {
                    return Err("Invalid launcher command ID".into());
                }
            }
            if l["recommendation"].as_u64().is_none_or(|n| n > 2000) {
                return Err("Invalid launcher recommendation".into());
            }
            if let Some(a) = l.get("inlineArgument") {
                let name = bounded_text(a, "name", 128)?;
                if !cmd["arguments"]
                    .as_array()
                    .is_some_and(|v| v.iter().any(|v| v["name"] == name))
                {
                    return Err("Invalid inline argument association".into());
                }
                bounded_text(a, "title", 128)?;
                bounded_text(a, "prompt", 256)?;
                if let Some(v) = a.get("validation") {
                    if v["kind"] != "integer"
                        || v["maxLength"]
                            .as_u64()
                            .is_none_or(|n| !(1..=16).contains(&n))
                        || v["min"]
                            .as_u64()
                            .zip(v["max"].as_u64())
                            .is_none_or(|(min, max)| min > max)
                    {
                        return Err("Invalid inline validation".into());
                    }
                    bounded_text(v, "message", 256)?;
                }
            }
        }
    }
    Ok(())
}
}
pub mod registry {
use serde_json::Value;
use std::path::{Path,PathBuf,Component};
#[derive(Clone)]
pub struct Installed { pub id:String,pub slug:String,pub path:PathBuf,pub resource_path:PathBuf,pub manifest:Value }
pub struct CoreContext {pub entries:Vec<Installed>}
fn collect(ctx:&CoreContext)->Vec<Installed> {ctx.entries.clone()}
fn installed(ctx:&CoreContext,id:&str)->Result<Installed,String> {ctx.entries.iter().find(|e|e.id==id).cloned().ok_or("not installed".into())}
fn platform() -> &'static str {
    if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else {
        "macOS"
    }
}
fn compatible(manifest: &Value, command: bool) -> bool {
    if command && manifest.get("platforms").is_none() {
        return true;
    }
    let Some(platforms) = manifest["platforms"]
        .as_array()
        .or_else(|| manifest["tezbar"]["platforms"].as_array())
    else {
        return platform() == "macOS";
    };
    platforms
        .iter()
        .filter_map(Value::as_str)
        .any(|value| match value.to_lowercase().as_str() {
            "macos" | "darwin" | "mac" => platform() == "macOS",
            "windows" | "win32" | "win" => platform() == "Windows",
            "linux" => platform() == "Linux",
            _ => false,
        })
}
fn fallback<'a>(value: &'a Value, key: &str, default: &'a str) -> &'a str {
    value[key]
        .as_str()
        .filter(|s| !s.is_empty())
        .unwrap_or(default)
}
pub(super) fn installed_command(ctx: &CoreContext, id: &str, command: &str) -> Option<Installed> {
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
fn icon_path(extension: &Installed) -> Option<String> {
    let name = fallback(&extension.manifest, "icon", "icon.png");
    if name.starts_with("https://") || name.starts_with("http://") {
        return Some(name.into());
    }
    let relative = Path::new(name.strip_prefix("./").unwrap_or(name));
    if relative
        .components()
        .any(|c| !matches!(c, Component::Normal(_)))
    {
        return None;
    }
    let mut candidates = vec![
        extension.resource_path.join(relative),
        extension.resource_path.join("assets").join(relative),
    ];
    if extension.id.starts_with("com.tezbar.native.") {
        if let Some(packages) = extension.path.parent() {
            if let Some(extensions) = packages.parent() {
                let legacy = extensions.join(&extension.slug);
                candidates.push(legacy.join(relative));
                candidates.push(legacy.join("assets").join(relative));
            }
            let legacy = packages.join(format!("raycast.{}", extension.slug));
            candidates.push(legacy.join(relative));
            candidates.push(legacy.join("assets").join(relative));
        }
    }
    candidates
        .into_iter()
        .find(|p| p.is_file())
        .map(|p| p.to_string_lossy().into())
}
fn command_icon_path(extension: &Installed, cmd: &Value) -> Option<String> {
    let name = cmd["icon"].as_str().or_else(|| cmd["iconPath"].as_str())?;
    if name.starts_with("https://") || name.starts_with("http://") {
        return Some(name.into());
    }
    let relative = Path::new(name.strip_prefix("./").unwrap_or(name));
    if relative
        .components()
        .any(|c| !matches!(c, Component::Normal(_)))
    {
        return None;
    }
    let mut candidates = vec![
        extension.resource_path.join(relative),
        extension.resource_path.join("assets").join(relative),
    ];
    if extension.id.starts_with("com.tezbar.native.") {
        if let Some(packages) = extension.path.parent() {
            if let Some(extensions) = packages.parent() {
                let legacy = extensions.join(&extension.slug);
                candidates.push(legacy.join(relative));
                candidates.push(legacy.join("assets").join(relative));
            }
            let legacy = packages.join(format!("raycast.{}", extension.slug));
            candidates.push(legacy.join(relative));
            candidates.push(legacy.join("assets").join(relative));
        }
    }
    candidates
        .into_iter()
        .find(|p| p.is_file())
        .map(|p| p.to_string_lossy().into())
}
pub(super) fn test_icon_path(e:&Installed)->Option<String> {icon_path(e)}
pub(super) fn test_command_icon_path(e:&Installed,c:&Value)->Option<String> {command_icon_path(e,c)}
}
