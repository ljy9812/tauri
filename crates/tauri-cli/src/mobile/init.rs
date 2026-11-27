// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{get_app, Target};
use crate::{
  helpers::app_paths::Dirs,
  helpers::{
    config::{get_config as get_tauri_config, Color, Config},
    template::JsonMap,
  },
  interface::AppInterface,
  ConfigValue, Result,
};
use cargo_mobile2::{
  config::app::App,
  reserved_names::KOTLIN_ONLY_KEYWORDS,
  util::{
    self,
    cli::{Report, TextWrapper},
  },
};
use handlebars::{
  Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderError, RenderErrorReason,
};

use std::{env::var_os, path::PathBuf};

pub fn command(
  target: Target,
  ci: bool,
  reinstall_deps: bool,
  skip_targets_install: bool,
  config: Vec<ConfigValue>,
) -> Result<()> {
  let dirs = crate::helpers::app_paths::resolve_dirs();
  let wrapper = TextWrapper::default();

  exec(
    target,
    &wrapper,
    ci,
    reinstall_deps,
    skip_targets_install,
    config,
    dirs,
  )?;
  Ok(())
}

fn exec(
  target: Target,
  wrapper: &TextWrapper,
  #[allow(unused_variables)] non_interactive: bool,
  #[allow(unused_variables)] reinstall_deps: bool,
  skip_targets_install: bool,
  config: Vec<ConfigValue>,
  dirs: Dirs,
) -> Result<App> {
  let tauri_config = get_tauri_config(
    target.platform_target(),
    &config.iter().map(|conf| &conf.0).collect::<Vec<_>>(),
    dirs.tauri,
  )?;

  let app = get_app(
    target,
    &tauri_config,
    &AppInterface::new(&tauri_config, None, dirs.tauri)?,
    dirs.tauri,
  );

  let (handlebars, mut map) = handlebars(&app);

  let mut args = std::env::args_os();

  let (binary, mut build_args) = args
    .next()
    .map(|bin| {
      let bin_path = PathBuf::from(&bin);
      let mut build_args = vec!["tauri"];

      if let Some(bin_stem) = bin_path.file_stem() {
        let r = regex::Regex::new("(nodejs|node)\\-?([1-9]*)*$").unwrap();
        if r.is_match(&bin_stem.to_string_lossy()) {
          if var_os("PNPM_PACKAGE_NAME").is_some() {
            return ("pnpm".into(), build_args);
          } else if is_pnpm_dlx() {
            return ("pnpm".into(), vec!["dlx", "@tauri-apps/cli"]);
          } else if let Some(npm_execpath) = var_os("npm_execpath") {
            let manager_stem = PathBuf::from(&npm_execpath)
              .file_stem()
              .unwrap()
              .to_os_string();
            let is_npm = manager_stem == "npm-cli";
            let binary = if is_npm {
              "npm".into()
            } else if manager_stem == "npx-cli" {
              "npx".into()
            } else {
              manager_stem
            };

            if is_npm {
              build_args.insert(0, "run");
              build_args.insert(1, "--");
            }

            return (binary, build_args);
          }
        } else if bin_stem == "deno" {
          build_args.insert(0, "task");
          return (std::ffi::OsString::from("deno"), build_args);
        } else if !cfg!(debug_assertions) && bin_stem == "cargo-tauri" {
          return (std::ffi::OsString::from("cargo"), build_args);
        }
      }

      (bin, build_args)
    })
    .unwrap_or_else(|| (std::ffi::OsString::from("cargo"), vec!["tauri"]));

  build_args.push(target.command_name());
  build_args.push(target.ide_build_script_name());

  let mut binary = binary.to_string_lossy().to_string();
  if binary.ends_with(".exe") || binary.ends_with(".cmd") || binary.ends_with(".bat") {
    // remove Windows-only extension
    binary.pop();
    binary.pop();
    binary.pop();
    binary.pop();
  }

  // OHOS templates need forward slashes (hvigor/ArkTS tooling); other targets
  // keep the upstream raw value.
  let tauri_binary = if matches!(target, Target::OpenHarmony) {
    dunce::simplified(std::path::Path::new(&binary))
      .to_string_lossy()
      .replace('\\', "/")
  } else {
    binary.clone()
  };
  map.insert("tauri-binary", tauri_binary);
  map.insert("tauri-binary-args", &build_args);
  map.insert("tauri-binary-args-str", build_args.join(" "));

  let app = match target {
    // Generate Android Studio project
    Target::Android => {
      let _env = super::android::env(non_interactive)?;
      let (config, metadata) =
        super::android::get_config(&app, &tauri_config, &[], &Default::default());
      map.insert("android", &config);
      super::android::project::gen(
        &config,
        &metadata,
        (handlebars, map),
        wrapper,
        skip_targets_install,
      )?;
      app
    }
    #[cfg(target_os = "macos")]
    // Generate Xcode project
    Target::Ios => {
      let (config, metadata) =
        super::ios::get_config(&app, &tauri_config, &[], &Default::default(), dirs.tauri)?;
      map.insert("apple", &config);
      super::ios::project::gen(
        &tauri_config,
        &config,
        &metadata,
        (handlebars, map),
        wrapper,
        non_interactive,
        reinstall_deps,
        skip_targets_install,
      )?;
      app
    }
    Target::OpenHarmony => {
      let (config, _metadata) =
        super::open_harmony::get_config(&app, &tauri_config, None, &Default::default());

      // Register template variables for OHOS templates
      let version = tauri_config.version.as_deref().unwrap_or("1.0.0");
      let publisher = tauri_config.bundle.publisher.clone().unwrap_or_else(|| {
        // Fall back to the second segment of the identifier (e.g. "com.tauri.api" -> "tauri");
        // if the identifier has no dot, fall back to the product name instead of an empty string.
        tauri_config
          .identifier
          .split('.')
          .nth(1)
          .map(|s| s.to_string())
          .filter(|s| !s.is_empty())
          .unwrap_or_else(|| tauri_config.product_name.clone().unwrap_or_default())
      });
      let short_description = tauri_config
        .bundle
        .short_description
        .clone()
        .unwrap_or_else(|| tauri_config.product_name.clone().unwrap_or_default());
      let version_code = tauri_config
        .bundle
        .open_harmony
        .version_code
        .unwrap_or_else(|| {
          tauri_config
            .version
            .as_ref()
            .and_then(|v| semver::Version::parse(v).ok())
            .map(|v| {
              let code = v.major * 1000000 + v.minor * 1000 + v.patch;
              code.clamp(1, u32::MAX as u64) as u32
            })
            .unwrap_or(1)
        });
      map.insert("version", version);

      // bundle sub-fields: {{bundle.publisher}}, {{bundle.short-description}}, {{bundle.open-harmony.*}}
      let mut bundle_map = serde_json::Map::new();
      bundle_map.insert(
        "publisher".to_string(),
        serde_json::to_value(&publisher).unwrap_or_default(),
      );
      bundle_map.insert(
        "short-description".to_string(),
        serde_json::to_value(&short_description).unwrap_or_default(),
      );

      let mut ohos_map = serde_json::Map::new();
      ohos_map.insert(
        "version-code".to_string(),
        serde_json::to_value(version_code).unwrap_or_default(),
      );
      bundle_map.insert(
        "open-harmony".to_string(),
        serde_json::Value::Object(ohos_map),
      );

      // Per-form deviceTypes for the dual-entry template: each entry's module.json5
      // emits its list via `{{{form-device-types}}}` (pre-serialized JSON string so
      // triple-brace renders it verbatim; a Value::Array would render unquoted).
      let (mobile_device_types_str, desktop_device_types_str) = {
        let conf = &tauri_config.bundle.open_harmony.device_types;
        let m = super::open_harmony::device_types_for_form(conf, "mobile");
        let d = super::open_harmony::device_types_for_form(conf, "desktop");
        (
          serde_json::to_string(&m).unwrap_or_else(|_| r#"["phone","tablet"]"#.into()),
          serde_json::to_string(&d).unwrap_or_else(|_| r#"["2in1"]"#.into()),
        )
      };

      map.insert("bundle", bundle_map);

      // First-frame window background from tauri.conf's first `app.windows`
      // background_color (issue Eulogizethesun/tauri#224 / C96) - see
      // window_background_color() below.
      let (background_color_set, background_color) = window_background_color(&tauri_config);
      map.insert("window-background-color-set", background_color_set);
      map.insert("window-background-color", background_color);
      // System-bar icon/text color derived from the same background's
      // luminance (issue Eulogizethesun/tauri#224 / C96) - see
      // system_bar_content_color() below. The bar backgrounds stay
      // transparent; only the content color is derived.
      map.insert(
        "system-bar-content-color",
        system_bar_content_color(&tauri_config),
      );

      // Entry module.json5 requestPermissions, PER FORM (review tauri#262
      // item 5): a configured form's list replaces that form's scaffold
      // baseline wholesale (issue Eulogizethesun/tauri#220) — pre-serialized
      // JSON like `form-device-types` so the triple-brace renders it
      // verbatim; unset config (whole field, or that form's key) leaves that
      // form's baseline untouched. Injected per form in the entry render
      // loop (project.rs), not into the global map.
      let (mobile_request_permissions, desktop_request_permissions) =
        entry_request_permissions(&tauri_config)?;

      let detected_plugins =
        super::open_harmony::plugins::detect_all_plugins(dirs.tauri).unwrap_or_default();

      let plugin_metadata: Vec<super::open_harmony::plugins::PluginMeta> =
        if detected_plugins.is_empty() {
          vec![]
        } else {
          detected_plugins
            .iter()
            .filter_map(|d| {
              match super::open_harmony::plugins::parse_plugin_meta(&d.har_path, &d.name) {
                Ok(meta) => {
                  if super::open_harmony::plugins::validate_plugin_meta(&meta).is_ok() {
                    Some(meta)
                  } else {
                    log::warn!("Skipping plugin '{}': invalid metadata", d.name);
                    None
                  }
                }
                Err(e) => {
                  log::warn!("Skipping plugin '{}': {}", d.name, e);
                  None
                }
              }
            })
            .collect()
        };

      super::open_harmony::project::gen_with_plugins(
        &app,
        &config,
        (handlebars, map),
        skip_targets_install,
        plugin_metadata.clone(),
        &mobile_device_types_str,
        &desktop_device_types_str,
        mobile_request_permissions.as_deref(),
        desktop_request_permissions.as_deref(),
      )?;

      if !plugin_metadata.is_empty() {
        let project_dir = config.project_dir();
        for plugin in &plugin_metadata {
          super::open_harmony::plugins::copy_plugin_har(plugin, &project_dir)?;
        }
        super::open_harmony::plugins::update_plugin_configs(&project_dir, &plugin_metadata)?;
      }

      app
    }
  };

  Report::victory(
    "Project generated successfully!",
    "Make cool apps! 🌻 🐕 🎉",
  )
  .print(wrapper);
  Ok(app)
}

/// First window's configured `background_color` (RGBA), the shared input for
/// both the first-frame window background and the system-bar content color
/// derivations (issue Eulogizethesun/tauri#224 / C96).
fn first_window_background_color(tauri_config: &Config) -> Option<Color> {
  tauri_config
    .app
    .windows
    .first()
    .and_then(|w| w.background_color)
}

/// Renders tauri.conf's first `app.windows` `background_color` as the OHOS
/// `#AARRGGBB` value of the `window-background-color` template variable: the
/// EntryAbility.ets.hbs templates apply it to the window layer in
/// onWindowStageCreate, so the first frame matches the configured background
/// instead of the system default until the webview layer paints - wry applies
/// `background_color` only at webview creation, after the first frame (issue
/// Eulogizethesun/tauri#224 / C96). Byte-identical rendering to wry's
/// `rgba_to_argb_hex`. Returns `(false, "")` when no window (or no
/// `background_color`) is configured: the template guards the background
/// calls with `{{#if window-background-color-set}}` (a separate boolean -
/// handlebars treats an empty string as truthy), because an unconditional
/// setWindowBackgroundColor (even with a white default) pins the title-bar
/// background and breaks dark mode. Alpha is passed through unclamped -
/// window-layer alpha semantics are the OS's, same as the webview layer's.
fn window_background_color(tauri_config: &Config) -> (bool, String) {
  first_window_background_color(tauri_config)
    .map(|c| {
      (
        true,
        format!("#{:02X}{:02X}{:02X}{:02X}", c.3, c.0, c.1, c.2),
      )
    })
    .unwrap_or_else(|| (false, String::new()))
}

/// Perceptual luma of an RGBA color in `[0, 1]` (ITU-R BT.601 coefficients).
/// Alpha is ignored: the derivation targets the opaque page background the
/// bars sit on, and a translucent configured background has no single
/// underlying color to composite over.
fn srgb_luma(color: Color) -> f64 {
  (0.299 * color.0 as f64 + 0.587 * color.1 as f64 + 0.114 * color.2 as f64) / 255.0
}

/// Derives the `system-bar-content-color` template variable: the
/// EntryAbility.ets.hbs templates keep the status/navigation bar backgrounds
/// transparent (the window background and theme show through) and set only
/// the bar icon/text color, derived from tauri.conf's first `app.windows`
/// `background_color` so it stays readable on the page background (issue
/// Eulogizethesun/tauri#224 / C96): bright background -> dark content
/// (`#FF000000`), dark background -> light content (`#FFFFFFFF`). When no
/// background is configured the light-theme default (dark content,
/// `#FF000000`) is used - matching the theme-following bar behavior of the
/// unset case instead of guessing a theme we don't read.
///
/// Retained gap (pre-existing, disclosed not fixed): this is a
/// generation-time value baked into the template, and the runtime theme
/// chain has no SystemBarProperties update, so the bar content color does
/// not follow OHOS dark mode / `setTheme` switches. With no
/// `background_color` configured, the dark-content default can end up on a
/// dark-mode bar background (the transparent bars let the dark theme show
/// through) with reduced contrast until the app sets the color itself.
fn system_bar_content_color(tauri_config: &Config) -> String {
  let bright = first_window_background_color(tauri_config)
    .map(|c| srgb_luma(c) > 0.5)
    .unwrap_or(true);
  if bright {
    "#FF000000".into()
  } else {
    "#FFFFFFFF".into()
  }
}

/// Renders the configured `bundle.openHarmony.requestPermissions` PER FORM
/// (review tauri#262 item 5) as the pre-serialized JSON strings of the
/// `form-request-permissions` template variable: `(mobile, desktop)`, each
/// `Some(validated JSON)` only when that form is configured, `None` when the
/// field — or that form's key — is unset (the entry module.json5 templates
/// keep that form's scaffold baseline, issue Eulogizethesun/tauri#220). Each
/// form's list is validated and normalized to the OS object elements by the
/// same helper the build-time rewrite uses.
fn entry_request_permissions(
  tauri_config: &Config,
) -> crate::Result<(Option<String>, Option<String>)> {
  let per_form = tauri_config
    .bundle
    .open_harmony
    .request_permissions
    .as_ref();
  let render = |list: Option<&[tauri_utils::config::OhosRequestPermission]>| match list {
    Some(permissions) => {
      super::open_harmony::plugins::request_permissions_json(permissions).map(Some)
    }
    None => Ok(None),
  };
  Ok((
    render(per_form.and_then(|p| p.mobile.as_deref()))?,
    render(per_form.and_then(|p| p.desktop.as_deref()))?,
  ))
}

pub(crate) fn handlebars(app: &App) -> (Handlebars<'static>, JsonMap) {
  let mut h = Handlebars::new();
  h.register_escape_fn(handlebars::no_escape);

  h.register_helper("html-escape", Box::new(html_escape));
  h.register_helper("join", Box::new(join));
  h.register_helper("quote-and-join", Box::new(quote_and_join));
  h.register_helper(
    "quote-and-join-colon-prefix",
    Box::new(quote_and_join_colon_prefix),
  );
  h.register_helper("snake-case", Box::new(snake_case));
  h.register_helper("escape-kotlin-keyword", Box::new(escape_kotlin_keyword));
  // don't mix these up or very bad things will happen to all of us
  h.register_helper("prefix-path", Box::new(prefix_path));
  h.register_helper("unprefix-path", Box::new(unprefix_path));

  let mut map = JsonMap::default();
  map.insert("app", app);

  (h, map)
}

fn get_str<'a>(helper: &'a Helper) -> &'a str {
  helper
    .param(0)
    .and_then(|v| v.value().as_str())
    .unwrap_or("")
}

