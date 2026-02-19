-- Arb Configuration

local wezterm = require 'wezterm'

local config = {}

-- `config_builder` validates every assignment and is expensive on large configs.
-- Keep startup fast by default; enable strict validation only when debugging config.
if os.getenv('ARB_STRICT_CONFIG') == '1' and wezterm.config_builder then
  config = wezterm.config_builder()
end

-- ===== Themes =====
-- Phase 1: Use WezTerm's builtin `color_scheme` mechanism rather than
-- maintaining a separate theme system.
--
-- `arb_themes` is the curated set we expose to users.
-- Keys are stable IDs (for mapping/persistence); values are WezTerm scheme names.
local arb_themes = {
  -- Keep current custom theme as default dark
  ['arb-dark'] = 'Arb Dark',

  -- Dark
  ['catppuccin-mocha'] = 'Catppuccin Mocha',
  ['tokyo-night'] = 'Tokyo Night',
  ['one-dark-pro'] = 'OneDark (Gogh)',
  ['dracula-plus'] = 'Dracula+',

  -- Light
  ['catppuccin-latte'] = 'Catppuccin Latte',
  ['one-light'] = 'One Light (Gogh)',
  ['solarized-light'] = 'Solarized Light (Gogh)',
  ['github-light'] = 'Github',
}

local function get_appearance_kind(window)
  local appearance
  if wezterm.gui and wezterm.gui.get_appearance then
    appearance = wezterm.gui.get_appearance()
  elseif window and window.get_appearance then
    appearance = window:get_appearance()
  end

  if appearance and appearance:find('Dark') then
    return 'Dark'
  end
  return 'Light'
end

local function arb_default_theme_pair()
  return {
    dark = 'arb-dark',
    light = 'catppuccin-latte',
  }
end

local function resolve_scheme_name(theme_id)
  if type(theme_id) ~= 'string' or theme_id == '' then
    return nil
  end
  return arb_themes[theme_id]
end

local function pick_effective_scheme(window, overrides)
  overrides = overrides or {}

  local appearance_kind = get_appearance_kind(window)
  local pair = arb_default_theme_pair()

  -- User override via config overrides.
  -- Supported keys:
  --   overrides.arb_theme = '<theme-id>'
  --   overrides.arb_theme_dark = '<theme-id>'
  --   overrides.arb_theme_light = '<theme-id>'
  --
  -- (This is intentionally overrides-only for now so we can ship without
  -- changing window/ crate or adding additional config plumbing.)
  if type(overrides.arb_theme) == 'string' then
    return resolve_scheme_name(overrides.arb_theme) or nil
  end
  if type(overrides.arb_theme_dark) == 'string' then
    if resolve_scheme_name(overrides.arb_theme_dark) then
      pair.dark = overrides.arb_theme_dark
    end
  end
  if type(overrides.arb_theme_light) == 'string' then
    if resolve_scheme_name(overrides.arb_theme_light) then
      pair.light = overrides.arb_theme_light
    end
  end

  if appearance_kind == 'Dark' then
    return resolve_scheme_name(pair.dark)
  end
  return resolve_scheme_name(pair.light)
end

local function scheme_tab_bar_colors(scheme)
  -- Prefer scheme.tab_bar if present; fall back to simple derived colors.
  local bg = scheme.background or '#15141b'
  local fg = scheme.foreground or '#edecee'
  local inactive_fg = scheme.ansi and scheme.ansi[8] or '#6b6b6b'
  local active_bg = scheme.selection_bg or (scheme.brights and scheme.brights[1]) or '#29263c'

  return {
    background = bg,
    active_tab = {
      bg_color = active_bg,
      fg_color = fg,
      intensity = 'Bold',
      underline = 'None',
      italic = false,
      strikethrough = false,
    },
    inactive_tab = {
      bg_color = bg,
      fg_color = inactive_fg,
      intensity = 'Normal',
    },
    inactive_tab_hover = {
      bg_color = scheme.selection_bg or bg,
      fg_color = fg,
      italic = false,
    },
    new_tab = {
      bg_color = bg,
      fg_color = inactive_fg,
    },
    new_tab_hover = {
      bg_color = scheme.selection_bg or bg,
      fg_color = fg,
    },
  }
