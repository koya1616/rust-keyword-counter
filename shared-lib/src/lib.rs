use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::process::Command;

pub mod dart;
pub mod golang;
pub mod javascript;
pub mod python;
pub mod ruby;
pub mod rust;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum OutputFormat {
  Plain,
  Json,
  Csv,
  Html,
  Graph,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Language {
  Rust,
  JavaScript,
  Ruby,
  Golang,
  Python,
  Dart,
}

#[derive(Serialize, Deserialize)]
pub struct AnalysisResult {
  pub language: Language,
  pub file_count: usize,
  pub total_keywords: usize,
  pub keyword_counts: HashMap<String, usize>,
  pub files_analyzed: Vec<String>,
}

impl AnalysisResult {
  pub fn new(language: Language) -> Self {
    Self {
      language,
      file_count: 0,
      total_keywords: 0,
      keyword_counts: HashMap::new(),
      files_analyzed: Vec::new(),
    }
  }

  pub fn add_file(&mut self, file_path: String, counts: HashMap<String, usize>) {
    self.file_count += 1;
    self.files_analyzed.push(file_path);

    for (keyword, count) in counts {
      *self.keyword_counts.entry(keyword).or_insert(0) += count;
      self.total_keywords += count;
    }
  }

  pub fn get_sorted_counts(&self) -> Vec<(&String, &usize)> {
    let mut sorted_counts: Vec<_> = self.keyword_counts.iter().collect();
    sorted_counts.sort_by(|a, b| b.1.cmp(a.1));
    sorted_counts
  }
}

pub struct KeywordAnalyzer;

impl KeywordAnalyzer {
  pub fn analyze_path(
    path: &str,
    language: Language,
  ) -> Result<AnalysisResult, Box<dyn std::error::Error>> {
    let mut result = AnalysisResult::new(language);
    let mut total_counts: HashMap<String, usize> = HashMap::new();
    let mut file_count = 0;

    let actual_path = if is_git_url(path) {
      clone_git_repo(path)?
    } else {
      path.to_string()
    };

    let analysis_result = match language {
      Language::Rust => rust::analyze_directory(&actual_path, &mut total_counts, &mut file_count),
      Language::JavaScript => {
        javascript::analyze_directory(&actual_path, &mut total_counts, &mut file_count)
      }
      Language::Ruby => ruby::analyze_directory(&actual_path, &mut total_counts, &mut file_count),
      Language::Golang => {
        golang::analyze_directory(&actual_path, &mut total_counts, &mut file_count)
      }
      Language::Python => {
        python::analyze_directory(&actual_path, &mut total_counts, &mut file_count)
      }
      Language::Dart => dart::analyze_directory(&actual_path, &mut total_counts, &mut file_count),
    };

    analysis_result?;

    result.file_count = file_count;
    result.keyword_counts = total_counts;
    result.total_keywords = result.keyword_counts.values().sum();

    // Clean up if it was a git repo
    if is_git_url(path) {
      let _ = fs::remove_dir_all(&actual_path);
    }

    Ok(result)
  }

