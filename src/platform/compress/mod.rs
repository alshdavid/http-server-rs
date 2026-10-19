use std::io::Write;

use brotli as brotli_rs;

pub fn brotli(input: &[u8]) -> Vec<u8> {
  let mut writer = brotli_rs::CompressorWriter::new(Vec::new(), 4096, 11, 22);
  writer.write_all(input).unwrap();
  writer.into_inner()
}

/// The content codings the server can produce, in preference order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Encoding {
  Brotli,
  Gzip,
}

impl Encoding {
  pub fn name(self) -> &'static str {
    match self {
      Encoding::Brotli => "br",
      Encoding::Gzip => "gzip",
    }
  }
}

/// Pick the most preferred supported coding from an `Accept-Encoding` header.
pub fn negotiate(accept_encoding: Option<&str>) -> Option<Encoding> {
  let header = accept_encoding?;

  [Encoding::Brotli, Encoding::Gzip]
    .into_iter()
    .filter_map(|encoding| {
      let quality = quality(header, encoding.name())?;
      (quality > 0.0).then_some((encoding, quality))
    })
    .fold(
      None,
      |best: Option<(Encoding, f32)>, candidate| match best {
        Some((_, quality)) if quality >= candidate.1 => best,
        _ => Some(candidate),
      },
    )
    .map(|(encoding, _)| encoding)
}

fn quality(
  header: &str,
  name: &str,
) -> Option<f32> {
  let mut wildcard = None;

  for part in header.split(',') {
    let mut params = part.split(';');
    let token = params.next().unwrap_or_default().trim();

    let quality = params
      .find_map(|param| param.trim().strip_prefix("q=")?.trim().parse::<f32>().ok())
      .unwrap_or(1.0);

    if token.eq_ignore_ascii_case(name) {
      return Some(quality);
    }

    if token == "*" {
      wildcard = Some(quality);
    }
  }

  wildcard
}

#[cfg(test)]
mod tests {
  use super::*;

  fn accepted(header: &str) -> Option<&'static str> {
    negotiate(Some(header)).map(Encoding::name)
  }

  #[test]
  fn prefers_brotli_when_accepted() {
    assert_eq!(accepted("gzip, deflate, br"), Some("br"));
  }

  #[test]
  fn honours_quality_values() {
    assert_eq!(accepted("br;q=0.5, gzip"), Some("gzip"));
  }

  #[test]
  fn falls_back_to_gzip() {
    assert_eq!(accepted("gzip"), Some("gzip"));
  }

  #[test]
  fn returns_none_when_nothing_supported() {
    assert_eq!(accepted("deflate"), None);
    assert_eq!(accepted("br;q=0"), None);
    assert_eq!(accepted("identity"), None);
    assert_eq!(negotiate(None), None);
  }

  #[test]
  fn honours_wildcard() {
    assert_eq!(accepted("*"), Some("br"));
    assert_eq!(accepted("br;q=0, *"), Some("gzip"));
  }
}