end

local function apply_theme_to_overrides(overrides, scheme_name)
  local scheme = wezterm.color.get_builtin_schemes()[scheme_name]
  if not scheme then
    return overrides
  end

  -- Ensure builtins win for terminal palette via `color_scheme`, while we keep
  -- arb-specific surfaces (tab bar, split, titlebar) in sync with the scheme.
  overrides.color_scheme = scheme_name

  overrides.colors = overrides.colors or {}
  overrides.colors.tab_bar = scheme.tab_bar or scheme_tab_bar_colors(scheme)
  overrides.colors.split = scheme.split or (scheme.brights and scheme.brights[1]) or '#3d3a4f'

  overrides.window_frame = overrides.window_frame or {}
  local titlebar_bg = (scheme.tab_bar and scheme.tab_bar.background) or scheme.background
  overrides.window_frame.active_titlebar_bg = titlebar_bg
  overrides.window_frame.inactive_titlebar_bg = titlebar_bg

  -- Also keep format-tab-title callback in sync (it uses hardcoded colors).
  overrides.arb_tab_title_fg_active = (scheme.tab_bar and scheme.tab_bar.active_tab and scheme.tab_bar.active_tab.fg_color)
    or scheme.foreground
    or '#edecee'
  overrides.arb_tab_title_fg_inactive = (scheme.tab_bar and scheme.tab_bar.inactive_tab and scheme.tab_bar.inactive_tab.fg_color)
    or (scheme.ansi and scheme.ansi[8])
    or '#6b6b6b'

  return overrides
end

-- ===== Theme Persistence =====
-- Persist the user's theme choice to disk so it survives restarts.
-- State file: ~/.config/arb/.arb_theme (plain text, contains just the theme ID)

local arb_theme_state_path = (os.getenv('HOME') or '') .. '/.config/arb/.arb_theme'

local function save_theme_state(theme_id)
  if type(theme_id) ~= 'string' or theme_id == '' then
    return
  end
  -- Ensure the directory exists
  os.execute('mkdir -p ' .. (os.getenv('HOME') or '') .. '/.config/arb')
  local f = io.open(arb_theme_state_path, 'w')
  if f then
    f:write(theme_id)
    f:close()
  end
end

local function load_theme_state()
  local f = io.open(arb_theme_state_path, 'r')
  if not f then
    return nil
  end
  local theme_id = f:read('*all')
  f:close()
  if not theme_id or theme_id == '' then
    return nil
  end
  -- Strip whitespace/newlines
  theme_id = theme_id:match('^%s*(.-)%s*$')
  if theme_id == '' then
    return nil
  end
  return theme_id
end

wezterm.on('window-config-reloaded', function(window)
  local overrides = window:get_config_overrides() or {}
  local desired = pick_effective_scheme(window, overrides)
  if not desired then
    return
  end
  if overrides.color_scheme == desired then
    return
  end

  window:set_config_overrides(apply_theme_to_overrides(overrides, desired))
end)




local function basename(path)
  return path:match('([^/]+)$')
end

-- URL decode helper for Chinese characters in paths
-- Converts %E9%9F%B3%E4%B9%90 -> 音乐
local function url_decode(str)
  if not str then
    return str
  end
  -- First, handle UTF-8 encoded sequences (%XX%YY%ZZ)
  local result = str:gsub('%%([0-9A-Fa-f][0-9A-Fa-f])', function(hex)
    return string.char(tonumber(hex, 16))
  end)
  return result
end

local function padding_matches(current, expected)
  return current
    and current.left == expected.left
    and current.right == expected.right
    and current.top == expected.top
    and current.bottom == expected.bottom
