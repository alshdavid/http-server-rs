use std::collections::HashMap;

use colored::Colorize;

#[derive(Default)]
pub enum Logger {
  Quiet,
  #[default]
  Default,
}

impl Logger {
  pub fn println(
    &self,
    message: impl AsRef<str>,
  ) {
    match self {
      Logger::Quiet => return,
      Logger::Default => {}
    }
    println!("{}", message.as_ref());
  }

  pub fn br(&self) {
    match self {
      Logger::Quiet => return,
      Logger::Default => {}
    }
    println!();
  }

  pub fn print_folder(
    &self,
    message: &str,
  ) {
    match self {
      Logger::Quiet => return,
      Logger::Default => {}
    }
    let key = "Directory:".to_string();
    println!("📁 {:<19} {}", key.bold(), message);
  }

  pub fn print_config(
    &self,
    key: &str,
    value: &bool,
  ) {
    match self {
      Logger::Quiet => return,
      Logger::Default => {}
    }
    let message = if *value { "Enabled" } else { "Disabled" };
    let key = format!("{}:", key);
    println!("🔧 {:<19} {}", key.bold(), message);
  }

  pub fn print_headers(
    &self,
    headers: &HashMap<String, Vec<String>>,
  ) {
    match self {
      Logger::Quiet => return,
      Logger::Default => {}
    }
    let mut headers = headers.iter().collect::<Vec<(&String, &Vec<String>)>>();
    headers.sort();

    let Some(longest) = headers.iter().max_by_key(|v| v.0) else {
      return;
    };

    let min_len = longest.0.len() + 3;

    for (key, values) in headers {
      let key = format!("{}:", key);
      println!("📩 {:<min_len$} {}", key.bold(), values.join(", "));
    }
  }
}
