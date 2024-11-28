use colored::Colorize;

use crate::config::Config;

pub struct Logger {
  quiet: bool,
}

impl Logger {
  pub fn new(config: &Config) -> Self {
    Self {
      quiet: config.quiet.clone(),
    }
  }

  pub fn println<M: AsRef<str>>(
    &self,
    message: M,
  ) {
    if !self.quiet {
      println!("{}", message.as_ref());
    }
  }

  pub fn br(&self) {
    if !self.quiet {
      println!("");
    }
  }

  pub fn print_config(
    &self,
    key: &str,
    value: &bool,
  ) {
    if !self.quiet {
      let message = if *value { "Enabled" } else { "Disabled" };
      println!("🔧 {}: {}", key.bold(), message.bold().bright_black());
    }
  }

  pub fn print_config_str(
    &self,
    key: &str,
    value: &str,
  ) {
    if !self.quiet {
      println!("🔧 {}: {}", key.bold(), value.bold().bright_black());
    }
  }
}