end

local fullscreen_uniform_padding = {
  left = '40px',
  right = '40px',
  top = '70px',
  bottom = '30px',
}

local function update_window_config(window, is_full_screen)
  local overrides = window:get_config_overrides() or {}
  if is_full_screen then
    if not padding_matches(overrides.window_padding, fullscreen_uniform_padding) or overrides.hide_tab_bar_if_only_one_tab ~= false then
      overrides.window_padding = fullscreen_uniform_padding
      overrides.hide_tab_bar_if_only_one_tab = false
      window:set_config_overrides(overrides)
    end
    return
  end

  if overrides.window_padding ~= nil or overrides.hide_tab_bar_if_only_one_tab ~= nil then
    overrides.window_padding = nil
    overrides.hide_tab_bar_if_only_one_tab = nil
    window:set_config_overrides(overrides)
  end
end

local function extract_path_from_cwd(cwd)
  if not cwd then
    return ''
  end

  local path = ''
  if type(cwd) == 'table' then
    path = cwd.file_path or cwd.path or tostring(cwd)
  else
    path = tostring(cwd)
  end

  path = path:gsub('^file://[^/]*', ''):gsub('/$', '')
  -- Decode URL-encoded characters (e.g., %E9%9F%B3%E4%B9%90 -> 音乐)
  path = url_decode(path)
  return path
end

local function tab_path_parts(pane)
  local cwd = pane.current_working_dir
  if not cwd then
    local ok, runtime_cwd = pcall(function()
      return pane:get_current_working_dir()
    end)
    if ok then
      cwd = runtime_cwd
    end
  end

  local path = extract_path_from_cwd(cwd)
  if path == '' then
    return '', ''
  end

  local current = basename(path) or path
  local parent_path = path:match('(.+)/[^/]+$') or ''
  local parent = basename(parent_path) or parent_path
  return parent, current
end

wezterm.on('format-tab-title', function(tab, _, _, _, _, max_width)
  local parent, current = tab_path_parts(tab.active_pane)
  local text = current
  if parent ~= '' and current ~= '' then
    text = parent .. '/' .. current
  end
  if text == '' then
    text = tab.active_pane.title
  end

  -- Preserve default tab index behavior when enabled.
  if config.show_tab_index_in_tab_bar then
    local idx = tab.tab_index
    if not config.tab_and_split_indices_are_zero_based then
      idx = idx + 1
    end
    text = tostring(idx) .. ': ' .. text
  end

  if tab.active_pane.is_zoomed then
    text = text .. ' [Z]'
  end
  text = wezterm.truncate_right(text, math.max(8, max_width - 2))

  local overrides = {}
  pcall(function()
    if tab.window then
      overrides = tab.window:get_config_overrides() or {}
    end
  end)
  local active_fg = overrides.arb_tab_title_fg_active or '#edecee'
  local inactive_fg = overrides.arb_tab_title_fg_inactive or '#6b6b6b'
  local fg = tab.is_active and active_fg or inactive_fg
  local intensity = tab.is_active and 'Bold' or 'Normal'
  return {
    { Attribute = { Intensity = intensity } },
    { Foreground = { Color = fg } },
    { Text = ' ' .. text .. ' ' },
  }
end)

wezterm.on('window-resized', function(window, pane)
  local dims = window:get_dimensions()
  update_window_config(window, dims.is_full_screen)
end)

wezterm.on('update-right-status', function(window)
  local dims = window:get_dimensions()
  if not dims.is_full_screen then
    window:set_right_status('')
    return
  end

  local clock_icon = wezterm.nerdfonts.md_clock_time_four_outline
    or wezterm.nerdfonts.md_clock_outline
    or ''
  local text = wezterm.strftime('%H:%M')
  if clock_icon ~= '' then
    window:set_right_status(wezterm.format({
      { Foreground = { Color = '#6b6b6b' } },
      { Text = ' ' .. clock_icon .. ' ' .. text .. ' ' },
    }))
    return
  end
  window:set_right_status(wezterm.format({
    { Foreground = { Color = '#6b6b6b' } },
    { Text = ' ' .. text .. ' ' },
  }))
end)

