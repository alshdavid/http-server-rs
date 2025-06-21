#![allow(unused)]

use base64::prelude::*;

pub fn encode<S: AsRef<str>>(input: S) -> String {
  BASE64_STANDARD.encode(input.as_ref().as_bytes())
}

pub fn decode_bytes<S: AsRef<str>>(input: S) -> anyhow::Result<Vec<u8>> {
  Ok(BASE64_STANDARD.decode(input.as_ref().as_bytes())?)
}

pub fn decode_string<S: AsRef<str>>(input: S) -> anyhow::Result<String> {
  Ok(String::from_utf8(decode_bytes(input)?)?)
}
