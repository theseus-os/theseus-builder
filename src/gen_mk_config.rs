use crate::log;
use crate::opt_default;
use crate::oops;
use crate::Config;
use crate::DEFAULT_CONFIG;

use std::fs::write;

use toml::Value;

pub fn process(config: &Config) {
    let stage = "gen-mk-config";

    let output = config.str("gen-mk-config.output");

    log!(stage, "generating {}", output);

    let mut path = Vec::new();
    let mut keys = Vec::new();
    let value = DEFAULT_CONFIG.parse::<Value>().unwrap();
    explore(&mut path, &mut keys, &value);

    let mut generated = String::new();
    for key in keys {
        let content = match opt_default(&key) {
            Value::String(_) => config.str(&key),
            Value::Boolean(_) => config.bool(&key).to_string(),
            Value::Array(_) => config.vec(&key).join(" "),
            _ => oops!(stage, "invalid property type for key {}", &key),
        };
        let var_name = key.to_uppercase().replace("-", "_").replace(".", "_");
        generated.push_str(&format!("{}=\"{}\"\n", var_name, content));
    }

    write(output, &generated).unwrap();
}

fn explore(path: &mut Vec<String>, keys: &mut Vec<String>, value: &Value) {
    if let Value::Table(table) = value {
        for (key, value) in table.iter() {
            path.push(key.clone());
            explore(path, keys, value);
            path.pop();
        }
    } else {
        keys.push(path.join("."));
    }
}