fn get_str_array(helper: &Helper, formatter: impl Fn(&str) -> String) -> Option<Vec<String>> {
  helper.param(0).and_then(|v| {
    v.value()
      .as_array()
      .and_then(|arr| arr.iter().map(|val| val.as_str().map(&formatter)).collect())
  })
}

fn html_escape(
  helper: &Helper,
  _: &Handlebars,
  _ctx: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  out
    .write(&handlebars::html_escape(get_str(helper)))
    .map_err(Into::into)
}

fn join(
  helper: &Helper,
  _: &Handlebars,
  _: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  out
    .write(
      &get_str_array(helper, |s| s.to_string())
        .ok_or_else(|| {
          RenderErrorReason::ParamTypeMismatchForName("join", "0".to_owned(), "array".to_owned())
        })?
        .join(", "),
    )
    .map_err(Into::into)
}

fn quote_and_join(
  helper: &Helper,
  _: &Handlebars,
  _: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  out
    .write(
      &get_str_array(helper, |s| format!("{s:?}"))
        .ok_or_else(|| {
          RenderErrorReason::ParamTypeMismatchForName(
            "quote-and-join",
            "0".to_owned(),
            "array".to_owned(),
          )
        })?
        .join(", "),
    )
    .map_err(Into::into)
}

fn quote_and_join_colon_prefix(
  helper: &Helper,
  _: &Handlebars,
  _: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  out
    .write(
      &get_str_array(helper, |s| format!("{:?}", format!(":{s}")))
        .ok_or_else(|| {
          RenderErrorReason::ParamTypeMismatchForName(
            "quote-and-join-colon-prefix",
            "0".to_owned(),
            "array".to_owned(),
          )
        })?
        .join(", "),
    )
    .map_err(Into::into)
}

