use std::io::Write;

use brotli as brotli_rs;
use flate2::Compression;
use flate2::write::GzEncoder;

pub fn brotli(input: &[u8]) -> Vec<u8> {
  let mut writer = brotli_rs::CompressorWriter::new(Vec::new(), 4096, 11, 22);
  writer.write_all(input).unwrap();
  writer.into_inner()
}

pub fn gzip(input: &[u8]) -> Vec<u8> {
  let mut writer = GzEncoder::new(Vec::new(), Compression::best());
  writer.write_all(input).unwrap();
  writer.finish().unwrap()
}
