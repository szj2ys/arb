use anyhow::{anyhow, bail, Context};
use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Parser, Clone, Default)]
pub struct ConfigCommand {
    /// Ensure ~/.config/kaku/kaku.lua exists, but do not open it.
    #[arg(long)]
    ensure_only: bool,
}

impl ConfigCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        let config_path = resolve_user_config_path();
        ensure_config_exists(&config_path)?;
        if self.ensure_only {
            println!("Ensured config: {}", config_path.display());
            return Ok(());
        }

        open_config(&config_path)?;
        println!("Opened config: {}", config_path.display());
        Ok(())
    }
}

fn resolve_user_config_path() -> PathBuf {
    config::CONFIG_DIRS
        .first()
        .cloned()
        .unwrap_or_else(|| config::HOME_DIR.join(".config").join("kaku"))
        .join("kaku.lua")
}

fn ensure_config_exists(config_path: &Path) -> anyhow::Result<()> {
    if config_path.exists() {
        return Ok(());
    }

    let parent = config_path
        .parent()
        .ok_or_else(|| anyhow!("invalid config path: {}", config_path.display()))?;
    config::create_user_owned_dirs(parent).context("create config directory")?;

    std::fs::write(config_path, minimal_user_config_template())
        .context("write minimal user config file")?;
    Ok(())
}

fn minimal_user_config_template() -> &'static str {
    r#"local wezterm = require 'wezterm'

local function resolve_bundled_config()
  local resource_dir = wezterm.executable_dir:gsub('MacOS/?$', 'Resources')
  local bundled = resource_dir .. '/kaku.lua'
  local f = io.open(bundled, 'r')
  if f then
    f:close()
    return bundled
  end

  local app_bundled = '/Applications/Kaku.app/Contents/Resources/kaku.lua'
  f = io.open(app_bundled, 'r')
  if f then
    f:close()
    return app_bundled
  end

  local home = os.getenv('HOME') or ''
  local home_bundled = home .. '/Applications/Kaku.app/Contents/Resources/kaku.lua'
  f = io.open(home_bundled, 'r')
  if f then
    f:close()
    return home_bundled
  end

  local dev_bundled = wezterm.executable_dir .. '/../../assets/macos/Kaku.app/Contents/Resources/kaku.lua'
  f = io.open(dev_bundled, 'r')
  if f then
    f:close()
    return dev_bundled
  end

  return nil
end

local config = {}
local bundled = resolve_bundled_config()

if bundled then
  local ok, loaded = pcall(dofile, bundled)
  if ok and type(loaded) == 'table' then
    config = loaded
  else
    wezterm.log_error('Kaku: failed to load bundled defaults from ' .. bundled)
  end
else
  wezterm.log_error('Kaku: bundled defaults not found')
end

-- User overrides:
-- Kaku intentionally keeps WezTerm-compatible Lua API names
-- for maximum compatibility, so `wezterm.*` here is expected.
--
-- 1) Font family and size
-- config.font = wezterm.font('JetBrains Mono')
-- config.font_size = 16.0
--
-- 2) Color scheme
-- config.color_scheme = 'Builtin Solarized Dark'
--
-- 3) Window size and padding
-- config.initial_cols = 120
-- config.initial_rows = 30
-- config.window_padding = { left = '24px', right = '24px', top = '40px', bottom = '20px' }
--
-- 4) Default shell/program
-- config.default_prog = { '/bin/zsh', '-l' }
--
-- 5) Cursor and scrollback
-- config.default_cursor_style = 'SteadyBar'
-- config.scrollback_lines = 20000
--
-- 6) Add or override a key binding
-- table.insert(config.keys, {
--   key = 'Enter',
--   mods = 'CMD|SHIFT',
--   action = wezterm.action.TogglePaneZoomState,
-- })

return config
"#
}

fn open_config(config_path: &Path) -> anyhow::Result<()> {
    if open_with_editor(config_path)? {
        return Ok(());
    }

    let status = Command::new("/usr/bin/open")
        .arg(config_path)
        .status()
        .context("open config file with default app")?;
    if status.success() {
        return Ok(());
    }
    bail!("failed to open config file: {}", config_path.display());
}

fn open_with_editor(config_path: &Path) -> anyhow::Result<bool> {
    let Some(editor) = std::env::var_os("EDITOR") else {
        return Ok(false);
    };

    let editor = editor.to_string_lossy().trim().to_string();
    if editor.is_empty() {
        return Ok(false);
    }

    let parts = shell_words::split(&editor)
        .with_context(|| format!("failed to parse EDITOR value `{}`", editor))?;
    if parts.is_empty() {
        return Ok(false);
    }

    let status = Command::new(&parts[0])
        .args(parts.iter().skip(1))
        .arg(config_path)
        .status()
        .with_context(|| format!("launch editor `{}`", parts[0]))?;

    Ok(status.success())
}