fn snake_case(
  helper: &Helper,
  _: &Handlebars,
  _: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  use heck::ToSnekCase as _;
  out
    .write(&get_str(helper).to_snek_case())
    .map_err(Into::into)
}

fn escape_kotlin_keyword(
  helper: &Helper,
  _: &Handlebars,
  _: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  let escaped_result = get_str(helper)
    .split('.')
    .map(|s| {
      if KOTLIN_ONLY_KEYWORDS.contains(&s) {
        format!("`{s}`")
      } else {
        s.to_string()
      }
    })
    .collect::<Vec<_>>()
    .join(".");

  out.write(&escaped_result).map_err(Into::into)
}

fn app_root(ctx: &Context) -> std::result::Result<&str, RenderError> {
  let app_root = ctx
    .data()
    .get("app")
    .ok_or_else(|| RenderErrorReason::Other("`app` missing from template data.".to_owned()))?
    .get("root-dir")
    .ok_or_else(|| {
      RenderErrorReason::Other("`app.root-dir` missing from template data.".to_owned())
    })?;
  app_root.as_str().ok_or_else(|| {
    RenderErrorReason::Other("`app.root-dir` contained invalid UTF-8.".to_owned()).into()
  })
}

fn prefix_path(
  helper: &Helper,
  _: &Handlebars,
  ctx: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  out
    .write(
      util::prefix_path(app_root(ctx)?, get_str(helper))
        .to_str()
        .ok_or_else(|| {
          RenderErrorReason::Other(
            "Either the `app.root-dir` or the specified path contained invalid UTF-8.".to_owned(),
          )
        })?,
    )
    .map_err(Into::into)
}

