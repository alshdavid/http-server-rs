pub trait Logger: Send + Sync {
  fn println(
    &self,
    message: &str,
  );
  fn br(&self);
  fn print_config(
    &self,
    key: &str,
    value: &bool,
  );
  fn print_config_str(
    &self,
    key: &str,
    value: &str,
  );
}
