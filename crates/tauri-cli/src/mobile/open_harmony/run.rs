// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use clap::{ArgAction, Parser};

use crate::{error::Context, ConfigValue, Result};

#[derive(Debug, Clone, Parser)]
#[clap(
  about = "Run your app in production mode on OpenHarmony",
  long_about = "Build, sign, install and run your app on a connected OpenHarmony device. \
    Signing is controlled via environment variables (OHOS_KEYSTORE_FILE, etc.)."
)]
pub struct Options {
  /// Run the app in release mode
  #[clap(short, long)]
  pub release: bool,
  /// List of cargo features to activate
  #[clap(short, long, action = ArgAction::Append, num_args(0..))]
  pub features: Option<Vec<String>>,
  /// JSON strings or paths to JSON, JSON5 or TOML files to merge with the default configuration file
  ///
  /// Configurations are merged in the order they are provided, which means a particular value overwrites previous values when a config key-value pair conflicts.
  ///
  /// Note that a platform-specific file is looked up and merged with the default file by default
  /// (tauri.macos.conf.json, tauri.linux.conf.json, tauri.windows.conf.json, tauri.android.conf.json, tauri.ios.conf.json and tauri.ohos.conf.json)
  /// but you can use this for more specific use cases such as different build flavors.
  #[clap(short, long)]
  pub config: Vec<ConfigValue>,
  /// Open DevEco Studio instead of running on a connected device
  #[clap(short, long)]
  pub open: bool,
  /// Runs on the given device name
  pub device: Option<String>,
  /// Command line arguments passed to the runner.
  /// Use `--` to explicitly mark the start of the arguments.
  /// e.g. `tauri ohos run -- [runnerArgs]`.
  #[clap(last(true))]
  pub args: Vec<String>,
  /// Do not error out if a version mismatch is detected on a Tauri package.
  #[clap(long)]
  pub ignore_version_mismatches: bool,
  /// Device type to build for (mobile or desktop)
  #[clap(long, default_value = "mobile", value_parser(["mobile", "desktop"]))]
  pub device_type: String,
}

pub fn command(options: Options, noise_level: cargo_mobile2::opts::NoiseLevel) -> Result<()> {
  // Step 1: Build + sign (delegates to the build command)
  let build_options = super::build::Options {
    debug: !options.release,
    targets: None,
    features: options.features.clone(),
    config: options.config.clone(),
    open: options.open,
    ci: false,
    device_type: options.device_type.clone(),
    app: false,
    args: options.args.clone(),
    ignore_version_mismatches: options.ignore_version_mismatches,
  };

  super::build::command(build_options, noise_level)?;

  // If --open, build command already opened DevEco Studio
  if options.open {
    return Ok(());
  }

  // Step 2: Install + launch + stream logs
  install_launch_and_log(&options)?;

  Ok(())
}

/// Detect device, install the signed HAP, and launch the app.
fn install_launch_and_log(options: &Options) -> Result<()> {
  use super::{device_prompt, env};
  use cargo_mobile2::open_harmony::{hap, hdc};

  let env = env()?;

  // Detect device
  let device = device_prompt(&env, options.device.as_deref())?;
  let device_id = device.id().to_string();

  // Resolve tauri config to get the OpenHarmony project config
  let dirs = crate::helpers::app_paths::resolve_dirs();
  let tauri_config = crate::helpers::config::get_config(
    tauri_utils::platform::Target::OpenHarmony,
    &options
      .config
      .iter()
      .map(|conf| &conf.0)
      .collect::<Vec<_>>(),
    dirs.tauri,
  )?;

  let interface =
    crate::interface::AppInterface::new(&tauri_config, None, dirs.tauri)?;
  let app = super::get_app(
    super::MobileTarget::OpenHarmony,
    &tauri_config,
    &interface,
    dirs.tauri,
  );
  let (config, _metadata) = super::get_config(
    &app,
    &tauri_config,
    None,
    &crate::mobile::CliOptions::default(),
  );

  let bundle_name = config.app().identifier().to_string();

  // Find the signed HAP (the build command already produced it).
  // Prefer signed, fall back to first available.
  let hap_paths = hap::haps_paths(&config);
  let hap_to_install = hap_paths
    .iter()
    .find(|p| {
      p.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .contains("-signed")
    })
    .map(|p| p.clone())
    .or_else(|| hap_paths.first().cloned())
    .context("no HAP file available to install — run `tauri ohos build` first")?;

  log::info!(
    "Installing {} to device {}",
    hap_to_install.display(),
    device.name()
  );

  // === Install: hdc -t <id> install -r <hap_path> ===
  let hap_str = hap_to_install.to_string_lossy().to_string();
  let install_result = hdc::hdc(
    &env,
    vec![
      "-t".to_string(),
      device_id.clone(),
      "install".to_string(),
      "-r".to_string(),
      hap_str.clone(),
    ],
  )
  .stdout_capture()
  .stderr_capture()
  .run()
  .context("failed to run hdc install")?;

  if !install_result.status.success() {
    // Fallback: uninstall + reinstall
    log::warn!("Install failed, trying uninstall first...");
    let _ = hdc::hdc(
      &env,
      [
        "-t",
        &device_id,
        "shell",
        "bm",
        "uninstall",
        "-n",
        &bundle_name,
      ],
    )
    .stdout_capture()
    .stderr_capture()
    .run();

    let retry_result = hdc::hdc(
      &env,
      vec![
        "-t".to_string(),
        device_id.clone(),
        "install".to_string(),
        "-r".to_string(),
        hap_str,
      ],
    )
    .stdout_capture()
    .stderr_capture()
    .run()
    .context("failed to run hdc install (retry)")?;

    if !retry_result.status.success() {
      let stdout = String::from_utf8_lossy(&retry_result.stdout);
      let stderr = String::from_utf8_lossy(&retry_result.stderr);
      if !stdout.is_empty() {
        log::error!("hdc stdout:\n{stdout}");
      }
      if !stderr.is_empty() {
        log::error!("hdc stderr:\n{stderr}");
      }
      crate::error::bail!("Failed to install HAP to device");
    }
  }

  log::info!("HAP installed successfully");

  // === Launch: hdc -t <id> shell aa start -b <bundleName> -a EntryAbility ===
  log::info!("Launching app: {bundle_name}");

  let launch_result = hdc::hdc(
    &env,
    [
      "-t",
      &device_id,
      "shell",
      "aa",
      "start",
      "-b",
      &bundle_name,
      "-a",
      "EntryAbility",
    ],
  )
  .stdout_capture()
  .stderr_capture()
  .run()
  .context("failed to run hdc launch")?;

  if !launch_result.status.success() {
    let stdout = String::from_utf8_lossy(&launch_result.stdout);
    let stderr = String::from_utf8_lossy(&launch_result.stderr);
    if !stdout.is_empty() {
      log::error!("hdc stdout:\n{stdout}");
    }
    if !stderr.is_empty() {
      log::error!("hdc stderr:\n{stderr}");
    }
    log::warn!("Failed to launch app on device");
    return Ok(());
  }

  log::info!("App launched successfully");

  Ok(())
}