wezterm.on('rename-tab-complete', function(window, pane, line)
  if not line or line == '' then
    return
  end
  -- Update current tab title; the GUI triggers this on double-click.
  -- The exact API is provided by the host application.
  pcall(function()
    window:active_tab():set_title(line)
  end)
end)

-- ===== Font =====
config.font = wezterm.font_with_fallback({
  { family = 'JetBrains Mono', weight = 'Regular' },
  { family = 'PingFang SC', weight = 'Regular' },
  'Apple Color Emoji',
})

config.font_rules = {
  {
    intensity = 'Normal',
    italic = true,
    font = wezterm.font_with_fallback({
      { family = 'JetBrains Mono', weight = 'Regular', italic = false },
      { family = 'PingFang SC', weight = 'Regular' },
    }),
  },
}

config.bold_brightens_ansi_colors = false
config.font_size = 17.0
config.line_height = 1.30
config.cell_width = 1.00
config.harfbuzz_features = { 'calt=0', 'clig=0', 'liga=0' }
config.use_cap_height_to_scale_fallback_fonts = false

config.freetype_load_target = 'Normal'

config.allow_square_glyphs_to_overflow_width = 'Always'
config.custom_block_glyphs = true
config.unicode_version = 14

-- ===== Cursor =====
config.default_cursor_style = 'BlinkingBar'
config.cursor_thickness = '2px'
config.cursor_blink_rate = 0

-- ===== Scrollback =====
config.scrollback_lines = 10000

-- ===== Mouse =====
config.selection_word_boundary = ' \t\n{}[]()"\'-'  -- Smart selection boundaries

-- ===== Window =====
config.window_padding = {
  left = '40px',
  right = '40px',
  top = '70px',
  bottom = '30px',
}

config.initial_cols = 110
config.initial_rows = 22
config.window_decorations = "INTEGRATED_BUTTONS|RESIZE"
config.window_frame = {
  font = wezterm.font({ family = 'JetBrains Mono', weight = 'Regular' }),
  font_size = 13.0,
  active_titlebar_bg = '#15141b',
  inactive_titlebar_bg = '#15141b',
}

-- Default scheme; persisted theme (if any) is applied after config.colors is set.
config.color_scheme = 'Arb Dark'

config.window_close_confirmation = 'NeverPrompt'

-- ===== Tab Bar =====
config.enable_tab_bar = true
config.tab_bar_position = 'Left'
config.use_fancy_tab_bar = true
config.tab_max_width = 32
config.hide_tab_bar_if_only_one_tab = false
config.show_tab_index_in_tab_bar = true
config.show_new_tab_button_in_tab_bar = false

-- Color scheme for tabs
config.colors = {
  -- Background
  foreground = '#edecee',
  background = '#15141b',

  -- Cursor
  cursor_bg = '#a277ff',
  cursor_fg = '#15141b',
  cursor_border = '#a277ff',

  -- Selection
  selection_bg = '#29263c',
  selection_fg = 'none',

  -- Normal colors (ANSI 0-7)
  ansi = {
    '#110f18',  -- black
    '#ff6767',  -- red
    '#61ffca',  -- green
    '#ffca85',  -- yellow
    '#a277ff',  -- blue
    '#a277ff',  -- magenta
    '#61ffca',  -- cyan
    '#edecee',  -- white
  },

  -- Bright colors (ANSI 8-15)
  brights = {
    '#4d4d4d',  -- bright black
    '#ff6767',  -- bright red
    '#61ffca',  -- bright green
    '#ffca85',  -- bright yellow
    '#a277ff',  -- bright blue
    '#a277ff',  -- bright magenta
    '#61ffca',  -- bright cyan
    '#edecee',  -- bright white
  },

  -- Split separator color (increased contrast for better visibility)
  split = '#3d3a4f',

  -- Tab bar colors
  tab_bar = {
    background = '#15141b',

    active_tab = {
      bg_color = '#29263c',
      fg_color = '#edecee',
      intensity = 'Bold',
      underline = 'None',
      italic = false,
      strikethrough = false,
    },

    inactive_tab = {
      bg_color = '#15141b',
      fg_color = '#6b6b6b',
      intensity = 'Normal',
    },

    inactive_tab_hover = {
      bg_color = '#1f1d28',
      fg_color = '#9b9b9b',
      italic = false,
    },

    new_tab = {
      bg_color = '#15141b',
      fg_color = '#6b6b6b',
    },

    new_tab_hover = {
      bg_color = '#1f1d28',
      fg_color = '#9b9b9b',
    },
  },
}

