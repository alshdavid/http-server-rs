use super::get_intranet_ip;

pub struct DomainOptions<'a> {
  pub domain: &'a str,
  pub domain_pretty: &'a str,
  pub port: usize,
}

pub fn get_domain(config: &DomainOptions) -> Option<String> {
  let intranet_domain = get_intranet_ip();
  let Some(intranet_domain_str) = intranet_domain.as_ref() else {
    return None;
  };
  if intranet_domain_str == &config.domain_pretty || intranet_domain_str == &config.domain {
    return None;
  }
  Some(format!("{}:{}", intranet_domain_str, config.port))
}
