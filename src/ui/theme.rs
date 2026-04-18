use std::path::Path;

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Runtime theme with resolved ratatui colors.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Theme {
    pub name: &'static str,

    // Base colors
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,

    // Panel border colors
    pub cpu_border: Color,
    pub mem_border: Color,
    pub net_border: Color,
    pub disk_border: Color,
    pub gpu_border: Color,
    pub sensor_border: Color,
    pub battery_border: Color,
    pub docker_border: Color,
    pub k8s_border: Color,
    pub remote_border: Color,

    // Semantic colors (usage/temperature thresholds)
    pub good: Color,
    pub warning: Color,
    pub critical: Color,

    // UI element colors
    pub text_primary: Color,
    pub text_secondary: Color,
    pub bar_filled: Color,
    pub bar_empty: Color,
    pub graph_line: Color,
    pub sparkline_cpu: Color,
    pub sparkline_mem: Color,
    pub sparkline_net: Color,
    pub sparkline_gpu: Color,
    pub sparkline_sensor: Color,

    // Status bar
    pub status_bar_bg: Color,
    pub status_bar_fg: Color,
    pub status_bar_accent: Color,

    // Selection / highlight
    pub selected_bg: Color,
    pub selected_fg: Color,

    // Dialog
    pub dialog_border: Color,
    pub dialog_bg: Color,
}

// ---------------------------------------------------------------------------
// Built-in themes
// ---------------------------------------------------------------------------

pub fn default_theme() -> Theme {
    Theme {
        name: "default",
        bg: Color::Reset,
        fg: Color::White,
        accent: Color::Cyan,
        cpu_border: Color::Cyan,
        mem_border: Color::Green,
        net_border: Color::Yellow,
        disk_border: Color::Blue,
        gpu_border: Color::Blue,
        sensor_border: Color::Magenta,
        battery_border: Color::Green,
        docker_border: Color::Cyan,
        k8s_border: Color::Cyan,
        remote_border: Color::Magenta,
        good: Color::Green,
        warning: Color::Yellow,
        critical: Color::Red,
        text_primary: Color::White,
        text_secondary: Color::DarkGray,
        bar_filled: Color::Green,
        bar_empty: Color::DarkGray,
        graph_line: Color::Cyan,
        sparkline_cpu: Color::Red,
        sparkline_mem: Color::Green,
        sparkline_net: Color::Blue,
        sparkline_gpu: Color::Magenta,
        sparkline_sensor: Color::Magenta,
        status_bar_bg: Color::DarkGray,
        status_bar_fg: Color::White,
        status_bar_accent: Color::Cyan,
        selected_bg: Color::DarkGray,
        selected_fg: Color::White,
        dialog_border: Color::Cyan,
        dialog_bg: Color::Reset,
    }
}

fn dracula_theme() -> Theme {
    // Dracula palette
    const BG: Color = Color::Rgb(40, 42, 54);
    const FG: Color = Color::Rgb(248, 248, 242);
    const PURPLE: Color = Color::Rgb(189, 147, 249);
    const CYAN: Color = Color::Rgb(139, 233, 253);
    const GREEN: Color = Color::Rgb(80, 250, 123);
    const YELLOW: Color = Color::Rgb(241, 250, 140);
    const PINK: Color = Color::Rgb(255, 121, 198);
    const RED: Color = Color::Rgb(255, 85, 85);
    const ORANGE: Color = Color::Rgb(255, 184, 108);
    const COMMENT: Color = Color::Rgb(98, 114, 164);

    Theme {
        name: "dracula",
        bg: BG,
        fg: FG,
        accent: PURPLE,
        cpu_border: CYAN,
        mem_border: GREEN,
        net_border: YELLOW,
        disk_border: PINK,
        gpu_border: PURPLE,
        sensor_border: ORANGE,
        battery_border: GREEN,
        docker_border: CYAN,
        k8s_border: CYAN,
        remote_border: PURPLE,
        good: GREEN,
        warning: YELLOW,
        critical: RED,
        text_primary: FG,
        text_secondary: COMMENT,
        bar_filled: GREEN,
        bar_empty: COMMENT,
        graph_line: CYAN,
        sparkline_cpu: RED,
        sparkline_mem: GREEN,
        sparkline_net: CYAN,
        sparkline_gpu: PURPLE,
        sparkline_sensor: ORANGE,
        status_bar_bg: Color::Rgb(68, 71, 90),
        status_bar_fg: FG,
        status_bar_accent: PURPLE,
        selected_bg: Color::Rgb(68, 71, 90),
        selected_fg: FG,
        dialog_border: PURPLE,
        dialog_bg: BG,
    }
}

