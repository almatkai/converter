use converter::domain::Domain;
use serde_json::Value;
#[test]
fn pinned_executable_typescript_hook_oracle() {
    let cases: Vec<Value> = serde_json::from_str(include_str!("../qa/oracle.json")).unwrap();
    let mut failures = Vec::new();
    for (i, c) in cases.iter().enumerate() {
        let command = c["command"].as_str().unwrap();
        let comma = c["comma"].as_bool().unwrap_or(true);
        let mut d = Domain::new(command, comma);
        if let Some(text) = c["autopaste"].as_str() {
            d.autopaste(command, text, c["advance"].as_bool().unwrap_or(true), comma)
                .unwrap();
        }
        for step in c["steps"].as_array().unwrap_or(&vec![]) {
            let key = step["key"].as_str().unwrap();
            let raw = step["value"].as_str().unwrap();
            if key == "fromBase" || key == "toBase" {
                d.choose_base(key, raw).unwrap();
            } else {
                d.edit(command, key, raw, comma).unwrap();
            }
        }
        let pass = if c["throws"].is_string() {
            d.error.is_some()
        } else {
            serde_json::to_value(&d.values).unwrap() == c["expected"]
                && serde_json::to_value(&d.best).unwrap() == c["best"]
        };
        if !pass {
            failures.push(format!(
                "case {i}: {}\nexpected {} / {} / {}\nactual {} / {:?} / {:?}",
                serde_json::to_string(c).unwrap(),
                c["expected"],
                c["best"],
                c["throws"],
                serde_json::to_string(&d.values).unwrap(),
                d.best,
                d.error
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures
            .iter()
            .take(12)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}
