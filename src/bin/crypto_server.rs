use std::io::{self, BufRead, Write, stderr};
use std::sync::OnceLock;

use serde_json::{json, Value};

const MAX_BODY_BYTES: usize = 2_097_152;
const MAX_KEY_BYTES: usize = 131_072;
const MAX_SIG_BYTES: usize = 131_072;

static ALGO_TABLE: OnceLock<Vec<(&'static str, u8, &'static str)>> = OnceLock::new();

fn algo_table() -> &'static [(&'static str, u8, &'static str)] {
    ALGO_TABLE.get_or_init(|| {
        let mayo1 = kylos_qpadl::algo_for_level(1)
            .expect("MAYO-1 must be enabled (liboqs built with -DOQS_ENABLE_SIG_MAYO=Yes)");
        let mayo3 = kylos_qpadl::algo_for_level(3)
            .expect("MAYO-3 must be enabled");
        let mayo5 = kylos_qpadl::algo_for_level(5)
            .expect("MAYO-5 must be enabled");
        vec![
            ("mayo1",      1, mayo1),
            ("mayo3",      3, mayo3),
            ("mayo5",      5, mayo5),
            ("falcon512",  1, kylos_qpadl::FALCON_512),
            ("ml_dsa_65",  3, kylos_qpadl::ML_DSA_65),
            ("sphincs_256f", 5, kylos_qpadl::SPHINCS_256F),
        ]
    })
}

fn resolve_algo(name: &str) -> Option<(&'static str, u8)> {
    for &(key, level, oqs_name) in algo_table() {
        if key == name { return Some((oqs_name, level)); }
    }
    None
}

fn respond(id: u64, result: Option<Value>, error: Option<&str>) {
    let mut msg = json!({ "id": id });
    if let Some(r) = result {
        msg["result"] = r;
    }
    if let Some(e) = error {
        msg["error"] = Value::String(e.to_string());
    }
    let _ = writeln!(io::stdout(), "{}", serde_json::to_string(&msg).unwrap_or_default());
    let _ = io::stdout().flush();
}

fn reject_too_large(label: &str, size: usize) -> Option<Vec<u8>> {
    let _ = writeln!(stderr(), "crypto_server: {label} too large ({size} bytes)");
    None
}

fn algo_to_value(key: &str, level: u8, oqs_name: &str) -> Value {
    json!({
        "id": key,
        "level": level,
        "enabled": kylos_qpadl::algo_is_enabled(oqs_name),
        "oqs_name": oqs_name,
    })
}