fn gruvbox_dark_theme() -> Theme {
    const BG: Color = Color::Rgb(40, 40, 40);
    const FG: Color = Color::Rgb(235, 219, 178);
    const ORANGE: Color = Color::Rgb(214, 93, 14);
    const RED: Color = Color::Rgb(204, 36, 29);
    const GREEN: Color = Color::Rgb(152, 151, 26);
    const YELLOW: Color = Color::Rgb(215, 153, 33);
    const BLUE: Color = Color::Rgb(69, 133, 136);
    const PURPLE: Color = Color::Rgb(177, 98, 134);
    const AQUA: Color = Color::Rgb(104, 157, 106);
    const GRAY: Color = Color::Rgb(146, 131, 116);
    const BG_LIGHT: Color = Color::Rgb(80, 73, 69);

    Theme {
        name: "gruvbox-dark",
        bg: BG,
        fg: FG,
        accent: ORANGE,
        cpu_border: ORANGE,
        mem_border: GREEN,
        net_border: YELLOW,
        disk_border: BLUE,
        gpu_border: PURPLE,
        sensor_border: AQUA,
        battery_border: GREEN,
        docker_border: BLUE,
        k8s_border: AQUA,
        remote_border: PURPLE,
        good: GREEN,
        warning: YELLOW,
        critical: RED,
        text_primary: FG,
        text_secondary: GRAY,
        bar_filled: AQUA,
        bar_empty: BG_LIGHT,
        graph_line: ORANGE,
        sparkline_cpu: RED,
        sparkline_mem: GREEN,
        sparkline_net: BLUE,
        sparkline_gpu: PURPLE,
        sparkline_sensor: AQUA,
        status_bar_bg: BG_LIGHT,
        status_bar_fg: FG,
        status_bar_accent: ORANGE,
        selected_bg: BG_LIGHT,
        selected_fg: FG,
        dialog_border: ORANGE,
        dialog_bg: BG,
    }
}

fn catppuccin_mocha_theme() -> Theme {
    const BG: Color = Color::Rgb(30, 30, 46);
    const FG: Color = Color::Rgb(205, 214, 244);
    const BLUE: Color = Color::Rgb(137, 180, 250);
    const GREEN: Color = Color::Rgb(166, 227, 161);
    const YELLOW: Color = Color::Rgb(249, 226, 175);
    const RED: Color = Color::Rgb(243, 139, 168);
    const MAUVE: Color = Color::Rgb(203, 166, 247);
    const TEAL: Color = Color::Rgb(148, 226, 213);
    const PEACH: Color = Color::Rgb(250, 179, 135);
    const PINK: Color = Color::Rgb(245, 194, 231);
    const OVERLAY0: Color = Color::Rgb(108, 112, 134);
    const SURFACE0: Color = Color::Rgb(49, 50, 68);

    Theme {
        name: "catppuccin-mocha",
        bg: BG,
        fg: FG,
        accent: BLUE,
        cpu_border: BLUE,
        mem_border: GREEN,
        net_border: YELLOW,
        disk_border: MAUVE,
        gpu_border: TEAL,
        sensor_border: PEACH,
        battery_border: GREEN,
        docker_border: TEAL,
        k8s_border: BLUE,
        remote_border: MAUVE,
        good: GREEN,
        warning: YELLOW,
        critical: RED,
        text_primary: FG,
        text_secondary: OVERLAY0,
        bar_filled: BLUE,
        bar_empty: SURFACE0,
        graph_line: TEAL,
        sparkline_cpu: RED,
        sparkline_mem: GREEN,
        sparkline_net: BLUE,
        sparkline_gpu: MAUVE,
        sparkline_sensor: PEACH,
        status_bar_bg: SURFACE0,
        status_bar_fg: FG,
        status_bar_accent: BLUE,
        selected_bg: SURFACE0,
        selected_fg: FG,
        dialog_border: PINK,
        dialog_bg: BG,
    }
}