fn unprefix_path(
  helper: &Helper,
  _: &Handlebars,
  ctx: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  out
    .write(
      util::unprefix_path(app_root(ctx)?, get_str(helper))
        .map_err(|_| {
          RenderErrorReason::Other(
            "Attempted to unprefix a path that wasn't in the app root dir.".to_owned(),
          )
        })?
        .to_str()
        .ok_or_else(|| {
          RenderErrorReason::Other(
            "Either the `app.root-dir` or the specified path contained invalid UTF-8.".to_owned(),
          )
        })?,
    )
    .map_err(Into::into)
}

fn is_pnpm_dlx() -> bool {
  var_os("NODE_PATH")
    .map(PathBuf::from)
    .is_some_and(|node_path| {
      let mut iter = node_path.components().peekable();
      while let Some(c) = iter.next() {
        if c.as_os_str() == "pnpm" && iter.peek().is_some_and(|c| c.as_os_str() == "dlx") {
          return true;
        }
      }
      false
    })
}

#[cfg(test)]
mod tests {
  use super::*;

  fn config_from(json: &str) -> Config {
    serde_json::from_str(json).expect("valid test config")
  }

  #[test]
  fn window_background_color_renders_argb_from_the_first_window() {
    // Object form: {red, green, blue, alpha} -> #AARRGGBB (alpha first, the
    // same byte order wry's rgba_to_argb_hex produces).
    let config = config_from(
      r##"{"identifier":"com.example.test","app":{"windows":[
        {"label":"first","backgroundColor":{"red":16,"green":32,"blue":48,"alpha":255}},
        {"label":"second","backgroundColor":{"red":1,"green":2,"blue":3,"alpha":4}}
      ]}}"##,
    );
    let (set, color) = window_background_color(&config);
    assert!(set);
    assert_eq!(color, "#FF102030");
  }

  #[test]
  fn window_background_color_accepts_the_string_and_rgb_forms() {
    // tauri.conf string form is #RRGGBB(AA) - alpha LAST. The helper
    // re-orders to OHOS #AARRGGBB (alpha first), same as wry's
    // rgba_to_argb_hex.
    let hex = config_from(
      r##"{"identifier":"com.example.test","app":{"windows":[{"label":"main","backgroundColor":"#102030"}]}}"##,
    );
    let (hex_set, hex) = window_background_color(&hex);
    assert!(hex_set);
    assert_eq!(hex, "#FF102030");
    let rgba = config_from(
      r##"{"identifier":"com.example.test","app":{"windows":[{"label":"main","backgroundColor":"#10203080"}]}}"##,
    );
    let (rgba_set, rgba_color) = window_background_color(&rgba);
    assert!(rgba_set);
    assert_eq!(rgba_color, "#80102030");
  }

  #[test]
  fn window_background_color_is_empty_when_unset() {
    // Window without background_color, and no windows at all: unset means
    // "not configured" - the template skips the background calls entirely.
    let unconfigured =
      config_from(r##"{"identifier":"com.example.test","app":{"windows":[{"label":"main"}]}}"##);
    assert_eq!(
      window_background_color(&unconfigured),
      (false, String::new())
    );
    let no_windows = config_from(r##"{"identifier":"com.example.test","app":{}}"##);
    assert_eq!(window_background_color(&no_windows), (false, String::new()));
  }

  #[test]
  fn ohos_entry_ability_templates_render_window_background_color() {
    // End-to-end with the real template files: the dashed top-level key
    // must survive handlebars' path grammar and land inside the quoted
    // ArkTS arguments. Both form variants render the same scaffold (the two
    // files are kept byte-identical on purpose).
    let tpls = [
      (
        "mobile",
        include_str!(
          "../../templates/mobile/open-harmony/entry_mobile/src/main/ets/entryability/EntryAbility.ets.hbs"
        ),
      ),
      (
        "desktop",
        include_str!(
          "../../templates/mobile/open-harmony/entry_desktop/src/main/ets/entryability/EntryAbility.ets.hbs"
        ),
      ),
    ];
    for (form, tpl) in tpls {
      let mut h = handlebars::Handlebars::new();
      h.register_escape_fn(handlebars::no_escape);
      h.register_template_string("entry", tpl).unwrap();
      let data = serde_json::json!({"window-background-color-set": true, "window-background-color": "#FF102030", "system-bar-content-color": "#FF000000"});
      let out = h.render("entry", &data).unwrap();
      assert!(
        out.contains("setWindowBackgroundColor('#FF102030')"),
        "{form}: background call not rendered as expected"
      );
      assert!(
        out.contains("setWindowContainerColor('#FF102030', '#FF102030')"),
        "{form}: container color call not rendered as expected"
      );
      assert!(
        out.contains("statusBarContentColor: '#FF000000'")
          && out.contains("navigationBarContentColor: '#FF000000'"),
        "{form}: derived system-bar content color not rendered"
      );
      // The bar backgrounds stay transparent regardless of the derivation -
      // only the content color flips.
      assert!(
        out.contains("statusBarColor: '#00000000'")
          && out.contains("navigationBarColor: '#00000000'"),
        "{form}: transparent bar backgrounds must be kept"
      );
      // Unset background: the background calls are skipped entirely (an
      // unconditional call pins the title bar and breaks dark mode), while
      // the system-bar setup must survive the skip.
      let unset = h
        .render(
          "entry",
          &serde_json::json!({"window-background-color-set": false, "window-background-color": "", "system-bar-content-color": "#FFFFFFFF"}),
        )
        .unwrap();
      assert!(
        !unset.contains("setWindowBackgroundColor('"),
        "{form}: background call must be skipped when unset"
      );
      assert!(
        !unset.contains("setWindowContainerColor('"),
        "{form}: container color call must be skipped when unset"
      );
      assert!(
        unset.contains("setWindowSystemBarProperties"),
        "{form}: system bar setup must survive the background skip"
      );
    }
  }

  #[test]
  fn system_bar_content_color_is_dark_on_a_bright_background() {
    let white = config_from(
      r##"{"identifier":"com.example.test","app":{"windows":[{"label":"main","backgroundColor":"#FFFFFF"}]}}"##,
    );
    assert_eq!(system_bar_content_color(&white), "#FF000000");
    // The OS light-theme default tone: still bright enough for dark content.
    let light = config_from(
      r##"{"identifier":"com.example.test","app":{"windows":[{"label":"main","backgroundColor":{"red":240,"green":240,"blue":240,"alpha":255}}]}}"##,
    );
    assert_eq!(system_bar_content_color(&light), "#FF000000");
  }

  #[test]
  fn system_bar_content_color_is_light_on_a_dark_background() {
    // The OS dark-theme default tone.
    let dark = config_from(
      r##"{"identifier":"com.example.test","app":{"windows":[{"label":"main","backgroundColor":"#1A1A1A"}]}}"##,
    );
    assert_eq!(system_bar_content_color(&dark), "#FFFFFFFF");
    // The scaffold's historical hardcoded black: dark enough for light
    // content (the old template's #FF000000 content on a black page was
    // invisible - that contrast bug is what the derivation fixes).
    let black = config_from(
      r##"{"identifier":"com.example.test","app":{"windows":[{"label":"main","backgroundColor":{"red":0,"green":0,"blue":0,"alpha":255}}]}}"##,
    );
    assert_eq!(system_bar_content_color(&black), "#FFFFFFFF");
  }

  #[test]
  fn system_bar_content_color_defaults_to_dark_content_when_unset() {
    // Unconfigured: keep the light-theme default (dark content) - the
    // template's pre-#224 behavior - instead of guessing a theme we don't
    // read.
    let unconfigured =
      config_from(r##"{"identifier":"com.example.test","app":{"windows":[{"label":"main"}]}}"##);
    assert_eq!(system_bar_content_color(&unconfigured), "#FF000000");
    let no_windows = config_from(r##"{"identifier":"com.example.test","app":{}}"##);
    assert_eq!(system_bar_content_color(&no_windows), "#FF000000");
  }

  #[test]
  fn entry_request_permissions_serializes_the_configured_list() {
    // Issue Eulogizethesun/tauri#220: the configured list is pre-serialized
    // (compact JSON, same shape the OS expects) for the template's
    // triple-brace. Per-form (review tauri#262 item 5): each configured form
    // returns its own list; the unconfigured form returns None.
    let config = config_from(
      r##"{"identifier":"com.example.test","bundle":{"openHarmony":{"requestPermissions":{
        "mobile":[
          "ohos.permission.INTERNET",
          {"name":"ohos.permission.READ_PASTEBOARD","reason":"$string:reason_clipboard","usedScene":{"abilities":["EntryAbility"],"when":"inuse"}}
        ],
        "desktop":["ohos.permission.INTERNET"]
      }}}}"##,
    );
    let (mobile, desktop) = entry_request_permissions(&config).unwrap();
    assert_eq!(
      mobile.as_deref(),
      Some(
        r#"[{"name":"ohos.permission.INTERNET"},{"name":"ohos.permission.READ_PASTEBOARD","reason":"$string:reason_clipboard","usedScene":{"abilities":["EntryAbility"],"when":"inuse"}}]"#
      )
    );
    assert_eq!(
      desktop.as_deref(),
      Some(r#"[{"name":"ohos.permission.INTERNET"}]"#)
    );
  }

  #[test]
  fn entry_request_permissions_fails_fast_on_non_ohos_names() {
    let config = config_from(
      r##"{"identifier":"com.example.test","bundle":{"openHarmony":{"requestPermissions":{"desktop":["android.permission.INTERNET"]}}}}"##,
    );
    let err = entry_request_permissions(&config).unwrap_err().to_string();
    assert!(
      err.contains("android.permission.INTERNET"),
      "error must name the offending value: {err}"
    );
  }

  #[test]
  fn entry_request_permissions_is_unset_when_not_configured() {
    // Unset: the scaffold baseline stays - the template keeps its own block.
    let config = config_from(r##"{"identifier":"com.example.test"}"##);
    assert_eq!(
      entry_request_permissions(&config).unwrap(),
      (None::<String>, None::<String>)
    );
    // The field set but this form's key unset: that form keeps its baseline
    // too (review tauri#262 item 5 - per-form replace, per-form keep).
    let config = config_from(
      r##"{"identifier":"com.example.test","bundle":{"openHarmony":{"requestPermissions":{"desktop":["ohos.permission.INTERNET"]}}}}"##,
    );
    let (mobile, desktop) = entry_request_permissions(&config).unwrap();
    assert_eq!(
      mobile, None,
      "an unconfigured mobile form must keep its scaffold baseline"
    );
    assert!(desktop.is_some());
  }

  #[test]
  fn ohos_entry_module_json5_templates_render_request_permissions() {
    // End-to-end with the real template files: both render forms (configured
    // list / unset baseline) must produce JSON5 the build's module.json5
    // rewrite can parse. The two files differ outside this block on purpose
    // (abilityStage/launchType), so each is rendered with its own form name.
    let tpls = [
      (
        "mobile",
        include_str!("../../templates/mobile/open-harmony/entry_mobile/src/main/module.json5"),
      ),
      (
        "desktop",
        include_str!("../../templates/mobile/open-harmony/entry_desktop/src/main/module.json5"),
      ),
    ];
    for (form, tpl) in tpls {
      let mut h = handlebars::Handlebars::new();
      h.register_escape_fn(handlebars::no_escape);
      h.register_template_string("module", tpl).unwrap();

      // Configured: the list replaces the scaffold baseline wholesale.
      let configured = h
        .render(
          "module",
          &serde_json::json!({
            "form": form,
            "form-device-types": r#"["phone","tablet"]"#,
            "form-request-permissions-set": true,
            "form-request-permissions": r#"[{"name":"ohos.permission.INTERNET"},{"name":"ohos.permission.READ_PASTEBOARD","reason":"$string:reason_clipboard","usedScene":{"abilities":["EntryAbility"],"when":"inuse"}}]"#,
          }),
        )
        .unwrap();
      let doc: serde_json::Value =
        json5::from_str(&configured).unwrap_or_else(|e| panic!("{form}: {e}: {configured}"));
      let list = doc["module"]["requestPermissions"].as_array().unwrap();
      assert_eq!(
        list.len(),
        2,
        "{form}: baseline must be replaced: {configured}"
      );
      assert_eq!(list[0]["name"], "ohos.permission.INTERNET");
      assert_eq!(list[1]["usedScene"]["when"], "inuse");
      assert!(
        !configured.contains("PUBLISH_AGENT_REMINDER"),
        "{form}: baseline entries must not survive the configured render"
      );

      // Unset: the baseline survives, including the desktop-only conditional.
      let unset = h
        .render(
          "module",
          &serde_json::json!({
            "form": form,
            "form-device-types": r#"["phone","tablet"]"#,
            "form-request-permissions-set": false,
            "form-request-permissions": "",
          }),
        )
        .unwrap();
      let doc: serde_json::Value =
        json5::from_str(&unset).unwrap_or_else(|e| panic!("{form}: {e}: {unset}"));
      let list = doc["module"]["requestPermissions"].as_array().unwrap();
      assert!(
        list
          .iter()
          .any(|p| p["name"] == "ohos.permission.PUBLISH_AGENT_REMINDER"),
        "{form}: baseline entries must survive the unset render: {unset}"
      );
      let has_print = list.iter().any(|p| p["name"] == "ohos.permission.PRINT");
      assert_eq!(has_print, form == "desktop", "{form}: desktop conditional");
    }
  }
}
