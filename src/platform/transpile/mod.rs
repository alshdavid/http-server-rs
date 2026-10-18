//! JIT transpilation of sources into something a browser can run.
//!
//! The transpilers are self-contained: [`typescript`] turns TypeScript and
//! TSX into JavaScript and [`scss`] turns SCSS and Sass into CSS. Each exposes
//! a transformer that owns reading the source, resolving its configuration and
//! producing the output.

pub mod scss;
pub mod typescript;
