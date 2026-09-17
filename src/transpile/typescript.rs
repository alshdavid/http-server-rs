use oxc::allocator::Allocator;
use oxc::ast::ast::SourceType;
use oxc::codegen::Codegen;
use oxc::codegen::CodegenOptions;
use oxc::parser::Parser;
use oxc::semantic::SemanticBuilder;
use oxc::transformer::JsxOptions;
use oxc::transformer::JsxRuntime;
use oxc::transformer::TransformOptions;
use oxc::transformer::Transformer;
use oxc::transformer::TypeScriptOptions;

use super::tsconfig::JsxMode;
use super::TransformerContext;
use super::TransformerResult;

/// Options picked from the tsconfig used to drive the transform.
#[derive(Debug, Clone, Default)]
struct JsxConfig {
  runtime: Option<JsxRuntime>,
  pragma: Option<String>,
  pragma_frag: Option<String>,
  import_source: Option<String>,
  development: bool,
  /// Explicitly preserve JSX in the output (skip the JSX transform).
  preserve: bool,
}

/// Translate the nearest tsconfig's `jsx` option into transform settings.
fn jsx_config_from_tsconfig(ctx: &TransformerContext) -> JsxConfig {
  let tsconfig = &ctx.tsconfig;

  let pragma = tsconfig
    .jsx_factory
    .clone()
    .unwrap_or_else(|| "React.createElement".to_string());

  let pragma_frag = tsconfig
    .jsx_fragment_factory
    .clone()
    .unwrap_or_else(|| "React.Fragment".to_string());

  // Options for the classic runtime, honouring custom factories.
  let classic = || JsxConfig {
    runtime: Some(JsxRuntime::Classic),
    pragma: Some(pragma.clone()),
    pragma_frag: Some(pragma_frag.clone()),
    ..JsxConfig::default()
  };

  // Options for the automatic runtime, honouring a custom import source.
  let automatic = |development: bool| JsxConfig {
    runtime: Some(JsxRuntime::Automatic),
    import_source: Some(
      tsconfig
        .jsx_import_source
        .clone()
        .unwrap_or_else(|| "react".to_string()),
    ),
    development,
    ..JsxConfig::default()
  };

  match tsconfig.jsx {
    // `react` - the classic runtime
    Some(JsxMode::React) => classic(),
    // `react-jsx` / `react-jsxdev` - the automatic runtime
    Some(JsxMode::ReactJsx) => automatic(false),
    Some(JsxMode::ReactJsxDev) => automatic(true),
    // `react-native` emits JSX and defers the transform to the bundler.
    // Treat it like `preserve` for our purposes.
    Some(JsxMode::Preserve) | Some(JsxMode::ReactNative) => JsxConfig {
      preserve: true,
      ..JsxConfig::default()
    },
    // No `jsx` setting: fall back to the classic runtime, which does not
    // require the browser to resolve `react/jsx-runtime`.
    None => classic(),
  }
}

pub fn transpile(ctx: TransformerContext) -> anyhow::Result<TransformerResult> {
  let jsx_config = jsx_config_from_tsconfig(&ctx);

  let source = String::from_utf8(ctx.content)?;

  let allocator = Allocator::default();

  let source_type = match ctx.kind.as_str() {
    "ts" => SourceType::ts(),
    "tsx" => SourceType::tsx(),
    _ => {
      anyhow::bail!("Got an invalid file type");
    }
  };

  let parse_result = Parser::new(&allocator, &source, source_type).parse();
  if !parse_result.diagnostics.is_empty() {
    // If we're parsing a .ts file, it is common for users to accidentally include JSX.
    // Re-parse in TSX mode to detect this case and emit a clearer error.
    if ctx.kind == "ts" {
      let allocator_tsx = Allocator::default();
      let parse_as_tsx = Parser::new(&allocator_tsx, &source, SourceType::tsx()).parse();
      // To avoid false positives (e.g. angle-bracket type assertions), also require
      // an obvious JSX marker in the source when TS parsing fails but TSX parsing succeeds.
      let looks_like_jsx = source.contains("</") || source.contains("/>") || source.contains("<>");
      if parse_as_tsx.diagnostics.is_empty() && looks_like_jsx {
        anyhow::bail!(
          "JSX syntax detected in a .ts file: {}. Rename the file to .tsx or remove JSX.",
          ctx.path.display()
        );
      }
    }

    let errors: Vec<String> = parse_result
      .diagnostics
      .iter()
      .map(|e| format!("{:?}", e))
      .collect();
    anyhow::bail!("Parse errors: {}", errors.join(", "));
  }
  let mut program = parse_result.program;

  let scoping_result = SemanticBuilder::new().build(&program);
  if !scoping_result.diagnostics.is_empty() {
    let errors: Vec<String> = scoping_result
      .diagnostics
      .iter()
      .map(|e| format!("{:?}", e))
      .collect();
    anyhow::bail!("Parse errors: {}", errors.join(", "));
  }
  let scoping = scoping_result.semantic.into_scoping();

  let mut jsx = if source_type.is_jsx() && !jsx_config.preserve {
    JsxOptions {
      runtime: jsx_config.runtime.unwrap_or(JsxRuntime::Classic),
      development: jsx_config.development,
      import_source: jsx_config.import_source.clone(),
      pragma: jsx_config.pragma.clone(),
      pragma_frag: jsx_config.pragma_frag.clone(),
      ..JsxOptions::default()
    }
  } else {
    JsxOptions::disable()
  };
  jsx.conform();

  let typescript = TypeScriptOptions {
    jsx_pragma: jsx_config
      .pragma
      .clone()
      .unwrap_or_else(|| "React.createElement".to_string())
      .into(),
    jsx_pragma_frag: jsx_config
      .pragma_frag
      .clone()
      .unwrap_or_else(|| "React.Fragment".to_string())
      .into(),
    ..TypeScriptOptions::default()
  };

  let transform_options = TransformOptions {
    typescript,
    jsx,
    ..Default::default()
  };

  let result = Transformer::new(&allocator, &ctx.path, &transform_options);
  let build_result = result.build_with_scoping(scoping, &mut program);
  if !build_result.diagnostics.is_empty() {
    let errors: Vec<String> = build_result
      .diagnostics
      .iter()
      .map(|e| format!("{:?}", e))
      .collect();
    anyhow::bail!("Transform errors: {}", errors.join(", "));
  }

  let generated = Codegen::new()
    .with_options(CodegenOptions {
      minify: false,
      ..CodegenOptions::default()
    })
    .build(&program);

  Ok(TransformerResult {
    code: generated.code,
  })
}