-- Register custom Arb default scheme so it can be used like a builtin.
-- We keep the existing hardcoded palette under this scheme name.
config.color_schemes = {
  ['Arb Dark'] = {
    foreground = config.colors.foreground,
    background = config.colors.background,
    cursor_bg = config.colors.cursor_bg,
    cursor_fg = config.colors.cursor_fg,
    cursor_border = config.colors.cursor_border,
    selection_bg = config.colors.selection_bg,
    selection_fg = config.colors.selection_fg,
    ansi = config.colors.ansi,
    brights = config.colors.brights,
    tab_bar = config.colors.tab_bar,
    split = config.colors.split,
  },
}

-- Apply persisted theme at config-build time (avoids race conditions from window events).
-- If the user previously selected a theme via Cmd+Shift+T, restore it now.
do
  local persisted_id = load_theme_state()
  if persisted_id then
    local scheme_name = resolve_scheme_name(persisted_id)
    if scheme_name then
      -- Merge all builtin schemes + our custom ones so the lookup always succeeds.
      local all_schemes = wezterm.color.get_builtin_schemes()
      if config.color_schemes then
        for k, v in pairs(config.color_schemes) do
          all_schemes[k] = v
        end
      end

      local scheme = all_schemes[scheme_name]
      if scheme then
        config.color_scheme = scheme_name

        -- Sync tab bar colors
        config.colors = config.colors or {}
        config.colors.tab_bar = scheme.tab_bar or scheme_tab_bar_colors(scheme)
        config.colors.split = scheme.split or (scheme.brights and scheme.brights[1]) or '#3d3a4f'

        -- Sync window frame / titlebar
        config.window_frame = config.window_frame or {}
        local titlebar_bg = (scheme.tab_bar and scheme.tab_bar.background) or scheme.background
        config.window_frame.active_titlebar_bg = titlebar_bg
        config.window_frame.inactive_titlebar_bg = titlebar_bg
      end
    end
  end
end

-- ===== Shell =====
local user_shell = os.getenv('SHELL')
if user_shell and #user_shell > 0 then
  config.default_prog = { user_shell, '-l' }
else
  config.default_prog = { '/bin/zsh', '-l' }
end

-- ===== macOS Specific =====
-- Keep Left Option as Meta so Alt-based Vim/Neovim keybindings work reliably.
config.send_composed_key_when_left_alt_is_pressed = false
-- Keep Right Option available for composing locale/symbol characters.
config.send_composed_key_when_right_alt_is_pressed = true
config.native_macos_fullscreen_mode = true
config.quit_when_all_windows_are_closed = false

