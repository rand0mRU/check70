use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use colored::*;
use crate::parser::{get_examples, get_contest_problem_from_link};
use crate::config::{new_test, AppState};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PageData {
    pub html: String,
    pub url: String,
    pub title: String,
}

async fn receive_html(
    data: web::Json<PageData>,
    state: web::Data<Mutex<AppState>>,
) -> impl Responder {
    let page_data = data.into_inner();
    
    // Парсим HTML
    let test_cases = get_examples(&page_data.html);
    
    // Получаем имя задачи
    let (contest, problem) = get_contest_problem_from_link(&page_data.url);
    let name = format!("{}-{}", contest, problem);
    
    // Сохраняем в конфиг
    let state = state.lock().unwrap();
    match new_test(&name, test_cases, &state.config_file) {
        Ok(_) => {
            println!("{} '{}'", 
                "Test written as".white(),
                name.green().bold()
            );
            println!("  {} {}", "URL:".dimmed(), page_data.url.dimmed());
            println!("  {} {}", "Title:".dimmed(), page_data.title.dimmed());
            println!();
            
            HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "name": name
            }))
        }
        Err(e) => {
            eprintln!("{} {}", "[ERROR]".red(), e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": e.to_string()
            }))
        }
    }
}

// Добавляем async и возвращаем Result<(), Box<dyn std::error::Error>>
pub async fn start_server() -> std::io::Result<()> {
    let config_file = ".check70.yaml";
    let app_state = web::Data::new(Mutex::new(AppState {
        config_file: config_file.to_string(),
    }));
    
    println!("{}", "[check70] Waiting for extension data...".green());
    println!();
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(web::resource("/receive-html").route(web::post().to(receive_html)))
            .service(web::resource("/health").route(web::get().to(|| async { HttpResponse::Ok().body("OK") })))
    })
    .bind("127.0.0.1:60177")?
    .run()
    .await
}