fn catppuccin_latte_theme() -> Theme {
    const BG: Color = Color::Rgb(239, 241, 245);
    const FG: Color = Color::Rgb(76, 79, 105);
    const BLUE: Color = Color::Rgb(30, 102, 245);
    const GREEN: Color = Color::Rgb(64, 160, 43);
    const YELLOW: Color = Color::Rgb(223, 142, 29);
    const RED: Color = Color::Rgb(210, 15, 57);
    const MAUVE: Color = Color::Rgb(136, 57, 239);
    const TEAL: Color = Color::Rgb(23, 146, 153);
    const PEACH: Color = Color::Rgb(254, 100, 11);
    const PINK: Color = Color::Rgb(234, 118, 203);
    const OVERLAY0: Color = Color::Rgb(156, 160, 176);
    const SURFACE0: Color = Color::Rgb(204, 208, 218);

    Theme {
        name: "catppuccin-latte",
        bg: BG,
        fg: FG,
        accent: BLUE,
        cpu_border: BLUE,
        mem_border: GREEN,
        net_border: YELLOW,
        disk_border: MAUVE,
        gpu_border: TEAL,
        sensor_border: PEACH,
        battery_border: GREEN,
        docker_border: TEAL,
        k8s_border: BLUE,
        remote_border: MAUVE,
        good: GREEN,
        warning: YELLOW,
        critical: RED,
        text_primary: FG,
        text_secondary: OVERLAY0,
        bar_filled: BLUE,
        bar_empty: SURFACE0,
        graph_line: TEAL,
        sparkline_cpu: RED,
        sparkline_mem: GREEN,
        sparkline_net: BLUE,
        sparkline_gpu: MAUVE,
        sparkline_sensor: PEACH,
        status_bar_bg: SURFACE0,
        status_bar_fg: FG,
        status_bar_accent: BLUE,
        selected_bg: SURFACE0,
        selected_fg: FG,
        dialog_border: PINK,
        dialog_bg: BG,
    }
}

fn nord_theme() -> Theme {
    const BG: Color = Color::Rgb(46, 52, 64);
    const FG: Color = Color::Rgb(216, 222, 233);
    const FROST0: Color = Color::Rgb(143, 188, 187);
    const FROST1: Color = Color::Rgb(136, 192, 208);
    const FROST2: Color = Color::Rgb(129, 161, 193);
    const FROST3: Color = Color::Rgb(94, 129, 172);
    const GREEN: Color = Color::Rgb(163, 190, 140);
    const YELLOW: Color = Color::Rgb(235, 203, 139);
    const RED: Color = Color::Rgb(191, 97, 106);
    const PURPLE: Color = Color::Rgb(180, 142, 173);
    const ORANGE: Color = Color::Rgb(208, 135, 112);
    const POLAR2: Color = Color::Rgb(67, 76, 94);

    Theme {
        name: "nord",
        bg: BG,
        fg: FG,
        accent: FROST1,
        cpu_border: FROST1,
        mem_border: GREEN,
        net_border: YELLOW,
        disk_border: FROST2,
        gpu_border: FROST3,
        sensor_border: ORANGE,
        battery_border: GREEN,
        docker_border: FROST0,
        k8s_border: FROST2,
        remote_border: PURPLE,
        good: GREEN,
        warning: YELLOW,
        critical: RED,
        text_primary: FG,
        text_secondary: Color::Rgb(76, 86, 106),
        bar_filled: FROST0,
        bar_empty: POLAR2,
        graph_line: FROST1,
        sparkline_cpu: RED,
        sparkline_mem: GREEN,
        sparkline_net: FROST2,
        sparkline_gpu: PURPLE,
        sparkline_sensor: ORANGE,
        status_bar_bg: POLAR2,
        status_bar_fg: FG,
        status_bar_accent: FROST1,
        selected_bg: POLAR2,
        selected_fg: FG,
        dialog_border: FROST1,
        dialog_bg: BG,
    }
}

fn solarized_dark_theme() -> Theme {
    const BASE03: Color = Color::Rgb(0, 43, 54);
    const BASE0: Color = Color::Rgb(131, 148, 150);
    const BASE01: Color = Color::Rgb(88, 110, 117);
    const BASE02: Color = Color::Rgb(7, 54, 66);
    const BLUE: Color = Color::Rgb(38, 139, 210);
    const CYAN: Color = Color::Rgb(42, 161, 152);
    const GREEN: Color = Color::Rgb(133, 153, 0);
    const YELLOW: Color = Color::Rgb(181, 137, 0);
    const RED: Color = Color::Rgb(220, 50, 47);
    const MAGENTA: Color = Color::Rgb(211, 54, 130);
    const VIOLET: Color = Color::Rgb(108, 113, 196);
    const ORANGE: Color = Color::Rgb(203, 75, 22);

    Theme {
        name: "solarized-dark",
        bg: BASE03,
        fg: BASE0,
        accent: BLUE,
        cpu_border: BLUE,
        mem_border: GREEN,
        net_border: YELLOW,
        disk_border: CYAN,
        gpu_border: VIOLET,
        sensor_border: ORANGE,
        battery_border: GREEN,
        docker_border: CYAN,
        k8s_border: BLUE,
        remote_border: MAGENTA,
        good: GREEN,
        warning: YELLOW,
        critical: RED,
        text_primary: BASE0,
        text_secondary: BASE01,
        bar_filled: CYAN,
        bar_empty: BASE02,
        graph_line: BLUE,
        sparkline_cpu: RED,
        sparkline_mem: GREEN,
        sparkline_net: BLUE,
        sparkline_gpu: VIOLET,
        sparkline_sensor: ORANGE,
        status_bar_bg: BASE02,
        status_bar_fg: BASE0,
        status_bar_accent: BLUE,
        selected_bg: BASE02,
        selected_fg: BASE0,
        dialog_border: MAGENTA,
        dialog_bg: BASE03,
    }
}

