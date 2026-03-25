use scraper::{Html, Selector};
use crate::config::TestCase;

pub fn get_examples(html: &str) -> Vec<TestCase> {
    let document = Html::parse_document(html);
    let mut samples = Vec::new();
    
    let sample_selector = Selector::parse("div.sample-test").unwrap();
    let input_selector = Selector::parse("div.input pre").unwrap();
    let output_selector = Selector::parse("div.output pre").unwrap();
    
    if let Some(sample_div) = document.select(&sample_selector).next() {
        let inputs: Vec<String> = sample_div
            .select(&input_selector)
            .map(|pre| pre.text().collect::<Vec<_>>().join("\n").trim().to_string())
            .collect();
            
        let outputs: Vec<String> = sample_div
            .select(&output_selector)
            .map(|pre| pre.text().collect::<Vec<_>>().join("\n").trim().to_string())
            .collect();
        
        for (input_text, output_text) in inputs.iter().zip(outputs.iter()) {
            let mut input = input_text.clone();
            let mut output = output_text.clone();
            
            if !input.ends_with('\n') {
                input.push('\n');
            }
            if !output.ends_with('\n') {
                output.push('\n');
            }
            
            input = input.replace(" \n", "\n");
            output = output.replace(" \n", "\n");
            
            samples.push(TestCase {
                start: String::new(),
                input,
                output,
                timeout: 1000,
            });
        }
    }
    
    samples
}

pub fn get_contest_problem_from_link(url: &str) -> (String, String) {
    let url = url.replace("https://", "").replace("http://", "");
    let parts: Vec<&str> = url.split('/').collect();
    
    if parts.contains(&"gym") || parts.contains(&"contest") {
        let contest = parts.get(parts.len() - 3).unwrap_or(&"").to_string();
        let problem = parts.get(parts.len() - 1).unwrap_or(&"").to_string();
        return (contest, problem);
    }
    
    let contest = parts.get(parts.len() - 2).unwrap_or(&"").to_string();
    let problem = parts.get(parts.len() - 1).unwrap_or(&"").to_string();
    (contest, problem)
}