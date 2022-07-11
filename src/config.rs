use serde::Deserialize;
use serde_yaml;
use std::env::var;
use std::path::Path;

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct Item {
    pub name: String,
    #[serde(default)]
    pub params: Vec<String>,
    #[serde(default)]
    pub signal: i32,
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct Config {
    pub list: Vec<Item>,
    pub sep: String,
    #[serde(default)]
    pub autosep: bool,
    pub delay: u64,
}

static DEFAULT_CONFIG: &str = r#"
list:
  - name: exec
    params:
      - echo
      - "Welcome sheep"
  - name: echo
    params:
      - much bar
  - name: battery
  - name: memory
  - name: time
sep: " | "
autosep: true
delay: 1000
"#;

pub fn read_config(p: Option<String>) -> Config {
    let config_home = match var("XDG_CONFIG_HOME")
        .or_else(|_| var("HOME").map(|home| format!("{}/.config", home)))
    {
        Ok(s) => s,
        Err(_) => "~/.config".to_string(),
    };
    let mut path = format!("{}/sbar/config.yaml", config_home);
    match p {
        Some(p) => {
            path = p;
        }
        None => {}
    }
    if Path::new(path.as_str()).exists() {
        match std::fs::File::open(path) {
            Ok(f) => match serde_yaml::from_reader(f) {
                Ok(c) => {
                    return c;
                },
                Err(_) => {
                    eprintln!("Failed parsing config file, using default config");
                }
            },
            Err(_) => {
                eprintln!("Failed loading config file, using default config")
            }
        };
    }
    match serde_yaml::from_str(DEFAULT_CONFIG) {
        Ok(c) => c,
        Err(_) => panic!("Failed parsing default configuration"),
    }
}
