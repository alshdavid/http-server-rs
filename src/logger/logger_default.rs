use colored::Colorize;

use super::Logger;

#[derive(Default)]
pub struct LoggerDefault {}

impl Logger for LoggerDefault {
  fn println(
    &self,
    message: &str,
  ) {
    println!("{}", message);
  }

  fn br(&self) {
    println!();
  }

  fn print_config(
    &self,
    key: &str,
    value: &bool,
  ) {
    let message = if *value { "Enabled" } else { "Disabled" };
    println!("🔧 {}: {}", key.bold(), message.bold().bright_black());
  }

  fn print_config_str(
    &self,
    key: &str,
    value: &str,
  ) {
    println!("🔧 {}: {}", key.bold(), value.bold().bright_black());
  }
}
