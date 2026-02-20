use anyhow::{anyhow, bail, Context};
use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Parser, Clone, Default)]
pub struct ConfigCommand {
    /// Ensure ~/.config/arb/arb.lua exists, but do not open it.
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
        .unwrap_or_else(|| config::HOME_DIR.join(".config").join("arb"))
        .join("arb.lua")
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
    r#"-- arb config template v1
local wezterm = require 'wezterm'

local function resolve_bundled_config()
  local resource_dir = wezterm.executable_dir:gsub('MacOS/?$', 'Resources')
  local bundled = resource_dir .. '/arb.lua'
  local f = io.open(bundled, 'r')
  if f then
    f:close()
    return bundled
  end

  local app_bundled = '/Applications/Arb.app/Contents/Resources/arb.lua'
  f = io.open(app_bundled, 'r')
  if f then
    f:close()
    return app_bundled
  end

  local home = os.getenv('HOME') or ''
  local home_bundled = home .. '/Applications/Arb.app/Contents/Resources/arb.lua'
  f = io.open(home_bundled, 'r')
  if f then
    f:close()
    return home_bundled
  end

  local dev_bundled = wezterm.executable_dir .. '/../../assets/macos/Arb.app/Contents/Resources/arb.lua'
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
    wezterm.log_error('Arb: failed to load bundled defaults from ' .. bundled)
  end
else
  wezterm.log_error('Arb: bundled defaults not found')
end

-- ═══════════════════════════════════════════════════════════════════
-- User overrides
-- ═══════════════════════════════════════════════════════════════════
-- Arb intentionally keeps WezTerm-compatible Lua API names
-- for maximum compatibility, so `wezterm.*` here is expected.

-- ═══ Appearance ═══

-- Pick a font that feels right for long coding sessions
-- config.font = wezterm.font('JetBrains Mono')
-- config.font_size = 16.0

-- Switch color scheme to match your editor/IDE theme
-- config.color_scheme = 'Builtin Solarized Dark'

-- Fine-tune window padding for a cleaner look
-- config.window_padding = { left = '24px', right = '24px', top = '40px', bottom = '20px' }

-- ═══ Terminal Behavior ═══

-- Set a preferred default shell (login mode for full profile loading)
-- config.default_prog = { '/bin/zsh', '-l' }

-- Choose a cursor style that's easy to spot in dense output
-- config.default_cursor_style = 'SteadyBar'

-- Keep more history so you can scroll back through build logs
-- config.scrollback_lines = 20000

-- Set initial window dimensions (columns x rows)
-- config.initial_cols = 120
-- config.initial_rows = 30

-- ═══ AI Coding Workflow ═══

-- Large scrollback so you never lose AI-generated output
-- config.scrollback_lines = 50000

-- Wider window for side-by-side terminal + AI chat
-- config.initial_cols = 140

-- Skip the close confirmation — AI sessions are easy to restart
-- config.window_close_confirmation = 'NeverPrompt'

-- Quick keybinding to open a new tab for parallel tasks
-- table.insert(config.keys, {
--   key = 't',
--   mods = 'CMD',
--   action = wezterm.action.SpawnTab 'CurrentPaneDomain',
-- })

-- ═══ Panes & Splits ═══

-- Navigate between panes with Cmd+Arrow keys
-- table.insert(config.keys, {
--   key = 'LeftArrow',
--   mods = 'CMD',
--   action = wezterm.action.ActivatePaneDirection 'Left',
-- })
-- table.insert(config.keys, {
--   key = 'RightArrow',
--   mods = 'CMD',
--   action = wezterm.action.ActivatePaneDirection 'Right',
-- })
-- table.insert(config.keys, {
--   key = 'UpArrow',
--   mods = 'CMD',
--   action = wezterm.action.ActivatePaneDirection 'Up',
-- })
-- table.insert(config.keys, {
--   key = 'DownArrow',
--   mods = 'CMD',
--   action = wezterm.action.ActivatePaneDirection 'Down',
-- })

-- Split the current pane horizontally or vertically
-- table.insert(config.keys, {
--   key = 'd',
--   mods = 'CMD',
--   action = wezterm.action.SplitHorizontal { domain = 'CurrentPaneDomain' },
-- })
-- table.insert(config.keys, {
--   key = 'd',
--   mods = 'CMD|SHIFT',
--   action = wezterm.action.SplitVertical { domain = 'CurrentPaneDomain' },
-- })

-- Toggle pane zoom to focus on one pane temporarily
-- table.insert(config.keys, {
--   key = 'Enter',
--   mods = 'CMD|SHIFT',
--   action = wezterm.action.TogglePaneZoomState,
-- })

-- ═══ Advanced ═══

-- Hide the tab bar for a distraction-free, minimal look
-- config.enable_tab_bar = false

-- Slight transparency to keep your desktop visible underneath
-- config.window_background_opacity = 0.95

-- Set environment variables for all shells launched by Arb
-- config.set_environment_variables = {
--   EDITOR = 'nvim',
-- }

return config
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_contains_version_marker() {
        let template = minimal_user_config_template();
        assert!(
            template.starts_with("-- arb config template v1"),
            "template must begin with version marker"
        );
    }

    #[test]
    fn template_ends_with_return_config() {
        let template = minimal_user_config_template();
        let trimmed = template.trim_end();
        assert!(
            trimmed.ends_with("return config"),
            "template must end with `return config`"
        );
    }

    #[test]
    fn template_contains_required_sections() {
        let template = minimal_user_config_template();
        let required_sections = [
            "═══ Appearance ═══",
            "═══ Terminal Behavior ═══",
            "═══ AI Coding Workflow ═══",
            "═══ Panes & Splits ═══",
            "═══ Advanced ═══",
        ];
        for section in &required_sections {
            assert!(
                template.contains(section),
                "template must contain section header: {}",
                section
            );
        }
    }

    #[test]
    fn template_preserves_wezterm_require() {
        let template = minimal_user_config_template();
        assert!(
            template.contains("local wezterm = require 'wezterm'"),
            "template must keep the wezterm require line"
        );
    }
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