  pub fn format_output(
    result: &AnalysisResult,
    format: OutputFormat,
    output_file: Option<String>,
  ) -> Result<String, Box<dyn std::error::Error>> {
    let sorted_counts = result.get_sorted_counts();

    match format {
      OutputFormat::Plain => Ok(format_plain(
        &sorted_counts,
        result.file_count,
        result.language,
      )),
      OutputFormat::Json => {
        let file_path =
          output_file.unwrap_or_else(|| generate_default_filename(result.language, "json"));
        write_json_to_file(&sorted_counts, result.file_count, &file_path)?;
        Ok(format!("JSON results written to: {file_path}"))
      }
      OutputFormat::Csv => {
        let file_path =
          output_file.unwrap_or_else(|| generate_default_filename(result.language, "csv"));
        write_csv_to_file(&sorted_counts, result.file_count, &file_path)?;
        Ok(format!("CSV results written to: {file_path}"))
      }
      OutputFormat::Html => {
        let file_path =
          output_file.unwrap_or_else(|| generate_default_filename(result.language, "html"));
        write_html_to_file(
          &sorted_counts,
          result.file_count,
          result.language,
          &file_path,
        )?;
        Ok(format!("HTML results written to: {file_path}"))
      }
      OutputFormat::Graph => {
        let file_path =
          output_file.unwrap_or_else(|| generate_default_filename(result.language, "svg"));
        write_graph_to_file(
          &sorted_counts,
          result.file_count,
          result.language,
          &file_path,
        )?;
        Ok(format!("Graph results written to: {file_path}"))
      }
    }
  }
}

pub fn is_git_url(input: &str) -> bool {
  input.starts_with("https://github.com/")
    || input.starts_with("http://github.com/")
    || input.starts_with("github.com/")
    || input.starts_with("https://gitlab.com/")
    || input.starts_with("http://gitlab.com/")
    || input.starts_with("gitlab.com/")
}

pub fn is_valid_github_repo_url(url: &str) -> bool {
  // Check if it's a valid GitHub URL format
  if let Some(path) = url.strip_prefix("https://github.com/") {
    // Should have format: username/repository
    let parts: Vec<&str> = path.split('/').collect();
    parts.len() >= 2 && !parts[0].is_empty() && !parts[1].is_empty()
  } else if let Some(path) = url.strip_prefix("http://github.com/") {
    let parts: Vec<&str> = path.split('/').collect();
    parts.len() >= 2 && !parts[0].is_empty() && !parts[1].is_empty()
  } else if let Some(path) = url.strip_prefix("github.com/") {
    let parts: Vec<&str> = path.split('/').collect();
    parts.len() >= 2 && !parts[0].is_empty() && !parts[1].is_empty()
  } else {
    false
  }
}

pub fn is_valid_gitlab_repo_url(url: &str) -> bool {
  // Check if it's a valid GitLab URL format
  if let Some(path) = url.strip_prefix("https://gitlab.com/") {
    // Should have format: username/repository or group/subgroup/repository
    let parts: Vec<&str> = path.split('/').collect();
    parts.len() >= 2 && !parts[0].is_empty() && !parts[1].is_empty()
  } else if let Some(path) = url.strip_prefix("http://gitlab.com/") {
    let parts: Vec<&str> = path.split('/').collect();
    parts.len() >= 2 && !parts[0].is_empty() && !parts[1].is_empty()
  } else if let Some(path) = url.strip_prefix("gitlab.com/") {
    let parts: Vec<&str> = path.split('/').collect();
    parts.len() >= 2 && !parts[0].is_empty() && !parts[1].is_empty()
  } else {
    false
  }
}

pub fn clone_git_repo(url: &str) -> Result<String, Box<dyn std::error::Error>> {
  let temp_dir = format!("/tmp/rust_analyzer_{}", std::process::id());

  let normalized_url = if url.starts_with("github.com/") || url.starts_with("gitlab.com/") {
    format!("https://{url}")
  } else {
    url.to_string()
  };

  eprintln!("Cloning repository: {normalized_url}");
  eprintln!("Target directory: {temp_dir}");

  let output = Command::new("git")
    .args(["clone", "--depth", "1", &normalized_url, &temp_dir])
    .output()?;

  if !output.status.success() {
    let error_msg = String::from_utf8_lossy(&output.stderr);
    return Err(format!("Failed to clone repository: {error_msg}").into());
  }

  eprintln!("Repository cloned successfully");
  Ok(temp_dir)
}

fn format_plain(
  sorted_counts: &[(&String, &usize)],
  file_count: usize,
  language: Language,
) -> String {
  let language_name = match language {
    Language::Rust => "Rust",
    Language::JavaScript => "JavaScript/TypeScript",
    Language::Ruby => "Ruby",
    Language::Golang => "Go",
    Language::Python => "Python",
    Language::Dart => "Dart",
  };

  let mut output = String::new();
  output.push_str(&format!(
    "\n=== {language_name} Keyword Analysis Results ===\n"
  ));
  output.push_str(&format!("Files analyzed: {file_count}\n"));
  output.push_str(&format!(
    "Total keywords found: {}\n\n",
    sorted_counts
      .iter()
      .map(|(_, count)| **count)
      .sum::<usize>()
  ));

  for (keyword, count) in sorted_counts {
    if **count > 0 {
      output.push_str(&format!("{keyword:12} : {count}\n"));
    }
  }

  output
}

fn generate_default_filename(language: Language, extension: &str) -> String {
  let language_name = match language {
    Language::Rust => "rust",
    Language::JavaScript => "javascript",
    Language::Ruby => "ruby",
    Language::Golang => "go",
    Language::Python => "python",
    Language::Dart => "dart",
  };

  let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs();

  format!("keyword_analysis_{language_name}_{timestamp}.{extension}")
}

fn write_json_to_file(
  sorted_counts: &[(&String, &usize)],
  file_count: usize,
  file_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  let mut file = fs::File::create(file_path)?;
  let total_keywords = sorted_counts
    .iter()
    .map(|(_, count)| **count)
    .sum::<usize>();

  writeln!(file, "{{")?;
  writeln!(file, "  \"files_analyzed\": {file_count},")?;
  writeln!(file, "  \"total_keywords\": {total_keywords},")?;
  writeln!(file, "  \"keywords\": {{")?;

  let mut first = true;
  for (keyword, count) in sorted_counts {
    if **count > 0 {
      if !first {
        writeln!(file, ",")?;
      }
      write!(file, "    \"{keyword}\": {count}")?;
      first = false;
    }
  }

  if !first {
    writeln!(file)?;
  }
  writeln!(file, "  }}")?;
  writeln!(file, "}}")?;

  println!("JSON results written to: {file_path}");
  Ok(())
}

fn write_csv_to_file(
  sorted_counts: &[(&String, &usize)],
  file_count: usize,
  file_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  let mut file = fs::File::create(file_path)?;
  let total_keywords = sorted_counts
    .iter()
    .map(|(_, count)| **count)
    .sum::<usize>();

  writeln!(file, "keyword,count")?;
  writeln!(file, "_files_analyzed,{file_count}")?;
  writeln!(file, "_total_keywords,{total_keywords}")?;

  for (keyword, count) in sorted_counts {
    if **count > 0 {
      writeln!(file, "{keyword},{count}")?;
    }
  }

  println!("CSV results written to: {file_path}");
  Ok(())
}

fn write_html_to_file(
  sorted_counts: &[(&String, &usize)],
  file_count: usize,
  language: Language,
  file_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  let mut file = fs::File::create(file_path)?;
  let language_name = match language {
    Language::Rust => "Rust",
    Language::JavaScript => "JavaScript/TypeScript",
    Language::Ruby => "Ruby",
    Language::Golang => "Go",
    Language::Python => "Python",
    Language::Dart => "Dart",
  };

  let total_keywords: usize = sorted_counts.iter().map(|(_, count)| **count).sum();

  let html_content = format!(
    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{language_name} Keyword Analysis Results</title>
    <style>
        body {{ font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 40px; background-color: #f5f5f5; }}
        .container {{ max-width: 800px; margin: 0 auto; background: white; padding: 30px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
        h1 {{ color: #333; text-align: center; margin-bottom: 30px; border-bottom: 3px solid #007acc; padding-bottom: 10px; }}
        .summary {{ background: #f8f9fa; padding: 20px; border-radius: 8px; margin-bottom: 30px; border-left: 4px solid #007acc; }}
        .summary h2 {{ margin-top: 0; color: #495057; }}
        .stat {{ display: inline-block; margin-right: 30px; }}
        .stat-value {{ font-size: 24px; font-weight: bold; color: #007acc; }}
        .stat-label {{ font-size: 14px; color: #6c757d; }}
        .keywords-table {{ width: 100%; border-collapse: collapse; margin-top: 20px; }}
        .keywords-table th {{ background: #007acc; color: white; padding: 12px; text-align: left; font-weight: 600; }}
        .keywords-table td {{ padding: 10px 12px; border-bottom: 1px solid #dee2e6; }}
        .keywords-table tr:nth-child(even) {{ background-color: #f8f9fa; }}
        .keywords-table tr:hover {{ background-color: #e3f2fd; }}
        .keyword {{ font-family: 'Consolas', 'Monaco', monospace; font-weight: 600; color: #495057; }}
        .count {{ font-weight: bold; color: #007acc; }}
        .progress-bar {{ width: 100%; height: 8px; background: #e9ecef; border-radius: 4px; overflow: hidden; }}
        .progress-fill {{ height: 100%; background: linear-gradient(90deg, #007acc, #40a9ff); transition: width 0.3s ease; }}
        .footer {{ text-align: center; margin-top: 30px; color: #6c757d; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>{language_name} Keyword Analysis Results</h1>
        <div class="summary">
            <h2>Analysis Summary</h2>
            <div class="stat">
                <div class="stat-value">{file_count}</div>
                <div class="stat-label">Files Analyzed</div>
            </div>
            <div class="stat">
                <div class="stat-value">{total_keywords}</div>
                <div class="stat-label">Total Keywords Found</div>
            </div>
        </div>"#
  );

  write!(file, "{html_content}")?;

  if !sorted_counts.is_empty() && total_keywords > 0 {
    let max_count = sorted_counts
      .iter()
      .map(|(_, count)| **count)
      .max()
      .unwrap_or(1);

    writeln!(
      file,
      r#"        <table class="keywords-table">
            <thead>
                <tr>
                    <th>Keyword</th>
                    <th>Count</th>
                    <th>Distribution</th>
                </tr>
            </thead>
            <tbody>"#
    )?;

    for (keyword, count) in sorted_counts {
      if **count > 0 {
        let percentage = ((**count as f64) / (max_count as f64) * 100.0) as u32;
        writeln!(
          file,
          r#"                <tr>
                    <td class="keyword">{keyword}</td>
                    <td class="count">{count}</td>
                    <td>
                        <div class="progress-bar">
                            <div class="progress-fill" style="width: {percentage}%;"></div>
                        </div>
                    </td>
                </tr>"#
        )?;
      }
    }

    writeln!(
      file,
      r#"            </tbody>
        </table>"#
    )?;
  } else {
    writeln!(
      file,
      r#"        <p style="text-align: center; color: #6c757d; font-style: italic;">No keywords found in the analyzed files.</p>"#
    )?;
  }

  writeln!(
    file,
    r#"        <div class="footer">
            <p>Generated by Multi-Language Keyword Analyzer</p>
        </div>
    </div>
</body>
</html>"#
  )?;

  println!("HTML results written to: {file_path}");
  Ok(())
}

fn write_graph_to_file(
  sorted_counts: &[(&String, &usize)],
  file_count: usize,
  language: Language,
  file_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  let mut file = fs::File::create(file_path)?;
  let language_name = match language {
    Language::Rust => "Rust",
    Language::JavaScript => "JavaScript/TypeScript",
    Language::Ruby => "Ruby",
    Language::Golang => "Go",
    Language::Python => "Python",
    Language::Dart => "Dart",
  };

  let total_keywords: usize = sorted_counts.iter().map(|(_, count)| **count).sum();
  let top_keywords: Vec<_> = sorted_counts
    .iter()
    .filter(|(_, count)| **count > 0)
    .take(20)
    .collect();

  if top_keywords.is_empty() {
    return Err("No keywords found to visualize".into());
  }

  let max_count = top_keywords
    .iter()
    .map(|(_, count)| **count)
    .max()
    .unwrap_or(1);
  let width = 1000;
  let height = 600;
  let margin = 60;
  let chart_height = height - 2 * margin - 50;
  let bar_width = (width - 2 * margin) / top_keywords.len().max(1);
  let colors = generate_color_palette(top_keywords.len());

  writeln!(file, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
  writeln!(
    file,
    "<svg width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\" xmlns=\"http://www.w3.org/2000/svg\">"
  )?;

  writeln!(
    file,
    r#"    <defs>
        <style>
            .title {{ font-family: 'Arial', sans-serif; font-size: 24px; font-weight: bold; text-anchor: middle; fill: #333; }}
            .subtitle {{ font-family: 'Arial', sans-serif; font-size: 14px; text-anchor: middle; fill: #666; }}
            .bar {{ cursor: pointer; transition: opacity 0.2s; }}
            .bar:hover {{ opacity: 0.8; }}
            .bar-label {{ font-family: 'Arial', sans-serif; font-size: 12px; text-anchor: middle; fill: #333; }}
            .count-label {{ font-family: 'Arial', sans-serif; font-size: 11px; text-anchor: middle; fill: #fff; font-weight: bold; }}
            .axis {{ stroke: #ccc; stroke-width: 1; }}
            .grid {{ stroke: #eee; stroke-width: 0.5; }}
        </style>
    </defs>"#
  )?;

  writeln!(
    file,
    "    <rect width=\"100%\" height=\"100%\" fill=\"#fafafa\"/>"
  )?;
  writeln!(
    file,
    "    <text x=\"{}\" y=\"30\" class=\"title\">{} Keyword Analysis</text>",
    width / 2,
    language_name
  )?;
  writeln!(file, "    <text x=\"{}\" y=\"50\" class=\"subtitle\">Files: {} | Total Keywords: {} | Top {} Keywords</text>", width / 2, file_count, total_keywords, top_keywords.len())?;

  let grid_lines = generate_grid_lines(margin, margin + 30, chart_height);
  write!(file, "{grid_lines}")?;

  for (i, (keyword, count)) in top_keywords.iter().enumerate() {
    let x = margin + i * bar_width;
    let bar_height = ((**count as f64) / (max_count as f64) * chart_height as f64) as usize;
    let y = margin + 30 + chart_height - bar_height;
    let color = &colors[i % colors.len()];

    writeln!(
      file,
      "    <rect class=\"bar\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\">",
      x + 2,
      y,
      bar_width - 4,
      bar_height,
      color
    )?;
    writeln!(file, "        <title>{keyword}: {count}</title>")?;
    writeln!(file, "    </rect>")?;

    if bar_height > 25 {
      writeln!(
        file,
        "    <text x=\"{}\" y=\"{}\" class=\"count-label\">{}</text>",
        x + bar_width / 2,
        y + 15,
        count
      )?;
    }

    let label_y = margin + 30 + chart_height + 15;
    let truncated_keyword = if keyword.len() > 8 {
      format!("{}...", &keyword[..5])
    } else {
      keyword.to_string()
    };

    writeln!(
      file,
      "    <text x=\"{}\" y=\"{}\" class=\"bar-label\" transform=\"rotate(-45, {}, {})\">{}</text>",
      x + bar_width / 2,
      label_y,
      x + bar_width / 2,
      label_y,
      truncated_keyword
    )?;
  }

  writeln!(
    file,
    "    <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" class=\"axis\"/>",
    margin,
    margin + 30,
    margin,
    margin + 30 + chart_height
  )?;
  writeln!(
    file,
    "    <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" class=\"axis\"/>",
    margin,
    margin + 30 + chart_height,
    margin + (width - 2 * margin),
    margin + 30 + chart_height
  )?;

  let y_labels = generate_y_axis_labels(margin - 10, margin + 30, chart_height, max_count);
  write!(file, "{y_labels}")?;
  writeln!(file, "</svg>")?;

  println!("Graph results written to: {file_path}");
  Ok(())
}

fn generate_color_palette(count: usize) -> Vec<String> {
  let base_colors = vec![
    "#3498db", "#e74c3c", "#2ecc71", "#f39c12", "#9b59b6", "#1abc9c", "#34495e", "#e67e22",
    "#95a5a6", "#f1c40f", "#c0392b", "#27ae60", "#8e44ad", "#16a085", "#2c3e50", "#d35400",
    "#7f8c8d", "#f4d03f", "#85c1e9", "#f8c471",
  ];

  (0..count)
    .map(|i| base_colors[i % base_colors.len()].to_string())
    .collect()
}

fn generate_grid_lines(x_start: usize, y_start: usize, height: usize) -> String {
  let mut lines = String::new();
  let grid_count = 5;

  for i in 0..=grid_count {
    let y = y_start + (height * i) / grid_count;
    lines.push_str(&format!(
      "    <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" class=\"grid\"/>\n",
      x_start,
      y,
      x_start + 880,
      y
    ));
  }

  lines
}

fn generate_y_axis_labels(x: usize, y_start: usize, height: usize, max_value: usize) -> String {
  let mut labels = String::new();
  let label_count = 5;

  for i in 0..=label_count {
    let y = y_start + height - (height * i) / label_count;
    let value = (max_value * i) / label_count;
    labels.push_str(&format!(
      "    <text x=\"{}\" y=\"{}\" class=\"bar-label\" text-anchor=\"end\">{}</text>\n",
      x,
      y + 4,
      value
    ));
  }

  labels
}

pub fn generate_json_content(sorted_counts: &[(&String, &usize)], file_count: usize) -> String {
  let total_keywords = sorted_counts
    .iter()
    .map(|(_, count)| **count)
    .sum::<usize>();

  let mut json = String::new();
  json.push_str("{\n");
  json.push_str(&format!("  \"files_analyzed\": {file_count},\n"));
  json.push_str(&format!("  \"total_keywords\": {total_keywords},\n"));
  json.push_str("  \"keywords\": {\n");

  let mut first = true;
  for (keyword, count) in sorted_counts {
    if **count > 0 {
      if !first {
        json.push_str(",\n");
      }
      json.push_str(&format!("    \"{keyword}\": {count}"));
      first = false;
    }
  }

  if !first {
    json.push('\n');
  }
  json.push_str("  }\n");
  json.push('}');

  json
}

pub fn generate_html_content(
  sorted_counts: &[(&String, &usize)],
  file_count: usize,
  language: Language,
) -> String {
  let language_name = match language {
    Language::Rust => "Rust",
    Language::JavaScript => "JavaScript/TypeScript",
    Language::Ruby => "Ruby",
    Language::Golang => "Go",
    Language::Python => "Python",
    Language::Dart => "Dart",
  };

  let total_keywords: usize = sorted_counts.iter().map(|(_, count)| **count).sum();

  let mut html = format!(
    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{language_name} Keyword Analysis Results</title>
    <style>
        body {{ font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 40px; background-color: #f5f5f5; }}
        .container {{ max-width: 800px; margin: 0 auto; background: white; padding: 30px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
        h1 {{ color: #333; text-align: center; margin-bottom: 30px; border-bottom: 3px solid #007acc; padding-bottom: 10px; }}
        .summary {{ background: #f8f9fa; padding: 20px; border-radius: 8px; margin-bottom: 30px; border-left: 4px solid #007acc; }}
        .summary h2 {{ margin-top: 0; color: #495057; }}
        .stat {{ display: inline-block; margin-right: 30px; }}
        .stat-value {{ font-size: 24px; font-weight: bold; color: #007acc; }}
        .stat-label {{ font-size: 14px; color: #6c757d; }}
        .keywords-table {{ width: 100%; border-collapse: collapse; margin-top: 20px; }}
        .keywords-table th {{ background: #007acc; color: white; padding: 12px; text-align: left; font-weight: 600; }}
        .keywords-table td {{ padding: 10px 12px; border-bottom: 1px solid #dee2e6; }}
        .keywords-table tr:nth-child(even) {{ background-color: #f8f9fa; }}
        .keywords-table tr:hover {{ background-color: #e3f2fd; }}
        .keyword {{ font-family: 'Consolas', 'Monaco', monospace; font-weight: 600; color: #495057; }}
        .count {{ font-weight: bold; color: #007acc; }}
        .progress-bar {{ width: 100%; height: 8px; background: #e9ecef; border-radius: 4px; overflow: hidden; }}
        .progress-fill {{ height: 100%; background: linear-gradient(90deg, #007acc, #40a9ff); transition: width 0.3s ease; }}
        .footer {{ text-align: center; margin-top: 30px; color: #6c757d; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>{language_name} Keyword Analysis Results</h1>
        <div class="summary">
            <h2>Analysis Summary</h2>
            <div class="stat">
                <div class="stat-value">{file_count}</div>
                <div class="stat-label">Files Analyzed</div>
            </div>
            <div class="stat">
                <div class="stat-value">{total_keywords}</div>
                <div class="stat-label">Total Keywords Found</div>
            </div>
        </div>"#
  );

  if !sorted_counts.is_empty() && total_keywords > 0 {
    let max_count = sorted_counts
      .iter()
      .map(|(_, count)| **count)
      .max()
      .unwrap_or(1);

    html.push_str(
      r#"        <table class="keywords-table">
            <thead>
                <tr>
                    <th>Keyword</th>
                    <th>Count</th>
                    <th>Distribution</th>
                </tr>
            </thead>
            <tbody>"#,
    );

    for (keyword, count) in sorted_counts {
      if **count > 0 {
        let percentage = ((**count as f64) / (max_count as f64) * 100.0) as u32;
        html.push_str(&format!(
          r#"                <tr>
                    <td class="keyword">{keyword}</td>
                    <td class="count">{count}</td>
                    <td>
                        <div class="progress-bar">
                            <div class="progress-fill" style="width: {percentage}%;"></div>
                        </div>
                    </td>
                </tr>"#
        ));
      }
    }

    html.push_str(
      r#"            </tbody>
        </table>"#,
    );
  } else {
    html.push_str(
      r#"        <p style="text-align: center; color: #6c757d; font-style: italic;">No keywords found in the analyzed files.</p>"#,
    );
  }

  html.push_str(
    r#"        <div class="footer">
            <p>Generated by Multi-Language Keyword Analyzer</p>
        </div>
    </div>
</body>
</html>"#,
  );

  html
}
