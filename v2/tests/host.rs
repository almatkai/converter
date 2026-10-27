// The actual pinned host functions, extracted without rewriting. Only the
// discovered verified-package list is stubbed; no host installation or app build.
#[allow(dead_code, clippy::all)]
mod actual {
    include!("../qa/host-functions.rs");
    use serde_json::{json, Value};
    #[test]
    fn runtime_manifest_and_negative_cases() {
        let manifest: Value = serde_json::from_str(include_str!("../extension.json")).unwrap();
        process_install::validate(&manifest).unwrap();
        for (key, value) in [
            ("schemaVersion", json!(2)),
            ("runtime", json!("javascript")),
            ("execution", json!("embedded")),
            ("protocolVersion", json!(2)),
            ("id", json!("com.tezbar.native.wrong")),
            ("capabilities", json!(["host.arbitrary"])),
        ] {
            let mut bad = manifest.clone();
            bad[key] = value;
            assert!(process_install::validate(&bad).is_err(), "{key}");
        }
        for (key, value) in [
            ("refreshMs", json!(499)),
            ("timeoutMs", json!(300001)),
            ("concurrency", json!("parallel")),
        ] {
            let mut bad = manifest.clone();
            bad["commands"][0][key] = value;
            assert!(process_install::validate(&bad).is_err());
        }
        for path in [
            "../bad",
            "/absolute",
            "bin/../../bad",
            "bin\\bad",
            "bin/CON.exe",
            "a//b",
            "a:b",
            "a/.",
            "a/..",
            "a/b.",
        ] {
            assert!(process_install::relative(path).is_err());
        }
        assert!(process_install::relative("resources/extension-icon.png").is_ok());
    }
    #[test]
    fn verified_exact_aliases_and_covered_rust_commands_only() {
        use registry::*;
        let manifest: Value = serde_json::from_str(include_str!("../extension.json")).unwrap();
        let mut ctx = CoreContext {
            entries: vec![Installed {
                id: "com.tezbar.native.converter".into(),
                slug: "converter".into(),
                path: "canonical".into(),
                resource_path: "active-version".into(),
                manifest,
            }],
        };
        for alias in [
            "com.tezbar.native.converter",
            "com.tezbar.converter",
            "raycast.converter",
        ] {
            for cmd in ["base-converter", "code-converter", "byte-converter"] {
                assert_eq!(
                    canonical_command_id(&ctx, alias, cmd).as_deref(),
                    Some("com.tezbar.native.converter")
                );
            }
        }
        for alias in ["com.tezbar.converter.evil", "raycast.other", "converter"] {
            assert!(canonical_command_id(&ctx, alias, "base-converter").is_none());
        }
        assert!(canonical_command_id(&ctx, "com.tezbar.converter", "uncovered").is_none());
        ctx.entries[0].manifest["commands"][0]["runtime"] = json!("javascript");
        assert!(canonical_command_id(&ctx, "com.tezbar.converter", "base-converter").is_none());
        ctx.entries[0].manifest["healthError"] = json!("bad hash");
        assert!(canonical_command_id(&ctx, "com.tezbar.converter", "code-converter").is_none());
    }
    #[test]
    fn resources_resolve_from_active_version_without_changing_canonical_root() {
        use registry::*;
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let entry = Installed {
            id: "com.tezbar.native.converter".into(),
            slug: "converter".into(),
            path: root.join(".qa/canonical"),
            resource_path: root.clone(),
            manifest: json!({"icon":"resources/extension-icon.png"}),
        };
        assert_eq!(
            test_icon_path(&entry),
            Some(
                root.join("resources/extension-icon.png")
                    .to_string_lossy()
                    .into()
            )
        );
        assert_eq!(
            test_command_icon_path(&entry, &json!({"icon":"resources/extension-icon.png"})),
            test_icon_path(&entry)
        );
        assert_eq!(entry.path, root.join(".qa/canonical"));
        assert!(test_command_icon_path(&entry, &json!({"icon":"../escape.png"})).is_none());
    }
}