fn handle_request(id: u64, method: &str, params: &Value) {
    match method {
        "status" => {
            let layers: Vec<Value> = algo_table().iter().map(|&(k, l, o)| algo_to_value(k, l, o)).collect();
            respond(id, Some(json!({ "algorithms": layers })), None);
        }

        "keypair" => {
            let algo_name = params.get("algorithm").and_then(Value::as_str).unwrap_or("mayo1");
            let (oqs_name, level) = match resolve_algo(algo_name) {
                Some(v) => v,
                None => { respond(id, None, Some("unknown algorithm")); return; }
            };
            let sig = match kylos_qpadl::create_sig_by_name(oqs_name) {
                Ok(s) => s,
                Err(_) => { respond(id, None, Some("sig init failed")); return; }
            };
            match sig.keypair() {
                Ok((pk, sk)) => {
                    respond(id, Some(json!({
                        "algorithm": oqs_name,
                        "level": level,
                        "public_key": base64_encode(&pk),
                        "secret_key": base64_encode(&sk),
                    })), None);
                }
                Err(_) => respond(id, None, Some("keypair generation failed")),
            }
        }

        "sign" => {
            let algo_name = params.get("algorithm").and_then(Value::as_str).unwrap_or("mayo1");
            let msg_b64 = match params.get("message").and_then(Value::as_str) {
                Some(s) => s,
                None => { respond(id, None, Some("missing message")); return; }
            };
            let sk_b64 = match params.get("secret_key").and_then(Value::as_str) {
                Some(s) => s,
                None => { respond(id, None, Some("missing secret_key")); return; }
            };
            let sk = match base64_decode(sk_b64) {
                Some(b) if b.len() <= MAX_KEY_BYTES => Some(b),
                Some(_) => reject_too_large("secret_key", sk_b64.len()),
                None => { respond(id, None, Some("invalid secret_key base64")); return; }
            };
            let sk = match sk { Some(v) => v, None => { respond(id, None, Some("secret_key too large")); return; } };
            let msg = match base64_decode(msg_b64) {
                Some(b) if b.len() <= MAX_BODY_BYTES => Some(b),
                Some(_) => reject_too_large("message", msg_b64.len()),
                None => { respond(id, None, Some("invalid message base64")); return; }
            };
            let msg = match msg { Some(v) => v, None => { respond(id, None, Some("message too large")); return; } };
            let (oqs_name, level) = match resolve_algo(algo_name) {
                Some(v) => v,
                None => { respond(id, None, Some("unknown algorithm")); return; }
            };
            let sig = match kylos_qpadl::create_sig_by_name(oqs_name) {
                Ok(s) => s,
                Err(_) => { respond(id, None, Some("sig init failed")); return; }
            };
            match sig.sign(&msg, &sk) {
                Ok(signature) => {
                    respond(id, Some(json!({
                        "algorithm": oqs_name,
                        "level": level,
                        "signature": base64_encode(&signature),
                    })), None);
                }
                Err(_) => respond(id, None, Some("signing failed")),
            }
        }

        "verify" => {
            let algo_name = params.get("algorithm").and_then(Value::as_str).unwrap_or("mayo1");
            let msg_b64 = match params.get("message").and_then(Value::as_str) {
                Some(s) => s,
                None => { respond(id, None, Some("missing message")); return; }
            };
            let sig_b64 = match params.get("signature").and_then(Value::as_str) {
                Some(s) => s,
                None => { respond(id, None, Some("missing signature")); return; }
            };
            let pk_b64 = match params.get("public_key").and_then(Value::as_str) {
                Some(s) => s,
                None => { respond(id, None, Some("missing public_key")); return; }
            };
            let msg = match base64_decode(msg_b64) {
                Some(b) if b.len() <= MAX_BODY_BYTES => Some(b),
                Some(_) => reject_too_large("message", msg_b64.len()),
                None => { respond(id, None, Some("invalid message base64")); return; }
            };
            let msg = match msg { Some(v) => v, None => { respond(id, None, Some("message too large")); return; } };
            let sig_bytes = match base64_decode(sig_b64) {
                Some(b) if b.len() <= MAX_SIG_BYTES => Some(b),
                Some(_) => reject_too_large("signature", sig_b64.len()),
                None => { respond(id, None, Some("invalid signature base64")); return; }
            };
            let sig_bytes = match sig_bytes { Some(v) => v, None => { respond(id, None, Some("signature too large")); return; } };
            let pk = match base64_decode(pk_b64) {
                Some(b) if b.len() <= MAX_KEY_BYTES => Some(b),
                Some(_) => reject_too_large("public_key", pk_b64.len()),
                None => { respond(id, None, Some("invalid public_key base64")); return; }
            };
            let pk = match pk { Some(v) => v, None => { respond(id, None, Some("public_key too large")); return; } };
            let (oqs_name, _level) = match resolve_algo(algo_name) {
                Some(v) => v,
                None => { respond(id, None, Some("unknown algorithm")); return; }
            };
            let sig_obj = match kylos_qpadl::create_sig_by_name(oqs_name) {
                Ok(s) => s,
                Err(_) => { respond(id, None, Some("sig init failed")); return; }
            };
            let valid = sig_obj.verify(&msg, &sig_bytes, &pk).is_ok();
            respond(id, Some(json!({ "valid": valid })), None);
        }

        _ => respond(id, None, Some("unknown method")),
    }
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        out.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 { out.push(CHARS[((triple >> 6) & 0x3F) as usize] as char); } else { out.push('='); }
        if chunk.len() > 2 { out.push(CHARS[(triple & 0x3F) as usize] as char); } else { out.push('='); }
    }
    out
}

fn base64_decode(s: &str) -> Option<Vec<u8>> {
    const DECODE: [i8; 128] = {
        let mut d = [-1i8; 128];
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut i = 0;
        while i < 64 {
            d[chars[i] as usize] = i as i8;
            i += 1;
        }
        d
    };
    let clean: Vec<u8> = s.bytes().filter(|&b| b != b'\n' && b != b'\r').collect();
    if clean.is_empty() { return Some(Vec::new()); }
    if clean.len() % 4 == 1 { return None; }
    let mut out = Vec::with_capacity(clean.len() / 4 * 3);
    for chunk in clean.chunks(4) {
        let mut accum = 0u32;
        let mut valid = 0u32;
        for &b in chunk {
            if b == b'=' { continue; }
            if b >= 128 || DECODE[b as usize] < 0 { return None; }
            if valid >= 4 { return None; }
            accum = (accum << 6) | (DECODE[b as usize] as u32);
            valid += 1;
        }
        accum <<= 6 * (4 - valid);
        out.push((accum >> 16) as u8);
        if valid >= 3 { out.push((accum >> 8) as u8); }
        if valid >= 4 { out.push(accum as u8); }
    }
    Some(out)
}

fn main() {
    kylos_qpadl::init();
    let stdin = io::stdin().lock();
    for line in stdin.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                let _ = writeln!(stderr(), "crypto_server: stdin error: {e}");
                break;
            }
        };
        let line = line.trim().to_string();
        if line.is_empty() { continue; }
        let parsed: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                let _ = writeln!(stderr(), "crypto_server: parse error: {e}");
                continue;
            }
        };
        let id = parsed.get("id").and_then(Value::as_u64).unwrap_or(0);
        let method = parsed.get("method").and_then(Value::as_str).unwrap_or_default();
        let params = parsed.get("params").unwrap_or(&Value::Null);
        handle_request(id, method, params);
    }
}
