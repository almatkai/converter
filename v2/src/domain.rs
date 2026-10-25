//! Mechanical Rust translation of the pinned TS utilities/hooks. No floating-point
//! replacement for the source's BigInt arithmetic. Bounds are explicit adaptations.
use base64::{engine::general_purpose::STANDARD, Engine};
use num_bigint::{BigInt, BigUint, Sign};
use num_traits::{ToPrimitive, Zero};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const MAX_INPUT: usize = 4096;
pub const UNITS: [(&str, usize); 8] = [
    ("bits", 0),
    ("Bytes", 3),
    ("KB", 13),
    ("MB", 23),
    ("GB", 33),
    ("TB", 43),
    ("PB", 53),
    ("EB", 63),
];
pub const CODES: [(&str, &str); 9] = [
    ("native", "Native"),
    ("unicode", "Unicode"),
    ("base64", "Base64"),
    ("utf8", "UTF-8"),
    ("ascii", "ASCII"),
    ("hex", "Hex"),
    ("decimal", "Decimal"),
    ("url", "Encoded URL"),
    ("entity", "HTML entity"),
];
fn js_space(c: char) -> bool {
    c.is_whitespace() && c != '\u{85}' || c == '\u{feff}'
}
pub fn trim(s: &str) -> &str {
    s.trim_matches(js_space)
}
pub fn bounded(s: &str) -> Result<(), String> {
    if s.len() > MAX_INPUT {
        Err(format!(
            "Input exceeds {MAX_INPUT} UTF-8 bytes; only a bounded draft prefix is retained."
        ))
    } else {
        Ok(())
    }
}
fn digit(c: char, radix: u32) -> Option<u32> {
    if c.is_ascii() {
        c.to_digit(radix)
    } else {
        None
    }
}
pub fn parse_big(s: &str, radix: u32) -> Option<BigUint> {
    if s.is_empty() {
        return None;
    }
    let mut n = BigUint::zero();
    for c in s.chars() {
        n = n * radix + digit(c, radix)?;
    }
    Some(n)
}
/// JS parseInt, including prefix/leading-junk/rounding once to Number.
fn parse_int(s: &str, radix: u32) -> f64 {
    let mut s = s.trim_start_matches(js_space);
    let negative = s.starts_with('-');
    if s.starts_with(['-', '+']) {
        s = &s[1..];
    }
    let mut radix = radix;
    if radix == 0 {
        radix = if s.starts_with("0x") || s.starts_with("0X") {
            16
        } else {
            10
        };
    }
    if radix == 16 && (s.starts_with("0x") || s.starts_with("0X")) {
        s = &s[2..];
    }
    let prefix: String = s
        .chars()
        .take_while(|&c| digit(c, radix).is_some())
        .collect();
    let n = parse_big(&prefix, radix)
        .and_then(|n| n.to_f64())
        .unwrap_or(f64::NAN);
    if negative {
        -n
    } else {
        n
    }
}
fn char_code(n: f64) -> u16 {
    if !n.is_finite() {
        0
    } else {
        n.trunc().rem_euclid(65536.0) as u16
    }
}
fn utf16_string(v: Vec<u16>) -> Result<String, String> {
    String::from_utf16(&v).map_err(|_| "Unpaired UTF-16 surrogate: original URI encoder throws; UTF-8 protocol cannot transport it. Draft retained; edit the input or Clear All.".into())
}
fn first_units(s: &str) -> impl Iterator<Item = u16> + '_ {
    s.chars().map(|c| c.encode_utf16(&mut [0; 2])[0])
}
fn decode_uri(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 2 >= bytes.len() {
                return String::new();
            }
            let Some(a) = digit(bytes[i + 1] as char, 16) else {
                return String::new();
            };
            let Some(b) = digit(bytes[i + 2] as char, 16) else {
                return String::new();
            };
            out.push((16 * a + b) as u8);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).unwrap_or_default()
}
fn decode_b64(s: &str) -> String {
    // Node Buffer accepts URL alphabet, junk, whitespace, absent padding; '='
    // terminates input. Incomplete one-character tails produce no bytes.
    let mut acc = 0_u32;
    let mut bits = 0;
    let mut out = Vec::new();
    for c in s.encode_utf16().map(|u| u as u8) {
        if c == b'=' {
            break;
        }
        let d = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            _ => continue,
        };
        acc = (acc << 6) | u32::from(d);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}
