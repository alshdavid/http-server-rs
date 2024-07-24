use colored::Colorize;

pub fn print_config(
  key: &str,
  value: &str,
) {
  println!("🔧 {}: {}", key.bold(), value.bold().bright_black());
}