fn solarized_light_theme() -> Theme {
    const BASE3: Color = Color::Rgb(253, 246, 227);
    const BASE00: Color = Color::Rgb(101, 123, 131);
    const BASE1: Color = Color::Rgb(147, 161, 161);
    const BASE2: Color = Color::Rgb(238, 232, 213);
    const BLUE: Color = Color::Rgb(38, 139, 210);
    const CYAN: Color = Color::Rgb(42, 161, 152);
    const GREEN: Color = Color::Rgb(133, 153, 0);
    const YELLOW: Color = Color::Rgb(181, 137, 0);
    const RED: Color = Color::Rgb(220, 50, 47);
    const MAGENTA: Color = Color::Rgb(211, 54, 130);
    const VIOLET: Color = Color::Rgb(108, 113, 196);
    const ORANGE: Color = Color::Rgb(203, 75, 22);

    Theme {
        name: "solarized-light",
        bg: BASE3,
        fg: BASE00,
        accent: BLUE,
        cpu_border: BLUE,
        mem_border: GREEN,
        net_border: YELLOW,
        disk_border: CYAN,
        gpu_border: VIOLET,
        sensor_border: ORANGE,
        battery_border: GREEN,
        docker_border: CYAN,
        k8s_border: BLUE,
        remote_border: MAGENTA,
        good: GREEN,
        warning: YELLOW,
        critical: RED,
        text_primary: BASE00,
        text_secondary: BASE1,
        bar_filled: CYAN,
        bar_empty: BASE2,
        graph_line: BLUE,
        sparkline_cpu: RED,
        sparkline_mem: GREEN,
        sparkline_net: BLUE,
        sparkline_gpu: VIOLET,
        sparkline_sensor: ORANGE,
        status_bar_bg: BASE2,
        status_bar_fg: BASE00,
        status_bar_accent: BLUE,
        selected_bg: BASE2,
        selected_fg: BASE00,
        dialog_border: MAGENTA,
        dialog_bg: BASE3,
    }
}

fn tokyo_night_theme() -> Theme {
    const BG: Color = Color::Rgb(26, 27, 38);
    const FG: Color = Color::Rgb(169, 177, 214);
    const BLUE: Color = Color::Rgb(122, 162, 247);
    const GREEN: Color = Color::Rgb(158, 206, 106);
    const YELLOW: Color = Color::Rgb(224, 175, 104);
    const RED: Color = Color::Rgb(247, 118, 142);
    const CYAN: Color = Color::Rgb(125, 207, 255);
    const MAGENTA: Color = Color::Rgb(187, 154, 247);
    const TEAL: Color = Color::Rgb(115, 218, 202);
    const ORANGE: Color = Color::Rgb(255, 158, 100);
    const COMMENT: Color = Color::Rgb(86, 95, 137);
    const BG_HIGHLIGHT: Color = Color::Rgb(41, 46, 66);

    Theme {
        name: "tokyo-night",
        bg: BG,
        fg: FG,
        accent: BLUE,
        cpu_border: BLUE,
        mem_border: GREEN,
        net_border: YELLOW,
        disk_border: CYAN,
        gpu_border: MAGENTA,
        sensor_border: ORANGE,
        battery_border: GREEN,
        docker_border: TEAL,
        k8s_border: BLUE,
        remote_border: MAGENTA,
        good: GREEN,
        warning: YELLOW,
        critical: RED,
        text_primary: FG,
        text_secondary: COMMENT,
        bar_filled: BLUE,
        bar_empty: BG_HIGHLIGHT,
        graph_line: CYAN,
        sparkline_cpu: RED,
        sparkline_mem: GREEN,
        sparkline_net: BLUE,
        sparkline_gpu: MAGENTA,
        sparkline_sensor: TEAL,
        status_bar_bg: BG_HIGHLIGHT,
        status_bar_fg: FG,
        status_bar_accent: BLUE,
        selected_bg: BG_HIGHLIGHT,
        selected_fg: FG,
        dialog_border: BLUE,
        dialog_bg: BG,
    }
}

