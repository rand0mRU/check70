use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
// use std::io::{Read, Write};
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

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    let cmd = &args[1];

    match cmd.as_str() {
        "--init" | "-i" => init_config(),
        "--init-clear" | "-ic" => init_config_clear(),
        // "-c" | "--copy" => {
        //     println!("{}", "[check70] Парсинг с сайта пока не реализован в Rust версии напрямую (требуется порт библиотеки)".yellow());
        // }
        _ => run_tests(&args),
    }
}

fn print_help() {
    println!("check70 utility.\nusage: check70 <test name> [args]");
    println!("Use '-i' for init");
}

fn init_config() {
    let mut config = Config::new();
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

fn init_config_clear() {
    let mut config = Config::new();
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
        let start_time = Instant::now();
        let cmd_parts: Vec<&str> = run.start.split_whitespace().collect();
        
        let mut child = Command::new(cmd_parts[0])
            .args(&cmd_parts[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Cannot start process");

        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(run.input.as_bytes()).unwrap();
        drop(stdin);

        let output = child.wait_with_output().expect("Error running");
        let duration = start_time.elapsed().as_millis();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();

        if stdout.trim() == run.output.trim() {
            println!("{}", format!("[test {}] result: ok. time: {} ms", i + 1, duration).green());
            correct += 1;
        } else {
            println!("{}", format!("[test {}] result: incorrect, process ended with error\n\tactual: {:?}\n\texpected: {:?}", i + 1, stdout, run.output).red());
            println!("error stack:\n{}", String::from_utf8_lossy(&output.stderr).to_string());
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