-- ===== Key Bindings =====
config.keys = {
  -- Cmd+R: clear screen + scrollback
  {
    key = 'r',
    mods = 'CMD',
    action = wezterm.action.Multiple({
      wezterm.action.SendKey({ key = 'l', mods = 'CTRL' }),
      wezterm.action.ClearScrollback('ScrollbackAndViewport'),
    }),
  },

  -- Cmd+Q: quit
  {
    key = 'q',
    mods = 'CMD',
    action = wezterm.action.QuitApplication,
  },

  -- Cmd+N: new window
  {
    key = 'n',
    mods = 'CMD',
    action = wezterm.action.SpawnWindow,
  },

  -- Cmd+W: close pane > close tab > hide app
  {
    key = 'w',
    mods = 'CMD',
    action = wezterm.action_callback(function(win, pane)
      local mux_win = win:mux_window()
      local tabs = mux_win and mux_win:tabs() or {}
      local current_tab = pane:tab()
      local panes = current_tab and current_tab:panes() or {}
      if #panes > 1 then
        win:perform_action(wezterm.action.CloseCurrentPane { confirm = false }, pane)
      elseif #tabs > 1 then
        win:perform_action(wezterm.action.CloseCurrentTab { confirm = false }, pane)
      else
        win:perform_action(wezterm.action.HideApplication, pane)
      end
    end),
  },

  -- Cmd+Shift+W: close current tab
  {
    key = 'w',
    mods = 'CMD|SHIFT',
    action = wezterm.action.CloseCurrentTab({ confirm = false }),
  },

  -- Cmd+T: new tab
  {
    key = 't',
    mods = 'CMD',
    action = wezterm.action.SpawnTab('CurrentPaneDomain'),
  },

  -- Cmd+Ctrl+F: toggle fullscreen
  {
    key = 'f',
    mods = 'CMD|CTRL',
    action = wezterm.action.ToggleFullScreen,
  },

  { key = 'f', mods = 'CMD', action = wezterm.action.Search { Regex = '' } },

  -- Cmd+M: minimize window
  {
    key = 'm',
    mods = 'CMD',
    action = wezterm.action.Hide,
  },

  -- Cmd+H: hide application
  {
    key = 'h',
    mods = 'CMD',
    action = wezterm.action.HideApplication,
  },

  -- Cmd+Shift+.: reload configuration
  {
    key = '.',
    mods = 'CMD|SHIFT',
    action = wezterm.action.ReloadConfiguration,
  },
  -- Some layouts report Shift+. as mapped:>, keep this as a fallback.
  {
    key = 'mapped:>',
    mods = 'CMD',
    action = wezterm.action.ReloadConfiguration,
  },

  -- Cmd+Shift+T: Theme selector
  {
    key = 't',
    mods = 'CMD|SHIFT',
    action = wezterm.action_callback(function(window, pane)
      local overrides = window:get_config_overrides() or {}
      local items = {}
      for id, scheme in pairs(arb_themes) do
        table.insert(items, { label = scheme, id = id })
      end
      table.sort(items, function(a, b)
        return a.label < b.label
      end)

      window:perform_action(
        wezterm.action.InputSelector({
          title = 'Choose Theme',
          choices = items,
          action = wezterm.action_callback(function(inner_window, _, choice_id)
            if not choice_id then
              return
            end
            overrides.arb_theme = choice_id
            local desired = pick_effective_scheme(inner_window, overrides)
            if desired then
              inner_window:set_config_overrides(apply_theme_to_overrides(overrides, desired))
              save_theme_state(choice_id)
            end
          end),
        }),
        pane
      )
    end),
  },

  -- Cmd+Equal/Minus/0: adjust font size
  {
    key = '=',
    mods = 'CMD',
    action = wezterm.action.IncreaseFontSize,
  },
  {
    key = '-',
    mods = 'CMD',
    action = wezterm.action.DecreaseFontSize,
  },
  {
    key = '0',
    mods = 'CMD',
    action = wezterm.action.ResetFontSize,
  },

  -- Alt+Left / Alt+Right: word jump
  {
    key = 'LeftArrow',
    mods = 'OPT',
    action = wezterm.action.SendKey({ key = 'b', mods = 'ALT' }),
  },
  {
    key = 'RightArrow',
    mods = 'OPT',
    action = wezterm.action.SendKey({ key = 'f', mods = 'ALT' }),
  },

  -- Cmd+Left / Cmd+Right: line start/end
  {
    key = 'LeftArrow',
    mods = 'CMD',
    action = wezterm.action.SendKey({ key = 'a', mods = 'CTRL' }),
  },
  {
    key = 'RightArrow',
    mods = 'CMD',
    action = wezterm.action.SendKey({ key = 'e', mods = 'CTRL' }),
  },

  -- Cmd+Backspace: delete to line start
  {
    key = 'Backspace',
    mods = 'CMD',
    action = wezterm.action.SendKey({ key = 'u', mods = 'CTRL' }),
  },

  -- Alt+Backspace: delete word
  {
    key = 'Backspace',
    mods = 'OPT',
    action = wezterm.action.SendKey({ key = 'w', mods = 'CTRL' }),
  },

  -- Cmd+D: vertical split
  {
    key = 'd',
    mods = 'CMD',
    action = wezterm.action.SplitHorizontal({ domain = 'CurrentPaneDomain' }),
  },

  -- Cmd+Shift+D: horizontal split
  {
    key = 'D',
    mods = 'CMD|SHIFT',
    action = wezterm.action.SplitVertical({ domain = 'CurrentPaneDomain' }),
  },

  -- Cmd+Shift+[ / ]: prev/next tab
  {
    key = '[',
    mods = 'CMD|SHIFT',
    action = wezterm.action.ActivateTabRelative(-1),
  },
  {
    key = ']',
    mods = 'CMD|SHIFT',
    action = wezterm.action.ActivateTabRelative(1),
  },

  -- Cmd+Option+Arrow: navigate between splits
  {
    key = 'LeftArrow',
    mods = 'CMD|OPT',
    action = wezterm.action.ActivatePaneDirection('Left'),
  },
  {
    key = 'RightArrow',
    mods = 'CMD|OPT',
    action = wezterm.action.ActivatePaneDirection('Right'),
  },
  {
    key = 'UpArrow',
    mods = 'CMD|OPT',
    action = wezterm.action.ActivatePaneDirection('Up'),
  },
  {
    key = 'DownArrow',
    mods = 'CMD|OPT',
    action = wezterm.action.ActivatePaneDirection('Down'),
  },

  -- Cmd+1~9: switch tab
  {
    key = '1',
    mods = 'CMD',
    action = wezterm.action.ActivateTab(0),
  },
  {
    key = '2',
    mods = 'CMD',
    action = wezterm.action.ActivateTab(1),
  },
  {
    key = '3',
    mods = 'CMD',
    action = wezterm.action.ActivateTab(2),
  },
  {
    key = '4',
    mods = 'CMD',
    action = wezterm.action.ActivateTab(3),
  },
  {
    key = '5',
    mods = 'CMD',
    action = wezterm.action.ActivateTab(4),
  },
  {
    key = '6',
    mods = 'CMD',
    action = wezterm.action.ActivateTab(5),
  },
  {
    key = '7',
    mods = 'CMD',
    action = wezterm.action.ActivateTab(6),
  },
  {
    key = '8',
    mods = 'CMD',
    action = wezterm.action.ActivateTab(7),
  },
  {
    key = '9',
    mods = 'CMD',
    action = wezterm.action.ActivateTab(8),
  },

  -- Cmd+Enter / Shift+Enter: newline without execute
  {
    key = 'Enter',
    mods = 'CMD',
    action = wezterm.action.SendString('\n'),
  },
  {
    key = 'Enter',
    mods = 'SHIFT',
    action = wezterm.action.SendString('\n'),
  },

  -- Cmd+Shift+Enter: Toggle Pane Zoom (Maximize active pane)
  {
    key = 'Enter',
    mods = 'CMD|SHIFT',
    action = wezterm.action.TogglePaneZoomState,
  },

  -- Cmd+Ctrl+Arrows: Resize panes
  {
    key = 'LeftArrow',
    mods = 'CMD|CTRL',
    action = wezterm.action.AdjustPaneSize { 'Left', 5 },
  },
  {
    key = 'RightArrow',
    mods = 'CMD|CTRL',
    action = wezterm.action.AdjustPaneSize { 'Right', 5 },
  },
  {
    key = 'UpArrow',
    mods = 'CMD|CTRL',
    action = wezterm.action.AdjustPaneSize { 'Up', 5 },
  },
  {
    key = 'DownArrow',
    mods = 'CMD|CTRL',
    action = wezterm.action.AdjustPaneSize { 'Down', 5 },
  },


}

