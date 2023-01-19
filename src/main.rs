extern crate serde_json;

use actix_files::NamedFile;
use actix_web::{error, get, web, App, Error as ActixError, HttpRequest, HttpResponse, HttpServer};
use rusqlite::OpenFlags;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
type Connection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;
type WordResult = Result<Vec<Word>, rusqlite::Error>;

#[derive(Debug, Serialize)]
struct Word {
    part_of_speech: String,
    definition: String,
}

#[derive(Debug, Serialize)]
struct WordResponse {
    word: String,
    definitions: Vec<Word>,
}

#[derive(Debug, Deserialize)]
struct SearchRequest {
    search: String,
}

async fn index(_req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path: PathBuf = "./index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

#[get("/api/words/{word}")]
async fn definition_json(path: web::Path<String>, pool: web::Data<Pool>) -> Result<HttpResponse, ActixError> {
    let word: String = path.into_inner().to_lowercase();
    let word_clone: String = word.clone();
    let result = execute(word, &pool).await?;
    let response: WordResponse = WordResponse{word: word_clone, definitions: result};
    Ok(HttpResponse::Ok().json(response))
}

#[get("/ui/words")]
async fn definition_text(info: web::Query<SearchRequest>, pool: web::Data<Pool>) -> Result<HttpResponse, ActixError> {
    let search_request: SearchRequest = info.into_inner();
    let word: String = search_request.search.to_lowercase();
    let word_clone: String = word.clone();
    let result = execute(word, &pool).await?;
    let response: WordResponse = WordResponse{word: word_clone, definitions: result};
    Ok(HttpResponse::Ok().body(::serde_json::to_string_pretty(&response).unwrap()))
}

async fn execute(word: String, pool: &Pool) -> Result<Vec<Word>, ActixError> {
    let pool = pool.clone();
    let connection = web::block(move || pool.get())
        .await?
        .map_err(error::ErrorInternalServerError)?;

    web::block(move || {
        get_word_definitions(word, connection)
    })
    .await?
    .map_err(error::ErrorInternalServerError)
}

fn get_word_definitions(word: String, connection: Connection) -> WordResult {
    let query = format!("SELECT * FROM dictionary WHERE word = '{}' ORDER BY part_of_speech", &word);
    connection.prepare(&query)?
        .query_map([], |row| {
            Ok(Word {
                part_of_speech: row.get(1)?,
                definition: row.get(2)?,
            })
        })
        .and_then(Iterator::collect)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let manager = r2d2_sqlite::SqliteConnectionManager::file("dictionary.db")
        .with_flags(OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX);
    let pool = r2d2::Pool::new(manager).unwrap();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/", web::get().to(index))
            .service(definition_json)
            .service(definition_text)
    })
    .bind(("0.0.0.0", 8080))?
    .workers(4)
    .run()
    .await
}

