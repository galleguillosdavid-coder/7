use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

pub(crate) fn load_config<P: AsRef<Path>>(path: P) -> HashMap<String, String> {
    let mut cfg = HashMap::new();
    if let Ok(text) = fs::read_to_string(path) {
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                let k = k.trim().to_string();
                let v = v.trim().trim_matches('"').trim_matches('\'').to_string();
                if !k.is_empty() {
                    cfg.insert(k, v);
                }
            }
        }
    }
    cfg
}

pub(crate) fn var(name: &str, config: &HashMap<String, String>) -> Option<String> {
    env::var(name).ok().or_else(|| config.get(name).cloned())
}

pub(crate) fn set_defaults_from_file(path: &str) -> HashMap<String, String> {
    let cfg = load_config(path);
    for (k, v) in &cfg {
        if env::var(k).is_err() {
            unsafe { env::set_var(k, v); }
        }
    }
    cfg
}