-- Copy on select (equivalent to Kitty's copy_on_select)
config.mouse_bindings = {
  {
    event = { Up = { streak = 1, button = 'Left' } },
    mods = 'NONE',
    action = wezterm.action.CompleteSelectionOrOpenLinkAtMouseCursor('ClipboardAndPrimarySelection'),
  },
  {
    event = { Up = { streak = 1, button = 'Left' } },
    mods = 'CMD',
    action = wezterm.action.OpenLinkAtMouseCursor,
  },
}

-- ===== Performance =====
config.enable_scroll_bar = false
config.front_end = 'OpenGL'
config.webgpu_power_preference = 'HighPerformance'
config.animation_fps = 60
config.max_fps = 60

-- ===== Visuals & Splits =====
-- Split pane gap: gutter = 1 + 2*gap cells, giving ~40px padding on each side
config.split_pane_gap = 2

-- Inactive panes: No dimming (consistent background)
config.inactive_pane_hsb = {
  saturation = 1.0,
  brightness = 1.0,
}

-- Prevent accidental clicks when focusing panes
config.swallow_mouse_click_on_pane_focus = true
config.swallow_mouse_click_on_window_focus = true

-- ===== First Run Experience & Config Version Check =====
wezterm.on('gui-startup', function(cmd)
  local home = os.getenv("HOME")
  local current_version = 6  -- Update this when config changes

  -- Check for configuration version
  local version_file = home .. "/.config/arb/.arb_config_version"
  local is_first_run = false
  local needs_update = false

  -- Read current user version
  local vf = io.open(version_file, "r")
  if vf then
    -- Has version file, check if update needed
    local user_version = tonumber(vf:read("*all")) or 0
    vf:close()
    if user_version < current_version then
      needs_update = true
    end
  else
    -- New user, show first run
    is_first_run = true
  end

  if is_first_run then
    -- First run experience
    os.execute("mkdir -p " .. home .. "/.config/arb")

    local resource_dir = wezterm.executable_dir:gsub("MacOS/?$", "Resources")
    local first_run_script = resource_dir .. "/first_run.sh"

    -- Fallback for dev environment
    local f_script = io.open(first_run_script, "r")
    if not f_script then
      first_run_script = wezterm.executable_dir .. "/../../assets/shell-integration/first_run.sh"
    else
      f_script:close()
    end

    wezterm.mux.spawn_window {
      args = { 'bash', first_run_script },
      width = 106,
      height = 22,
    }
    return
  end

  if needs_update then
    -- Re-run guided setup on version upgrades
    local resource_dir = wezterm.executable_dir:gsub("MacOS/?$", "Resources")
    local first_run_script = resource_dir .. "/first_run.sh"

    -- Fallback for dev environment
    local f_script = io.open(first_run_script, "r")
    if not f_script then
      first_run_script = wezterm.executable_dir .. "/../../assets/shell-integration/first_run.sh"
    else
      f_script:close()
    end

    wezterm.mux.spawn_window {
      args = { 'bash', first_run_script },
      width = 106,
      height = 22,
    }
    return
  end

  -- Normal startup
  if not cmd then
    wezterm.mux.spawn_window(cmd or {})
  end
end)

return config
