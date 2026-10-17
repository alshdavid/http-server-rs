use std::io::Write;

use brotli as brotli_rs;

pub fn brotli(input: &[u8]) -> Vec<u8> {
  let mut writer = brotli_rs::CompressorWriter::new(Vec::new(), 4096, 11, 22);
  writer.write_all(input).unwrap();
  writer.into_inner()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
  Brotli,
  Gzip,
  Identity,
}

impl Encoding {
  pub fn token(self) -> &'static str {
    match self {
      Encoding::Brotli => "br",
      Encoding::Gzip => "gzip",
      Encoding::Identity => "identity",
    }
  }
}

/// Pick the best encoding the client accepts from `available`, honouring
/// `Accept-Encoding` q-values and preferring `available`'s order on ties.
///
/// A missing header, or one that rejects every availability, yields
/// [`Encoding::Identity`].
pub fn negotiate_encoding(
  accept_encoding: Option<&str>,
  available: &[Encoding],
) -> Encoding {
  let Some(header) = accept_encoding else {
    return Encoding::Identity;
  };

  let mut best_coding: Option<(Encoding, f32)> = None;

  for encoding in available.iter().copied() {
    let q = coding_quality(header, encoding.token());
    if best_coding.is_none_or(|(_, best_q)| q > best_q) {
      best_coding = Some((encoding, q));
    }
  }

  match best_coding {
    Some((encoding, q)) if q > 0.0 && q >= identity_quality(header) => encoding,
    _ => Encoding::Identity,
  }
}

fn coding_quality(
  header: &str,
  coding: &str,
) -> f32 {
  let mut wildcard: Option<f32> = None;

  for part in header.split(',') {
    let part = part.trim();
    if part.is_empty() {
      continue;
    }

    let mut segments = part.split(';');
    let name = segments.next().unwrap_or_default().trim();
    let q = segments
      .filter_map(|param| param.trim().strip_prefix("q="))
      .find_map(|value| value.trim().parse::<f32>().ok())
      .unwrap_or(1.0);

    if name.eq_ignore_ascii_case(coding) {
      return q;
    }

    if name == "*" {
      wildcard = Some(q);
    }
  }

  wildcard.unwrap_or(0.0)
}

/// Identity is acceptable by default unless the header refuses it, either
/// with `identity;q=0` or a matching `*;q=0`.
fn identity_quality(header: &str) -> f32 {
  let has_wildcard = header
    .split(',')
    .any(|part| part.trim().split(';').next() == Some("*"));

  let explicit = header.split(',').any(|part| {
    part
      .trim()
      .split(';')
      .next()
      .is_some_and(|name| name.trim().eq_ignore_ascii_case("identity"))
  });

  if explicit || has_wildcard {
    coding_quality(header, "identity")
  } else {
    1.0
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const ALL: [Encoding; 2] = [Encoding::Brotli, Encoding::Gzip];

  #[test]
  fn missing_header_prefers_identity() {
    assert_eq!(negotiate_encoding(None, &ALL), Encoding::Identity);
  }

  #[test]
  fn explicit_brotli_is_selected() {
    assert_eq!(negotiate_encoding(Some("br"), &ALL), Encoding::Brotli);
  }

  #[test]
  fn explicit_gzip_is_selected() {
    assert_eq!(negotiate_encoding(Some("gzip"), &ALL), Encoding::Gzip);
  }

  #[test]
  fn brotli_preferred_on_equal_quality() {
    assert_eq!(negotiate_encoding(Some("gzip, br"), &ALL), Encoding::Brotli);
  }

  #[test]
  fn higher_quality_wins() {
    assert_eq!(
      negotiate_encoding(Some("br;q=0.5, gzip;q=1.0"), &ALL),
      Encoding::Gzip
    );
  }

  #[test]
  fn zero_quality_rejects_coding() {
    assert_eq!(
      negotiate_encoding(Some("br;q=0, gzip"), &ALL),
      Encoding::Gzip
    );
  }

  #[test]
  fn rejects_all_available_codings() {
    assert_eq!(
      negotiate_encoding(Some("gzip;q=0, br;q=0"), &ALL),
      Encoding::Identity
    );
  }

  #[test]
  fn identity_only_means_identity() {
    assert_eq!(
      negotiate_encoding(Some("identity"), &ALL),
      Encoding::Identity
    );
  }

  #[test]
  fn wildcard_accepts_brotli() {
    assert_eq!(negotiate_encoding(Some("*"), &ALL), Encoding::Brotli);
  }

  #[test]
  fn identity_refused_by_wildcard_falls_back_to_coding() {
    assert_eq!(
      negotiate_encoding(Some("identity;q=0, *;q=0.5"), &ALL),
      Encoding::Brotli
    );
  }

  #[test]
  fn identity_refused_and_no_coding_available() {
    assert_eq!(
      negotiate_encoding(Some("identity;q=0, br;q=0, gzip;q=0"), &ALL),
      Encoding::Identity
    );
  }

  #[test]
  fn only_brotli_available_falls_back_to_identity_for_gzip_client() {
    assert_eq!(
      negotiate_encoding(Some("gzip"), &[Encoding::Brotli]),
      Encoding::Identity
    );
  }
}
