use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestCase {
    pub start: String,
    pub input: String,
    pub output: String,
    pub timeout: u32,
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

pub fn create_default_config(config_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config {
        check: Some(CheckConfig {
            compile: String::new(),
            runs: vec![TestCase {
                start: String::new(),
                input: String::new(),
                output: String::new(),
                timeout: 1000,
            }],
        }),
        problems: HashMap::new(),
    };
    
    let yaml_string = serde_yaml::to_string(&config)?;
    let mut file = File::create(config_file)?;
    file.write_all(yaml_string.as_bytes())?;
    
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
            create_default_config(config_file)?;
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