fn one_dark_theme() -> Theme {
    const BG: Color = Color::Rgb(40, 44, 52);
    const FG: Color = Color::Rgb(171, 178, 191);
    const BLUE: Color = Color::Rgb(97, 175, 239);
    const GREEN: Color = Color::Rgb(152, 195, 121);
    const YELLOW: Color = Color::Rgb(229, 192, 123);
    const RED: Color = Color::Rgb(224, 108, 117);
    const CYAN: Color = Color::Rgb(86, 182, 194);
    const MAGENTA: Color = Color::Rgb(198, 120, 221);
    const ORANGE: Color = Color::Rgb(209, 154, 102);
    const COMMENT: Color = Color::Rgb(92, 99, 112);
    const GUTTER: Color = Color::Rgb(76, 82, 99);

    Theme {
        name: "one-dark",
        bg: BG,
        fg: FG,
        accent: BLUE,
        cpu_border: BLUE,
        mem_border: GREEN,
        net_border: YELLOW,
        disk_border: CYAN,
        gpu_border: MAGENTA,
        sensor_border: ORANGE,
        battery_border: GREEN,
        docker_border: CYAN,
        k8s_border: BLUE,
        remote_border: MAGENTA,
        good: GREEN,
        warning: YELLOW,
        critical: RED,
        text_primary: FG,
        text_secondary: COMMENT,
        bar_filled: BLUE,
        bar_empty: GUTTER,
        graph_line: CYAN,
        sparkline_cpu: RED,
        sparkline_mem: GREEN,
        sparkline_net: BLUE,
        sparkline_gpu: MAGENTA,
        sparkline_sensor: CYAN,
        status_bar_bg: Color::Rgb(33, 37, 43),
        status_bar_fg: FG,
        status_bar_accent: BLUE,
        selected_bg: GUTTER,
        selected_fg: FG,
        dialog_border: BLUE,
        dialog_bg: BG,
    }
}

fn monokai_theme() -> Theme {
    const BG: Color = Color::Rgb(39, 40, 34);
    const FG: Color = Color::Rgb(248, 248, 242);
    const CYAN: Color = Color::Rgb(102, 217, 239);
    const GREEN: Color = Color::Rgb(166, 226, 46);
    const ORANGE: Color = Color::Rgb(253, 151, 31);
    const RED_PINK: Color = Color::Rgb(249, 38, 114);
    const PURPLE: Color = Color::Rgb(174, 129, 255);
    const YELLOW: Color = Color::Rgb(230, 219, 116);
    const COMMENT: Color = Color::Rgb(117, 113, 94);
    const GUTTER: Color = Color::Rgb(73, 72, 62);

    Theme {
        name: "monokai",
        bg: BG,
        fg: FG,
        accent: CYAN,
        cpu_border: CYAN,
        mem_border: GREEN,
        net_border: ORANGE,
        disk_border: PURPLE,
        gpu_border: RED_PINK,
        sensor_border: YELLOW,
        battery_border: GREEN,
        docker_border: CYAN,
        k8s_border: PURPLE,
        remote_border: CYAN,
        good: GREEN,
        warning: ORANGE,
        critical: RED_PINK,
        text_primary: FG,
        text_secondary: COMMENT,
        bar_filled: GREEN,
        bar_empty: GUTTER,
        graph_line: CYAN,
        sparkline_cpu: RED_PINK,
        sparkline_mem: GREEN,
        sparkline_net: CYAN,
        sparkline_gpu: PURPLE,
        sparkline_sensor: YELLOW,
        status_bar_bg: GUTTER,
        status_bar_fg: FG,
        status_bar_accent: CYAN,
        selected_bg: GUTTER,
        selected_fg: FG,
        dialog_border: CYAN,
        dialog_bg: BG,
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Get all built-in theme names.
pub fn builtin_theme_names() -> Vec<&'static str> {
    builtin_themes().iter().map(|t| t.name).collect()
}

/// Look up a built-in theme by name (case-insensitive).
pub fn get_builtin_theme(name: &str) -> Option<Theme> {
    let lower = name.to_lowercase();
    builtin_themes().into_iter().find(|t| t.name == lower)
}

/// Get all built-in themes.
pub fn builtin_themes() -> Vec<Theme> {
    vec![
        default_theme(),
        dracula_theme(),
        gruvbox_dark_theme(),
        catppuccin_mocha_theme(),
        catppuccin_latte_theme(),
        nord_theme(),
        solarized_dark_theme(),
        solarized_light_theme(),
        tokyo_night_theme(),
        one_dark_theme(),
        monokai_theme(),
    ]
}

// ---------------------------------------------------------------------------
// TOML theme file support
// ---------------------------------------------------------------------------

/// TOML-serializable theme definition.
/// Uses hex color strings like `"#282a36"` or named colors like `"red"`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ThemeFile {
    pub name: String,
    pub colors: ThemeColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ThemeColors {
    pub bg: String,
    pub fg: String,
    pub accent: String,

    pub cpu_border: String,
    pub mem_border: String,
    pub net_border: String,
    pub disk_border: String,
    pub gpu_border: String,
    pub sensor_border: String,
    pub battery_border: String,
    pub docker_border: String,
    pub k8s_border: String,
    pub remote_border: String,

    pub good: String,
    pub warning: String,
    pub critical: String,

    pub text_primary: String,
    pub text_secondary: String,
    pub bar_filled: String,
    pub bar_empty: String,
    pub graph_line: String,
    pub sparkline_cpu: String,
    pub sparkline_mem: String,
    pub sparkline_net: String,
    pub sparkline_gpu: String,
    pub sparkline_sensor: String,

    pub status_bar_bg: String,
    pub status_bar_fg: String,
    pub status_bar_accent: String,

    pub selected_bg: String,
    pub selected_fg: String,

    pub dialog_border: String,
    pub dialog_bg: String,
}

/// Parse a color string into a ratatui `Color`.
///
/// Supports hex colors (`"#282a36"`) and named colors (`"red"`, `"reset"`, etc.).
#[allow(dead_code)]
pub fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim();

    // Hex color
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(hex.get(0..2)?, 16).ok()?;
        let g = u8::from_str_radix(hex.get(2..4)?, 16).ok()?;
        let b = u8::from_str_radix(hex.get(4..6)?, 16).ok()?;
        return Some(Color::Rgb(r, g, b));
    }

    // Named colors (case-insensitive)
    match s.to_lowercase().as_str() {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "white" => Some(Color::White),
        "gray" | "grey" => Some(Color::Gray),
        "darkgray" | "darkgrey" | "dark_gray" | "dark_grey" => Some(Color::DarkGray),
        "lightred" | "light_red" => Some(Color::LightRed),
        "lightgreen" | "light_green" => Some(Color::LightGreen),
        "lightyellow" | "light_yellow" => Some(Color::LightYellow),
        "lightblue" | "light_blue" => Some(Color::LightBlue),
        "lightmagenta" | "light_magenta" => Some(Color::LightMagenta),
        "lightcyan" | "light_cyan" => Some(Color::LightCyan),
        "reset" | "default" => Some(Color::Reset),
        _ => None,
    }
}

