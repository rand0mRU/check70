use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::collections::HashMap;
use colored::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestCase {
    pub start: String,
    pub input: String,
    pub output: String,
    pub timeout: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Run {
    start: String,
    input: String,
    output: String,
    timeout: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfigEntry {
    compile: String,
    runs: Vec<Run>,
}

type ConfigHash = HashMap<String, ConfigEntry>;
const CONFIG_FILE: &str = ".check70.yaml";

pub fn init_config() {
    let mut config = ConfigHash::new();
    let example_run = Run {
        start: "python test.py".to_string(),
        input: "10\n2\n".to_string(),
        output: "5.0\n".to_string(),
        timeout: 1000,
    };
    
    config.insert("py-example".to_string(), ConfigEntry {
        compile: "".to_string(),
        runs: vec![example_run],
    });

    let f = File::create(CONFIG_FILE).expect("Error creating file");
    serde_yaml::to_writer(f, &config).expect("Error YAML writing");
    println!("{}", "[check70] Configuration ready".green());
}

pub fn init_config_clear() {
    let mut config = ConfigHash::new();
    let example_run = Run {
        start: "".to_string(),
        input: "".to_string(),
        output: "".to_string(),
        timeout: 1000,
    };
    
    config.insert("check".to_string(), ConfigEntry {
        compile: "".to_string(),
        runs: vec![example_run],
    });

    let f = File::create(CONFIG_FILE).expect("Error creating file");
    serde_yaml::to_writer(f, &config).expect("Error YAML writing");
    println!("{}", "[check70] Configuration ready".green());
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProblemConfig {
    pub compile: String,
    pub runs: Vec<TestCase>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub check: Option<CheckConfig>,
    #[serde(flatten)]
    pub problems: HashMap<String, ProblemConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckConfig {
    pub compile: String,
    pub runs: Vec<TestCase>,
}

pub struct AppState {
    pub config_file: String,
}

pub fn create_default_config() -> Result<(), Box<dyn std::error::Error>> {
    init_config_clear();
    
    Ok(())
}

pub fn new_test(name: &str, test_cases: Vec<TestCase>, config_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config_content = fs::read_to_string(config_file);
    
    let mut config: Config = match config_content {
        Ok(content) => serde_yaml::from_str(&content).unwrap_or_else(|_| Config {
            check: None,
            problems: HashMap::new(),
        }),
        Err(_) => {
            create_default_config()?;
            Config {
                check: None,
                problems: HashMap::new(),
            }
        }
    };
    
    config.problems.insert(name.to_string(), ProblemConfig {
        compile: String::new(),
        runs: test_cases,
    });
    
    let yaml_string = serde_yaml::to_string(&config)?;
    let mut file = File::create(config_file)?;
    file.write_all(yaml_string.as_bytes())?;
    
    Ok(())
}