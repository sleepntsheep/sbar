static DEFAULT_CONFIG: &str = r#"
list:
  - name: exec
    prefix: "  "
      # item's prefix and suffix will overwrite global 
    params:
      - pamixer
      - "--get-volume-human"
    signal: 46
  - name: battery
    interval: 30
  - name: memory
    interval: 10
  - name: time
    params:
      - " %Y-%m-%d %I:%M:%S"
    interval: 1
    fg: '#000000' # qoute is needed because # is used for yaml comments
    bg: '#ffffff'

# placed between every component
sep: "|"
# global prefix, suffix
prefix: " "
suffix: " "
# use status2d patch for color (bg, fg property in component)
status2d_color: true
"#;

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
    #[serde(default)]
    pub interval: u64,
    #[serde(default)]
    pub bg: Option<String>,
    #[serde(default)]
    pub fg: Option<String>,
    #[serde(default)]
    pub str: Option<String>,
    #[serde(default)]
    pub prefix: String,
    #[serde(default)]
    pub suffix: String,
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct Bar {
    pub list: Vec<Item>,
    pub sep: String,
    #[serde(default)]
    pub status2d_color: bool,
    #[serde(default)]
    pub counter: u64,
    #[serde(default)]
    pub prefix: String,
    #[serde(default)]
    pub suffix: String,
}

pub fn read_config(p: Option<String>) -> Bar {
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
                }
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