/// Load a TOML theme file and convert it into a runtime `Theme`.
///
/// The returned theme's `name` field is leaked into a `&'static str` so it can
/// be stored alongside built-in themes. This is intentional — theme loading
/// happens once at startup so the small allocation is acceptable.
#[allow(dead_code)]
pub fn load_theme_file(path: &Path) -> anyhow::Result<Theme> {
    let contents = std::fs::read_to_string(path)?;
    let file: ThemeFile = toml::from_str(&contents)?;
    theme_from_file(file)
}

#[allow(dead_code)]
fn require_color(s: &str, field: &str) -> anyhow::Result<Color> {
    parse_color(s).ok_or_else(|| anyhow::anyhow!("invalid color for '{field}': {s}"))
}

#[allow(dead_code)]
fn theme_from_file(file: ThemeFile) -> anyhow::Result<Theme> {
    let c = &file.colors;
    Ok(Theme {
        // Leak the name string so it lives for 'static — this only happens
        // once per loaded theme file so the leak is negligible.
        name: Box::leak(file.name.into_boxed_str()),
        bg: require_color(&c.bg, "bg")?,
        fg: require_color(&c.fg, "fg")?,
        accent: require_color(&c.accent, "accent")?,
        cpu_border: require_color(&c.cpu_border, "cpu_border")?,
        mem_border: require_color(&c.mem_border, "mem_border")?,
        net_border: require_color(&c.net_border, "net_border")?,
        disk_border: require_color(&c.disk_border, "disk_border")?,
        gpu_border: require_color(&c.gpu_border, "gpu_border")?,
        sensor_border: require_color(&c.sensor_border, "sensor_border")?,
        battery_border: require_color(&c.battery_border, "battery_border")?,
        docker_border: require_color(&c.docker_border, "docker_border")?,
        k8s_border: require_color(&c.k8s_border, "k8s_border")?,
        remote_border: require_color(&c.remote_border, "remote_border")?,
        good: require_color(&c.good, "good")?,
        warning: require_color(&c.warning, "warning")?,
        critical: require_color(&c.critical, "critical")?,
        text_primary: require_color(&c.text_primary, "text_primary")?,
        text_secondary: require_color(&c.text_secondary, "text_secondary")?,
        bar_filled: require_color(&c.bar_filled, "bar_filled")?,
        bar_empty: require_color(&c.bar_empty, "bar_empty")?,
        graph_line: require_color(&c.graph_line, "graph_line")?,
        sparkline_cpu: require_color(&c.sparkline_cpu, "sparkline_cpu")?,
        sparkline_mem: require_color(&c.sparkline_mem, "sparkline_mem")?,
        sparkline_net: require_color(&c.sparkline_net, "sparkline_net")?,
        sparkline_gpu: require_color(&c.sparkline_gpu, "sparkline_gpu")?,
        sparkline_sensor: require_color(&c.sparkline_sensor, "sparkline_sensor")?,
        status_bar_bg: require_color(&c.status_bar_bg, "status_bar_bg")?,
        status_bar_fg: require_color(&c.status_bar_fg, "status_bar_fg")?,
        status_bar_accent: require_color(&c.status_bar_accent, "status_bar_accent")?,
        selected_bg: require_color(&c.selected_bg, "selected_bg")?,
        selected_fg: require_color(&c.selected_fg, "selected_fg")?,
        dialog_border: require_color(&c.dialog_border, "dialog_border")?,
        dialog_bg: require_color(&c.dialog_bg, "dialog_bg")?,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn default_theme_matches_expectations() {
        let t = default_theme();
        assert_eq!(t.name, "default");
        assert_eq!(t.bg, Color::Reset);
        assert_eq!(t.fg, Color::White);
        assert_eq!(t.accent, Color::Cyan);
        assert_eq!(t.cpu_border, Color::Cyan);
        assert_eq!(t.mem_border, Color::Green);
        assert_eq!(t.net_border, Color::Yellow);
        assert_eq!(t.disk_border, Color::Blue);
        assert_eq!(t.good, Color::Green);
        assert_eq!(t.warning, Color::Yellow);
        assert_eq!(t.critical, Color::Red);
        assert_eq!(t.text_primary, Color::White);
        assert_eq!(t.text_secondary, Color::DarkGray);
        assert_eq!(t.sparkline_cpu, Color::Red);
        assert_eq!(t.sparkline_mem, Color::Green);
        assert_eq!(t.status_bar_bg, Color::DarkGray);
        assert_eq!(t.dialog_border, Color::Cyan);
        assert_eq!(t.dialog_bg, Color::Reset);
    }

    #[test]
    fn builtin_theme_count() {
        assert_eq!(builtin_themes().len(), 11);
    }

    #[test]
    fn get_builtin_by_name() {
        let t = get_builtin_theme("dracula");
        assert!(t.is_some());
        assert_eq!(t.as_ref().unwrap().name, "dracula");

        // Case-insensitive
        let t2 = get_builtin_theme("Dracula");
        assert!(t2.is_some());
        assert_eq!(t2.as_ref().unwrap().name, "dracula");

        let t3 = get_builtin_theme("NORD");
        assert!(t3.is_some());
        assert_eq!(t3.as_ref().unwrap().name, "nord");

        let t4 = get_builtin_theme("Tokyo-Night");
        assert!(t4.is_some());
        assert_eq!(t4.as_ref().unwrap().name, "tokyo-night");
    }

    #[test]
    fn get_builtin_unknown_returns_none() {
        assert!(get_builtin_theme("nonexistent").is_none());
        assert!(get_builtin_theme("").is_none());
    }

    #[test]
    fn parse_color_hex() {
        assert_eq!(parse_color("#282a36"), Some(Color::Rgb(40, 42, 54)));
        assert_eq!(parse_color("#000000"), Some(Color::Rgb(0, 0, 0)));
        assert_eq!(parse_color("#ffffff"), Some(Color::Rgb(255, 255, 255)));
        assert_eq!(parse_color("#FF0000"), Some(Color::Rgb(255, 0, 0)));

        // Invalid hex
        assert_eq!(parse_color("#zzzzzz"), None);
        assert_eq!(parse_color("#12"), None);
        assert_eq!(parse_color("#1234567"), None);
    }

    #[test]
    fn parse_color_named() {
        assert_eq!(parse_color("red"), Some(Color::Red));
        assert_eq!(parse_color("Green"), Some(Color::Green));
        assert_eq!(parse_color("CYAN"), Some(Color::Cyan));
        assert_eq!(parse_color("white"), Some(Color::White));
        assert_eq!(parse_color("reset"), Some(Color::Reset));
        assert_eq!(parse_color("darkgray"), Some(Color::DarkGray));
        assert_eq!(parse_color("dark_gray"), Some(Color::DarkGray));
        assert_eq!(parse_color("light_blue"), Some(Color::LightBlue));
        assert_eq!(parse_color("default"), Some(Color::Reset));

        // Unknown name
        assert_eq!(parse_color("foobar"), None);
    }

    #[test]
    fn all_themes_have_unique_names() {
        let themes = builtin_themes();
        let mut names = HashSet::new();
        for theme in &themes {
            assert!(
                names.insert(theme.name),
                "duplicate theme name: {}",
                theme.name
            );
        }
        assert_eq!(names.len(), themes.len());
    }

    #[test]
    fn builtin_theme_names_matches_themes() {
        let names = builtin_theme_names();
        let themes = builtin_themes();
        assert_eq!(names.len(), themes.len());
        for (name, theme) in names.iter().zip(themes.iter()) {
            assert_eq!(*name, theme.name);
        }
    }

    #[test]
    fn theme_file_roundtrip() {
        let toml_str = r##"
name = "custom-test"

[colors]
bg = "#1e1e2e"
fg = "#cdd6f4"
accent = "#89b4fa"
cpu_border = "#89b4fa"
mem_border = "#a6e3a1"
net_border = "#f9e2af"
disk_border = "#cba6f7"
gpu_border = "#94e2d5"
sensor_border = "#fab387"
battery_border = "#a6e3a1"
docker_border = "#94e2d5"
k8s_border = "#89b4fa"
remote_border = "#cba6f7"
good = "#a6e3a1"
warning = "#f9e2af"
critical = "#f38ba8"
text_primary = "#cdd6f4"
text_secondary = "#6c7086"
bar_filled = "#89b4fa"
bar_empty = "#313244"
graph_line = "#94e2d5"
sparkline_cpu = "#f38ba8"
sparkline_mem = "#a6e3a1"
sparkline_net = "#89b4fa"
sparkline_gpu = "#cba6f7"
sparkline_sensor = "#fab387"
status_bar_bg = "#313244"
status_bar_fg = "#cdd6f4"
status_bar_accent = "#89b4fa"
selected_bg = "#313244"
selected_fg = "#cdd6f4"
dialog_border = "#f5c2e7"
dialog_bg = "#1e1e2e"
"##;

        let file: ThemeFile = toml::from_str(toml_str).expect("should parse TOML");
        let theme = theme_from_file(file).expect("should convert to Theme");

        assert_eq!(theme.name, "custom-test");
        assert_eq!(theme.bg, Color::Rgb(30, 30, 46));
        assert_eq!(theme.accent, Color::Rgb(137, 180, 250));
        assert_eq!(theme.critical, Color::Rgb(243, 139, 168));
    }

    #[test]
    fn theme_file_with_named_colors() {
        let toml_str = r#"
name = "named-colors"

[colors]
bg = "reset"
fg = "white"
accent = "cyan"
cpu_border = "cyan"
mem_border = "green"
net_border = "yellow"
disk_border = "blue"
gpu_border = "blue"
sensor_border = "magenta"
battery_border = "green"
docker_border = "cyan"
k8s_border = "cyan"
remote_border = "magenta"
good = "green"
warning = "yellow"
critical = "red"
text_primary = "white"
text_secondary = "darkgray"
bar_filled = "green"
bar_empty = "darkgray"
graph_line = "cyan"
sparkline_cpu = "red"
sparkline_mem = "green"
sparkline_net = "blue"
sparkline_gpu = "magenta"
sparkline_sensor = "magenta"
status_bar_bg = "darkgray"
status_bar_fg = "white"
status_bar_accent = "cyan"
selected_bg = "darkgray"
selected_fg = "white"
dialog_border = "cyan"
dialog_bg = "reset"
"#;

        let file: ThemeFile = toml::from_str(toml_str).expect("should parse TOML");
        let theme = theme_from_file(file).expect("should convert to Theme");

        assert_eq!(theme.bg, Color::Reset);
        assert_eq!(theme.fg, Color::White);
        assert_eq!(theme.accent, Color::Cyan);
    }

    #[test]
    fn theme_file_invalid_color_returns_error() {
        let toml_str = r#"
name = "bad"

[colors]
bg = "not_a_color"
fg = "white"
accent = "cyan"
cpu_border = "cyan"
mem_border = "green"
net_border = "yellow"
disk_border = "blue"
gpu_border = "blue"
sensor_border = "magenta"
battery_border = "green"
docker_border = "cyan"
k8s_border = "cyan"
remote_border = "magenta"
good = "green"
warning = "yellow"
critical = "red"
text_primary = "white"
text_secondary = "darkgray"
bar_filled = "green"
bar_empty = "darkgray"
graph_line = "cyan"
sparkline_cpu = "red"
sparkline_mem = "green"
sparkline_net = "blue"
sparkline_gpu = "magenta"
sparkline_sensor = "magenta"
status_bar_bg = "darkgray"
status_bar_fg = "white"
status_bar_accent = "cyan"
selected_bg = "darkgray"
selected_fg = "white"
dialog_border = "cyan"
dialog_bg = "reset"
"#;

        let file: ThemeFile = toml::from_str(toml_str).expect("should parse TOML");
        let result = theme_from_file(file);
        assert!(result.is_err());
    }
}
