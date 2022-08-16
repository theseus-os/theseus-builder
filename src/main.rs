use std::fs::read_to_string;
use std::fs::create_dir_all;
use std::fs::create_dir;
use std::fs::read_dir;
use std::fmt::Display;
use std::process::exit;
use std::process::ExitStatus;
use std::process::Command;
use std::path::Path;
use std::io::Error;
use std::io::ErrorKind;
use std::ffi::OsString;
use std::mem::swap;

use toml::Value;
use toml::map::Map;

use pico_args::Arguments;

mod discover;
mod directories;
mod build_cells;
mod link_nano_core;
mod serialize_nano_core_syms;
mod relink_rlibs;
mod copy_crate_objects;
mod relink_objects;
mod strip_objects;
mod add_bootloader;

pub fn die() -> ! {
    exit(1)
}

fn main() {
    let mut args = Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        // println!("{}", include_str!("help.txt"));
        println!("sorry, no help atm");
    } else {
        let config_path = match args.value_from_str(["-c", "--config-file"]) {
            Ok(path) => path,
            _ => "config.toml".to_string(),
        };

        log!("reading config", "config file: {}", config_path);

        let cfg_string = match read_to_string(&config_path) {
            Ok(cfg_string) => cfg_string,
            Err(e) => oops!("main", "{}", e),
        };

        let mut config = match cfg_string.parse::<Value>() {
            Ok(value) => value,
            Err(e) => oops!("main", "{}", e),
        };

        apply_overrides(&mut config, args.finish());

        let steps = [
            discover::process,
            directories::process,
            build_cells::process,
            link_nano_core::process,
            serialize_nano_core_syms::process,
            relink_rlibs::process,
            copy_crate_objects::process,
            relink_objects::process,
            strip_objects::process,
            add_bootloader::process,
        ];

        log!("main", "configuration was parsed successfully");

        for step in steps {
            step(&config);
        }
    }
}

fn get_config(mut config: &mut Value, mut path: Vec<String>) -> (&mut Value, String) {
    let last = path.pop().expect("invalid override key");
    for key in path {
        if config.get(&key).is_none() {
            let config = config.as_table_mut().unwrap();
            config.insert(key.clone(), Value::from(Map::new()));
        }
        config = config.get_mut(&key).unwrap();
    }
    (config, last)
}

fn parse_toml_value(value: String) -> Value {
    if let Ok(integer) = value.parse::<i64>() {
        Value::Integer(integer)
    } else if let Ok(float) = value.parse::<f64>() {
        Value::Float(float)
    } else if let Ok(boolean) = value.parse::<bool>() {
        Value::Boolean(boolean)
    } else {
        Value::String(value)
    }
}

fn apply_overrides(config: &mut Value, override_args: Vec<OsString>) {
    let mut array = None;
    for arg in override_args {
        let arg = arg.into_string().expect("arguments must be valid UTF-8");

        if let Some((mut vec, path)) = array.take() {
            if arg == "]" {
                let (config, key) = get_config(config, path);
                let config = config.as_table_mut().unwrap();
                config.insert(key, Value::Array(vec));
                array = None;
            } else {
                vec.push(parse_toml_value(arg));
                array = Some((vec, path));
            }

        } else {

            let mut path = Vec::new();
            let mut key = String::new();
            let mut value = String::new();
            let mut collect = false;
            for c in arg.chars() {
                if collect {
                    value.push(c);
                } else if c == '.' {
                    let mut tmp = String::new();
                    swap(&mut tmp, &mut key);
                    path.push(tmp);
                } else if c == '=' {
                    collect = true;
                } else {
                    key.push(c);
                }
            }
            path.push(key);
            if value == "[" {
                array = Some((Vec::new(), path));
            } else {
                let (config, key) = get_config(config, path);
                let value = parse_toml_value(value);
                let config = config.as_table_mut().unwrap();
                config.insert(key, value);
            }

        }
    }
}

#[macro_export]
macro_rules! log {
    ($log_stage:expr, $($arg:tt)*) => {{
        print!("[{}] ", $log_stage);
        println!($($arg)*);
    }}
}

#[macro_export]
macro_rules! oops {
    ($log_stage:expr, $($arg:tt)*) => {{
        print!("[{}] error: ", $log_stage);
        println!($($arg)*);
        crate::die();
    }}
}

fn check_result(stage: &str, result: Result<ExitStatus, Error>, binary: &str) {
    let no_problem = match result {
        Ok(result) => result.success(),
        _ => false,
    };

    if !no_problem {
        oops!(stage, "{} invocation failed", binary);
    }
}

fn run_env(stage: &str, binary: &str, env: &[(&str, &str)], args: &[&[&str]]) {
    let mut command = Command::new(binary);
    for (key, value) in env {
        command.env(key, value);
    }
    for args in args {
        command.args(*args);
    }
    check_result(stage, command.status(), binary);
}

fn run(stage: &str, binary: &str, args: &[&[&str]]) {
    run_env(stage, binary, &[], args)
}

fn try_create_dir<P: AsRef<Path> + Display>(path: P, all: bool) {
    let op = match all {
        true => create_dir_all,
        _    => create_dir,
    };
    if let Err(e) = op(&path) {
        if e.kind() != ErrorKind::AlreadyExists {
            println!("could not create directory: {}", path);
            crate::die();
        }
    }
}

fn list_dir<P: AsRef<Path> + Display>(stage: &str, path: P) -> Vec<(String, bool)> {
    let inner = |path| -> Option<Vec<(String, bool)>> {
        let mut out = Vec::new();
        let iter = read_dir(&path).ok()?;
        for entry in iter {
            let entry = entry.ok()?;
            let is_dir = entry.file_type().ok()?.is_dir();
            let name = entry.file_name().into_string().ok()?;
            out.push((name, is_dir))
        }
        Some(out)
    };
    let err = || oops!(stage, "failed to list directory `{}`", path);
    inner(&path).unwrap_or_else(err)
}

fn opt_default(path: &[&str]) -> Value {
    let mut config = &include_str!("defaults.toml").parse::<Value>().unwrap();
    for key in path {
        if let Some(value) = config.get(key) {
            config = value;
        } else {
            println!("missing option in config: {}", path.join("/"));
            crate::die();
        }
    }
    config.clone()
}

pub fn opt(mut config: &Value, path: &[&str]) -> Value {
    for key in path {
        if let Some(value) = config.get(key) {
            config = value;
        } else {
            return opt_default(path);
        }
    }
    config.clone()
}

pub fn opt_bool(config: &Value, path: &[&str]) -> bool {
    if let Value::Boolean(boolean) = opt(config, path) {
        boolean
    } else {
        println!("wrong type: {} must be a boolean!", path.last().unwrap());
        crate::die();
    }
}

pub fn opt_str(config: &Value, path: &[&str]) -> String {
    if let Value::String(string) = opt(config, path) {
        string
    } else {
        println!("wrong type: {} must be a string!", path.last().unwrap());
        crate::die();
    }
}

pub fn opt_str_vec(config: &Value, path: &[&str]) -> Vec<String> {
    let crash = || -> ! {
        println!("wrong type: {} must be an array!", path.last().unwrap());
        crate::die()
    };
    if let Value::Array(array) = opt(config, path) {
        let mut out = Vec::with_capacity(array.len());
        for item in array {
            if let Value::String(string) = item {
                out.push(string);
            } else {
                crash();
            }
        }
        out
    } else {
        crash();
    }
}
