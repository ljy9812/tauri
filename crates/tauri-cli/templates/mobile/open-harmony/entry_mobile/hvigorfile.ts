import { hapTasks } from '@ohos/hvigor-ohos-plugin';
import { hvigor, HvigorPlugin, HvigorNode, HvigorTask } from '@ohos/hvigor';
import { execFileSync } from 'child_process';
import { resolve } from 'path';

export default {
  system: hapTasks,  /* Built-in plugin of Hvigor. It cannot be modified. */
  plugins:[tauriPlugin()]         /* Custom plugin to extend the functionality of Hvigor. */
}

function tauriPlugin(): HvigorPlugin {
  return {
    pluginId: 'tauri',
    apply(node: HvigorNode) {
      const buildRustCode = () => {
        // When the Tauri CLI drives the build directly (non `--open` paths:
        // `ohos build`, `ohos dev` run, `ohos build --app`), it has already
        // compiled the Rust `.so` itself via `Target::build`. Re-running the
        // cargo build here would (a) duplicate the work and (b) re-enter the
        // CLI via `dev-eco-studio-script`, whose `read_options` expects a live
        // WebSocket parent that may be gone (stale server-addr file → panic /
        // CI hang). `--open` / IDE-direct builds leave this unset so the plugin
        // compiles the `.so` as before.
        if (process.env.TAURI_OHOS_SKIP_DEVECO_SCRIPT) return;
        // Bake this entry module's form so the `.so` compiled via
        // `dev-eco-studio-script` gets the right `cfg(mobile)`/`cfg(desktop)`.
        // Only effective on the `--open`/IDE path (CLI path skips above).
        process.env.OHOS_DEVICE_TYPE = "{{form}}";
        const properties = hvigor.getParameter().getProperties();
        const target = properties.target || "aarch64";
        execFileSync(`{{tauri-binary}}`,
          [{{quote-and-join tauri-binary-args}}, "--target", target.toString()], {
            cwd: resolve(__dirname, "{{root-dir-rel}}"),
            stdio: "inherit",
          });
      }

      node.getTaskByName('default@ConfigureCmake')!.afterRun(buildRustCode);
    }
  }
}
