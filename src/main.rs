mod server;
mod config;
mod parser;

use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Write};
use std::process::{Command, Stdio};
use std::time::Instant;
use std::{env, process};

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

type Config = HashMap<String, ConfigEntry>;

const CONFIG_FILE: &str = ".check70.yaml";

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return Ok(());
    }

    let cmd = &args[1];

    match cmd.as_str() {
        "--init" | "-i" => {
            config::init_config();
            Ok(())
        }
        "--init-clear" | "-ic" => {
            config::init_config_clear();
            Ok(())
        }
        "-w" => {
            server::start_server().await?;
            Ok(())
        }
        _ => {
            run_tests(&args);
            Ok(())
        }
    }
}

fn print_help() {
    println!("check70 utility.\nusage: check70 <test name> [args]");
    println!("Use '-i' for init");
    println!("Use '-w' for start web server");
}

fn run_tests(args: &[String]) {
    let test_name = &args[1];
    let config_path = CONFIG_FILE;

    let file = File::open(config_path).expect("Config not found");
    let config: Config = serde_yaml::from_reader(file).expect("Error config parsing");

    let entry = match config.get(test_name) {
        Some(e) => e,
        None => {
            println!("{}", format!("[check70] Test '{}' not found", test_name).red());
            process::exit(1);
        }
    };

    // Компиляция
    if !entry.compile.is_empty() {
        let status = if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", &entry.compile]).status()
        } else {
            Command::new("sh").args(["-c", &entry.compile]).status()
        };

        if status.is_err() || !status.unwrap().success() {
            println!("{}", "[check70] Compilation error".red());
            process::exit(1);
        }
        println!("{}", "[compile] File compiled\n".green());
    }

    let exit_on_error = !args.contains(&"-se".to_string());
    let mut correct = 0;
    let mut incorrect = 0;

    for (i, run) in entry.runs.iter().enumerate() {
        let cmd_parts: Vec<&str> = run.start.split_whitespace().collect();
        
        let mut child = Command::new(cmd_parts[0])
            .args(&cmd_parts[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Cannot start process");
        
        let start_time = Instant::now();
        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(run.input.as_bytes()).unwrap();
        drop(stdin);

        let output = child.wait_with_output().expect("Error running");
        let duration = start_time.elapsed().as_millis();
        
        let stdout = String::from_utf8_lossy(&output.stdout)
            .to_string()
            .replace("\r\n", "\n");
        let expected = run.output.replace("\r\n", "\n");

        if stdout.trim() == expected.trim() {
            println!("{}", format!("[test {}] result: ok. time: {} ms", i + 1, duration).green());
            correct += 1;
        } else {
            println!("{}", format!("[test {}] result: incorrect, process ended with error\n\tactual: {:?}\n\texpected: {:?}", i + 1, stdout, expected).red());
            
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if !stderr.is_empty() {
                println!("error stack:\n{}", stderr);
            }
            incorrect += 1;
            if exit_on_error { break; }
        }
    }

    if incorrect == 0 {
        println!("\n{}", "[check70] test result: ok".green());
    } else {
        println!("\n[check70] test result:\n\t{} {}\n\t{} {}", "correct:".green(), correct, "incorrect:".red(), incorrect);
        process::exit(1);
    }
}