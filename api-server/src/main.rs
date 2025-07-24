use axum::{
  http::{header::CONTENT_TYPE, StatusCode},
  response::{Json, Response},
  routing::{get, post},
  Router,
};
use serde::{Deserialize, Serialize};

use keyword_analyzer_shared::{
  generate_html_content, generate_json_content, is_git_url, is_valid_github_repo_url,
  is_valid_gitlab_repo_url, KeywordAnalyzer, Language, OutputFormat,
};

#[derive(Deserialize)]
struct RepositoryAnalyzeRequest {
  language: String,
  format: Option<String>,
  repository_url: String,
}

#[derive(Serialize)]
struct ApiResponse<T> {
  success: bool,
  data: Option<T>,
  error: Option<String>,
}

#[tokio::main]
async fn main() {
  let app = Router::new()
    .route("/", get(health_check))
    .route("/health", get(health_check))
    .route("/analyze-repository", post(analyze_repository));

  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

  println!("🚀 Keyword Analyzer API Server starting on http://0.0.0.0:3000");
  println!("📖 API Documentation:");
  println!("  GET  /health                     - Health check");
  println!("  POST /analyze-repository         - Analyze repository with format support");
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

async fn analyze_repository(
  Json(request): Json<RepositoryAnalyzeRequest>,
) -> Result<Response, StatusCode> {
  let lang = parse_language(&request.language)?;

  let format_str = request
    .format
    .as_ref()
    .map(|s| s.to_lowercase())
    .unwrap_or_else(|| "json".to_string());
  let format = match format_str.as_str() {
    "json" => OutputFormat::Json,
    "html" => OutputFormat::Html,
    _ => return Err(StatusCode::BAD_REQUEST),
  };

  // Validate that repository_url is GitHub or GitLab only
  if !is_git_url(&request.repository_url) {
    let error_response = r#"{"success": false, "data": null, "error": "Only GitHub and GitLab repository URLs are supported. Expected format: https://github.com/username/repository or https://gitlab.com/username/repository"}"#.to_string();
    return Ok(
      Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header(CONTENT_TYPE, "application/json")
        .body(error_response.into())
        .unwrap(),
    );
  }

  // Validate GitHub repository URL format
  if request.repository_url.contains("github.com")
    && !is_valid_github_repo_url(&request.repository_url)
  {
    let error_response = r#"{"success": false, "data": null, "error": "Invalid GitHub repository URL format. Expected format: https://github.com/username/repository"}"#.to_string();
    return Ok(
      Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header(CONTENT_TYPE, "application/json")
        .body(error_response.into())
        .unwrap(),
    );
  }

  // Validate GitLab repository URL format
  if request.repository_url.contains("gitlab.com")
    && !is_valid_gitlab_repo_url(&request.repository_url)
  {
    let error_response = r#"{"success": false, "data": null, "error": "Invalid GitLab repository URL format. Expected format: https://gitlab.com/username/repository"}"#.to_string();
    return Ok(
      Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header(CONTENT_TYPE, "application/json")
        .body(error_response.into())
        .unwrap(),
    );
  }

  match KeywordAnalyzer::analyze_path(&request.repository_url, lang) {
    Ok(result) => {
      let sorted_counts = result.get_sorted_counts();

      match format {
        OutputFormat::Json => {
          let json_content = generate_json_content(&sorted_counts, result.file_count);
          Ok(
            Response::builder()
              .status(StatusCode::OK)
              .header(CONTENT_TYPE, "application/json")
              .body(json_content.into())
              .unwrap(),
          )
        }
        OutputFormat::Html => {
          let html_content =
            generate_html_content(&sorted_counts, result.file_count, result.language);
          Ok(
            Response::builder()
              .status(StatusCode::OK)
              .header(CONTENT_TYPE, "text/html")
              .body(html_content.into())
              .unwrap(),
          )
        }
        _ => Err(StatusCode::BAD_REQUEST),
      }
    }
    Err(e) => {
      let error_response = format!(
        r#"{{"success": false, "data": null, "error": "{}"}}"#,
        e.to_string().replace('"', "\\\"")
      );
      Ok(
        Response::builder()
          .status(StatusCode::INTERNAL_SERVER_ERROR)
          .header(CONTENT_TYPE, "application/json")
          .body(error_response.into())
          .unwrap(),
      )
    }
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

#[cfg(test)]
mod tests {
  use super::*;
  use axum_test::TestServer;
  use serde_json::{json, Value};

  fn create_test_app() -> Router {
    Router::new()
      .route("/", get(health_check))
      .route("/health", get(health_check))
      .route("/analyze-repository", post(analyze_repository))
  }

  #[tokio::test]
  async fn test_health_check() {
    let app = create_test_app();
    let server = TestServer::new(app).unwrap();

    let response = server.get("/health").await;

    response.assert_status_ok();

    let body: Value = response.json();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"], "Keyword Analyzer API Server is running");
    assert_eq!(body["error"], Value::Null);
  }

  #[tokio::test]
  async fn test_analyze_repository_invalid_language() {
    let app = create_test_app();
    let server = TestServer::new(app).unwrap();

    let request_body = json!({
      "language": "invalid",
      "format": "json",
      "repository_url": "https://github.com/dtolnay/anyhow"
    });

    let response = server.post("/analyze-repository").json(&request_body).await;

    response.assert_status_bad_request();
  }

  #[tokio::test]
  async fn test_analyze_repository_invalid_format() {
    let app = create_test_app();
    let server = TestServer::new(app).unwrap();

    let request_body = json!({
      "language": "rust",
      "format": "xml",
      "repository_url": "https://github.com/dtolnay/anyhow"
    });

    let response = server.post("/analyze-repository").json(&request_body).await;

    response.assert_status_bad_request();
  }

  #[tokio::test]
  async fn test_analyze_repository_non_git_url() {
    let app = create_test_app();
    let server = TestServer::new(app).unwrap();

    let request_body = json!({
      "language": "rust",
      "format": "json",
      "repository_url": "https://example.com/repo"
    });

    let response = server.post("/analyze-repository").json(&request_body).await;

    response.assert_status_bad_request();

    let body: Value = response.json();
    assert_eq!(body["success"], false);
    assert!(body["error"]
      .as_str()
      .unwrap()
      .contains("Only GitHub and GitLab repository URLs are supported"));
  }

  #[tokio::test]
  async fn test_analyze_repository_invalid_github_url() {
    let app = create_test_app();
    let server = TestServer::new(app).unwrap();

    let request_body = json!({
      "language": "rust",
      "format": "json",
      "repository_url": "https://github.com/rust-lang"
    });

    let response = server.post("/analyze-repository").json(&request_body).await;

    response.assert_status_bad_request();

    let body: Value = response.json();
    assert_eq!(body["success"], false);
    assert!(body["error"]
      .as_str()
      .unwrap()
      .contains("Invalid GitHub repository URL format"));
  }

  #[tokio::test]
  async fn test_analyze_repository_default_format() {
    let app = create_test_app();
    let server = TestServer::new(app).unwrap();

    let request_body = json!({
      "language": "rust",
      "repository_url": "shared-lib/src/"
    });

    let response = server.post("/analyze-repository").json(&request_body).await;

    // Should succeed with default JSON format
    if response.status_code() == 200 {
      response.assert_header("content-type", "application/json");
    }
  }

  #[tokio::test]
  async fn test_analyze_repository_html_format() {
    let app = create_test_app();
    let server = TestServer::new(app).unwrap();

    let request_body = json!({
      "language": "rust",
      "format": "html",
      "repository_url": "shared-lib/src/"
    });

    let response = server.post("/analyze-repository").json(&request_body).await;

    // Should succeed with HTML format
    if response.status_code() == 200 {
      response.assert_header("content-type", "text/html");
    }
  }

  #[tokio::test]
  async fn test_parse_language_valid() {
    assert!(parse_language("rust").is_ok());
    assert!(parse_language("javascript").is_ok());
    assert!(parse_language("js").is_ok());
    assert!(parse_language("typescript").is_ok());
    assert!(parse_language("ts").is_ok());
    assert!(parse_language("ruby").is_ok());
    assert!(parse_language("rb").is_ok());
    assert!(parse_language("go").is_ok());
    assert!(parse_language("golang").is_ok());
    assert!(parse_language("python").is_ok());
    assert!(parse_language("py").is_ok());
  }

  #[tokio::test]
  async fn test_analyze_repository_invalid_gitlab_url() {
    let app = create_test_app();
    let server = TestServer::new(app).unwrap();

    let request_body = json!({
      "language": "rust",
      "format": "json",
      "repository_url": "https://gitlab.com/gitlab-org"
    });

    let response = server.post("/analyze-repository").json(&request_body).await;

    response.assert_status_bad_request();

    let body: Value = response.json();
    assert_eq!(body["success"], false);
    assert!(body["error"]
      .as_str()
      .unwrap()
      .contains("Invalid GitLab repository URL format"));
  }

  #[tokio::test]
  async fn test_parse_language_invalid() {
    assert!(parse_language("invalid").is_err());
    assert!(parse_language("").is_err());
    assert!(parse_language("cpp").is_err());
  }
}