fn bigint_js(s: &str) -> Option<BigInt> {
    let s = trim(s);
    if s.is_empty() {
        return Some(BigInt::zero());
    }
    for (prefix, radix) in [
        ("0x", 16),
        ("0X", 16),
        ("0o", 8),
        ("0O", 8),
        ("0b", 2),
        ("0B", 2),
    ] {
        if let Some(s) = s.strip_prefix(prefix) {
            return parse_big(s, radix).map(|n| BigInt::from_biguint(Sign::Plus, n));
        }
    }
    let (sign, s) = if let Some(s) = s.strip_prefix('-') {
        (Sign::Minus, s)
    } else {
        (Sign::Plus, s.strip_prefix('+').unwrap_or(s))
    };
    parse_big(s, 10).map(|n| BigInt::from_biguint(sign, n))
}
pub fn decode(kind: &str, raw: &str, comma: bool) -> Result<String, String> {
    let s = trim(raw);
    let units = match kind {
        "native" => return Ok(raw.into()),
        "unicode" => s
            .split("\\u")
            .filter(|p| !p.is_empty())
            .map(|p| char_code(parse_int(p, 16)))
            .collect(),
        "base64" => return Ok(decode_b64(s)),
        "utf8" => return Ok(decode_uri(&raw.replace("\\x", "%"))),
        "url" => return Ok(decode_uri(raw)),
        "ascii" if comma => s
            .split(',')
            .filter_map(|p| {
                let n = parse_int(p, 0);
                (!n.is_nan()).then(|| char_code(n))
            })
            .collect(),
        "ascii" => {
            let mut out = Vec::new();
            let mut acc = String::new();
            for c in s.chars() {
                acc.push(c);
                let n = parse_int(&acc, 0);
                if n > 13.0 {
                    out.push(char_code(n));
                    acc.clear();
                }
            }
            out
        }
        "hex" => {
            let s = s.strip_prefix("0x").unwrap_or(s);
            let mut hexes = Vec::<f64>::new();
            let mut accumulated = 0.0;
            let mut unknown = false;
            for c in s.chars() {
                if let Some(d) = digit(c, 16) {
                    if unknown {
                        accumulated = accumulated * 16.0 + f64::from(d);
                    } else if !hexes.is_empty() || d != 0 {
                        hexes.push(f64::from(d));
                    }
                } else if c == '[' {
                    unknown = true;
                } else if c == ']' {
                    unknown = false;
                    hexes.push(0.0);
                    hexes.push(accumulated);
                    accumulated = 0.0;
                }
            }
            if hexes.len() % 2 == 1 {
                hexes.insert(0, 0.0);
            }
            hexes
                .chunks_exact(2)
                .map(|p| char_code(p[1] + 16.0 * p[0]))
                .collect()
        }
        "decimal" => {
            let Some(mut n) = bigint_js(s) else {
                return Ok(String::new());
            };
            let mut out = Vec::new();
            while n > BigInt::zero() {
                out.push((&n % BigInt::from(256)).to_u16().unwrap());
                n /= 256;
            }
            out.reverse();
            out
        }
        "entity" => s
            .replace(';', "")
            .split("&#x")
            .filter(|p| !p.is_empty())
            .map(|p| char_code(parse_int(p, 16)))
            .collect(),
        _ => return Err("Unknown encoding".into()),
    };
    utf16_string(units)
}
pub fn encode(kind: &str, native: &str, comma: bool) -> String {
    match kind {
        "native" => native.into(),
        "unicode" | "entity" => {
            use std::fmt::Write;
            let mut out = String::new();
            for u in native.encode_utf16() {
                if kind == "unicode" {
                    write!(out, "\\u{u:04x}").unwrap();
                } else {
                    write!(out, "&#x{u:04x};").unwrap();
                }
            }
            out
        }
        "base64" => STANDARD.encode(native),
        "utf8" => tezbar_extension_sdk::encode_component(native).replace('%', "\\x"),
        "url" => tezbar_extension_sdk::encode_component(native),
        "ascii" => first_units(native)
            .map(|u| u.to_string())
            .collect::<Vec<_>>()
            .join(if comma { "," } else { "" }),
        "hex" => {
            if native.is_empty() {
                String::new()
            } else {
                "0x".to_owned()
                    + &first_units(native)
                        .map(|u| {
                            if u > 255 {
                                format!("[{u:x}]")
                            } else {
                                format!("{u:02x}")
                            }
                        })
                        .collect::<String>()
            }
        }
        "decimal" => {
            if native.is_empty() {
                return String::new();
            }
            let mut n = BigUint::zero();
            for u in first_units(native) {
                if u >= 256 {
                    return String::new();
                }
                n = n * 256_u32 + u;
            }
            n.to_string()
        }
        _ => String::new(),
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Domain {
    pub values: BTreeMap<String, String>,
    pub source: String,
    pub raw: String,
    pub from_base: u32,
    pub to_base: u32,
    pub focused: String,
    pub best: Option<String>,
    pub error: Option<String>,
    pub parsed: Option<String>,
    pub override_base: u32,
    pub override_id: u32,
}
impl Domain {
    pub fn new(command: &str, comma: bool) -> Self {
        let mut d = Self {
            from_base: 10,
            to_base: 2,
            focused: if command == "byte-converter" {
                "bits"
            } else {
                "base10"
            }
            .into(),
            ..Self::default()
        };
        let key = match command {
            "base-converter" => "base10",
            "byte-converter" => "bits",
            _ => "native",
        };
        let _ = d.edit(command, key, "", comma);
        d
    }
    pub fn edit(&mut self, command: &str, key: &str, raw: &str, comma: bool) -> Result<(), String> {
        if let Err(error) = bounded(raw) {
            self.source = key.into();
            self.focused = key.into();
            self.raw = raw.chars().take(MAX_INPUT).collect();
            self.values.insert(key.into(), self.raw.clone());
            self.error = Some(error);
            return Ok(());
        }
        self.source = key.into();
        self.raw = raw.into();
        self.focused = key.into();
        self.error = None;
        match command {
            "base-converter" => {
                let base = if key == "input" {
                    self.from_base
                } else {
                    key.strip_prefix("base")
                        .and_then(|s| s.parse().ok())
                        .filter(|b| (2..=36).contains(b))
                        .ok_or("Unknown base field")?
                };
                let v = trim(raw);
                let prefix = match key {
                    "base2" => "0b",
                    "base8" => "0o",
                    "base16" => "0x",
                    _ => "",
                };
                let n = parse_big(
                    if prefix.is_empty() {
                        v
                    } else {
                        v.strip_prefix(prefix).unwrap_or(v)
                    },
                    base,
                );
                self.parsed = n.map(|n| n.to_string());
                self.override_base = base;
                self.override_id = u32::from(key == "input");
                self.refresh_base();
            }
            "byte-converter" => {
                let exp = UNITS
                    .iter()
                    .find(|(u, _)| *u == key)
                    .ok_or("Unknown byte field")?
                    .1;
                let (parsed, q) = parse_big_float(trim(raw))?;
                let value = parsed.map(|n| (n << exp) / q);
                let mut best = (BigUint::zero(), None);
                let mut closest: Option<(BigUint, &str)> = None;
                for (unit, exponent) in UNITS {
                    let v = value.as_ref().map(|n| (n * 100_u32) >> exponent);
                    self.values
                        .insert(unit.into(), v.as_ref().map(insert_dot).unwrap_or_default());
                    if let Some(v) = v {
                        if v < BigUint::from(100000_u32) && v > best.0 {
                            best = (v, Some(unit));
                        } else if closest.as_ref().is_none_or(|(c, _)| &v < c) {
                            closest = Some((v, unit));
                        }
                    }
                }
                self.best = if let Some(u) = best.1 {
                    Some(format!("{} {u}", insert_dot(&best.0)))
                } else {
                    closest.map(|(v, u)| format!("{} {u}", insert_dot(&v)))
                };
                self.values.insert(key.into(), raw.into());
            }
            "code-converter" => {
                if !CODES.iter().any(|(k, _)| *k == key) {
                    return Err("Unknown code field".into());
                }
                match decode(key, raw, comma) {
                    Ok(native) => {
                        for (k, _) in CODES {
                            self.values.insert(k.into(), encode(k, &native, comma));
                        }
                        self.values.insert(key.into(), raw.into());
                    }
                    Err(e) => {
                        self.values.insert(key.into(), raw.into());
                        self.error = Some(e);
                    }
                }
            }
            _ => return Err("Unknown command".into()),
        }
        Ok(())
    }
    fn refresh_base(&mut self) {
        let n = self.parsed.as_ref().and_then(|n| parse_big(n, 10));
        for (key, base, id) in [
            ("base10", 10, 0),
            ("base2", 2, 0),
            ("base4", 4, 0),
            ("base8", 8, 0),
            ("base16", 16, 0),
            ("base32", 32, 0),
            ("input", self.from_base, 1),
            ("output", self.to_base, 1),
        ] {
            let value = if base == self.override_base && id == self.override_id {
                self.raw.clone()
            } else {
                n.as_ref().map(|n| n.to_str_radix(base)).unwrap_or_default()
            };
            self.values.insert(key.into(), value);
        }
    }
    pub fn choose_base(&mut self, key: &str, raw: &str) -> Result<(), String> {
        let b: u32 = raw.parse().map_err(|_| "Invalid radix")?;
        if !(2..=36).contains(&b) {
            return Err("Radix must be 2–36".into());
        }
        if key == "fromBase" {
            self.from_base = b;
        } else {
            self.to_base = b;
        }
        self.refresh_base();
        Ok(())
    }
    pub fn autopaste(
        &mut self,
        command: &str,
        raw: &str,
        advance: bool,
        comma: bool,
    ) -> Result<(), String> {
        if let Err(error) = bounded(raw) {
            let key = match command {
                "base-converter" => "base10",
                "byte-converter" => "bits",
                _ => "native",
            };
            self.source = key.into();
            self.focused = key.into();
            self.raw = raw.chars().take(MAX_INPUT).collect();
            self.values.insert(key.into(), self.raw.clone());
            self.error = Some(error);
            return Ok(());
        }
        match command {
            "base-converter" => {
                let s = trim(raw);
                let (base, prefix) = if s.starts_with("0x") {
                    (16, "0x")
                } else if s.starts_with("0b") {
                    (2, "0b")
                } else if s.starts_with("0o") {
                    (8, "0o")
                } else if s.chars().any(|c| c.is_ascii_alphabetic())
                    && s.chars().all(|c| c.is_ascii_alphanumeric())
                {
                    if s.chars().all(|c| {
                        c.is_ascii_digit() || ('a'..='v').contains(&c.to_ascii_lowercase())
                    }) {
                        (32, "")
                    } else {
                        (36, "")
                    }
                } else {
                    (10, "")
                };
                let base = if base == 36 && !advance { 10 } else { base };
                // Source base36 focus selects advanced FROM only after setting the value.
                let input = if prefix.is_empty() {
                    s
                } else {
                    s.strip_prefix(prefix).unwrap_or(s)
                };
                let key = format!("base{base}");
                self.edit(command, &key, input, comma)?;
                self.raw = s.into();
                self.refresh_base();
                if base == 36 {
                    self.from_base = 36;
                    self.focused = "input".into();
                    self.refresh_base();
                }
            }
            "byte-converter" => {
                let s = trim(raw);
                for (unit, _) in UNITS {
                    if let Some(v) = s.strip_suffix(unit) {
                        self.edit(command, unit, v.trim_end_matches(js_space), comma)?;
                    }
                }
            }
            _ => self.edit(command, "native", raw, comma)?,
        }
        Ok(())
    }
}
pub fn insert_dot(n: &BigUint) -> String {
    let mut s = n.to_string();
    while s.len() < 3 {
        s.insert(0, '0');
    }
    s.insert(s.len() - 2, '.');
    while s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
    }
    s
}
pub fn parse_big_float(raw: &str) -> Result<(Option<BigUint>, BigUint), String> {
    let mut split = raw.split('e');
    let m = split.next().unwrap_or("");
    let exponent = parse_int(split.next().unwrap_or(""), 0);
    let exponent = if exponent.is_nan() { 0.0 } else { exponent };
    let dot = m.find('.');
    let digits = m.replacen('.', "", 1);
    let exp = dot.map(|p| m.len() - p - 1).unwrap_or(0) as f64 - exponent;
    if !exp.is_finite() || exp.abs() > 4096.0 {
        return Err(
            "Scientific exponent exceeds the bounded ±4096 digit range; draft retained.".into(),
        );
    }
    let exp = exp as i32;
    let mut n = parse_big(&digits, 10);
    if exp < 0 {
        n = n.map(|n| n * BigUint::from(10_u32).pow((-exp) as u32));
    }
    Ok((n, BigUint::from(10_u32).pow(exp.max(0) as u32)))
}
