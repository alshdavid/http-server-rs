use super::Logger;

#[derive(Default)]
pub struct LoggerNoop {}

impl Logger for LoggerNoop {
  fn println(
    &self,
    _message: &str,
  ) {
  }

  fn br(&self) {}

  fn print_config(
    &self,
    _key: &str,
    _value: &bool,
  ) {
  }

  fn print_config_str(
    &self,
    _key: &str,
    _value: &str,
  ) {
  }
}
