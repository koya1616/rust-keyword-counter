use axum::{
  extract::{Path, Query},
  http::StatusCode,
  response::Json,
  routing::{get, post},
  Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio;

use keyword_analyzer_shared::{AnalysisResult, KeywordAnalyzer, Language};

#[derive(Deserialize)]
struct AnalyzeQuery {
  path: Option<String>,
}

#[derive(Deserialize)]
struct AnalyzeRequest {
  language: String,
  path: String,
}

#[derive(Serialize)]
struct ApiResponse<T> {
  success: bool,
  data: Option<T>,
  error: Option<String>,
}

#[derive(Serialize)]
struct AnalysisResponse {
  language: String,
  file_count: usize,
  total_keywords: usize,
  keyword_counts: HashMap<String, usize>,
  files_analyzed: Vec<String>,
}

impl From<AnalysisResult> for AnalysisResponse {
  fn from(result: AnalysisResult) -> Self {
    let language_name = match result.language {
      Language::Rust => "rust",
      Language::JavaScript => "javascript",
      Language::Ruby => "ruby",
      Language::Golang => "go",
      Language::Python => "python",
    };

    Self {
      language: language_name.to_string(),
      file_count: result.file_count,
      total_keywords: result.total_keywords,
      keyword_counts: result.keyword_counts,
      files_analyzed: result.files_analyzed,
    }
  }
}

#[tokio::main]
async fn main() {
  let app = Router::new()
    .route("/", get(health_check))
    .route("/health", get(health_check))
    .route("/analyze", post(analyze_post))
    .route("/analyze/:language/*path", get(analyze_get))
    .route("/languages", get(get_supported_languages));

  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

  println!("🚀 Keyword Analyzer API Server starting on http://0.0.0.0:3000");
  println!("📖 API Documentation:");
  println!("  GET  /health                     - Health check");
  println!("  GET  /languages                  - List supported languages");
  println!("  GET  /analyze/:language/:path    - Analyze path with language");
  println!("  POST /analyze                    - Analyze with JSON body");
  println!();

  axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> Json<ApiResponse<&'static str>> {
  Json(ApiResponse {
    success: true,
    data: Some("Keyword Analyzer API Server is running"),
    error: None,
  })
}

async fn get_supported_languages() -> Json<ApiResponse<Vec<&'static str>>> {
  Json(ApiResponse {
    success: true,
    data: Some(vec!["rust", "javascript", "ruby", "go", "python"]),
    error: None,
  })
}

async fn analyze_get(
  Path((language, path)): Path<(String, String)>,
  Query(params): Query<AnalyzeQuery>,
) -> Result<Json<ApiResponse<AnalysisResponse>>, StatusCode> {
  let lang = parse_language(&language)?;
  let target_path = params.path.unwrap_or(path);

  match KeywordAnalyzer::analyze_path(&target_path, lang) {
    Ok(result) => Ok(Json(ApiResponse {
      success: true,
      data: Some(result.into()),
      error: None,
    })),
    Err(e) => Ok(Json(ApiResponse {
      success: false,
      data: None,
      error: Some(e.to_string()),
    })),
  }
}

async fn analyze_post(
  Json(request): Json<AnalyzeRequest>,
) -> Result<Json<ApiResponse<AnalysisResponse>>, StatusCode> {
  let lang = parse_language(&request.language)?;

  match KeywordAnalyzer::analyze_path(&request.path, lang) {
    Ok(result) => Ok(Json(ApiResponse {
      success: true,
      data: Some(result.into()),
      error: None,
    })),
    Err(e) => Ok(Json(ApiResponse {
      success: false,
      data: None,
      error: Some(e.to_string()),
    })),
  }
}

fn parse_language(lang_str: &str) -> Result<Language, StatusCode> {
  match lang_str.to_lowercase().as_str() {
    "rust" | "rs" => Ok(Language::Rust),
    "javascript" | "js" | "typescript" | "ts" => Ok(Language::JavaScript),
    "ruby" | "rb" => Ok(Language::Ruby),
    "go" | "golang" => Ok(Language::Golang),
    "python" | "py" => Ok(Language::Python),
    _ => Err(StatusCode::BAD_REQUEST),
  }
}
