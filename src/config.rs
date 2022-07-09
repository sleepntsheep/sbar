use serde::Deserialize;
use serde_yaml;
use std::env::var;
use std::path::Path;
use log;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Item {
    pub name: String,
    #[serde(default)]
    pub params: Vec<String>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    pub list: Vec<Item>,
    pub sep: String,
    pub delay: u64,
}

static DEFAULT_CONFIG: &str = r#"
list:
  - name: exec
    params:
      - echo
      - "Welcome sheep"
  - name: memory
    params:
      - LMAO
  - name: time
sep: " | "
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
    println!("{}", path);
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
                    log::warn!("Failed parsing config file, using default config");
                }
            },
            Err(_) => {
                log::warn!("Failed loading config file, using default config")
            }
        };
    }
    match serde_yaml::from_str(DEFAULT_CONFIG) {
        Ok(c) => c,
        Err(_) => panic!("Failed parsing default configuration"),
    }
}
