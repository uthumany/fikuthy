use crossterm::terminal;
use std::io::{self, IsTerminal, Write};

const WORD: &str = "UTHARNESS";
const COLORS: [(u8, u8, u8); 9] = [
    (180, 76, 255),
    (32, 214, 244),
    (85, 219, 36),
    (255, 210, 31),
    (255, 138, 22),
    (255, 61, 79),
    (52, 120, 246),
    (180, 76, 255),
    (180, 76, 255),
];
const GLYPHS: [[&str; 3]; 8] = [
    ["█ █", "█ █", "███"],
    ["███", " █ ", " █ "],
    ["█ █", "███", "█ █"],
    [" █ ", "███", "█ █"],
    ["██ ", "██ ", "█ █"],
    ["█ █", "███", "█ █"],
    ["███", "██ ", "███"],
    [" ██", " █ ", "██ "],
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BannerLayout {
    Full,
    Compressed,
    Wrapped,
    Compact,
    Minimal,
    Hidden,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BannerPreference {
    Full,
    Compact,
    Minimal,
    Hidden,
}
impl BannerPreference {
    pub fn from_environment() -> Self {
        match std::env::var("UTHARNESS_BANNER")
            .unwrap_or_else(|_| "full".into())
            .to_ascii_lowercase()
            .as_str()
        {
            "hide" | "hidden" | "off" | "false" => Self::Hidden,
            "minimal" => Self::Minimal,
            "compact" => Self::Compact,
            _ => Self::Full,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ColorDepth {
    None,
    Ansi16,
    Ansi256,
    TrueColor,
}

pub fn terminal_width() -> u16 {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .or_else(|| terminal::size().ok().map(|(w, _)| w))
        .unwrap_or(80)
        .max(1)
}
pub fn layout_for_width(width: u16, preference: BannerPreference) -> BannerLayout {
    if preference == BannerPreference::Hidden {
        return BannerLayout::Hidden;
    }
    if width < 40 || preference == BannerPreference::Minimal {
        return BannerLayout::Minimal;
    }
    if width < 60 || preference == BannerPreference::Compact {
        return BannerLayout::Compact;
    }
    if width < 90 {
        BannerLayout::Wrapped
    } else if width < 120 {
        BannerLayout::Compressed
    } else {
        BannerLayout::Full
    }
}
fn color_depth(ansi: bool) -> ColorDepth {
    if !ansi
        || std::env::var_os("NO_COLOR").is_some()
        || std::env::var("TERM").is_ok_and(|v| v == "dumb")
    {
        return ColorDepth::None;
    }
    if std::env::var("COLORTERM").is_ok_and(|v| v.contains("truecolor") || v.contains("24bit"))
        || std::env::var("UTHARNESS_COLOR").is_ok_and(|v| v == "truecolor")
    {
        ColorDepth::TrueColor
    } else if std::env::var("TERM").is_ok_and(|v| v.contains("256color"))
        || std::env::var("UTHARNESS_COLOR").is_ok_and(|v| v == "ansi256")
    {
        ColorDepth::Ansi256
    } else {
        ColorDepth::Ansi16
    }
}
fn start_color(index: usize, depth: ColorDepth) -> String {
    let (r, g, b) = COLORS[index.min(8)];
    match depth {
        ColorDepth::None => String::new(),
        ColorDepth::TrueColor => format!("\x1b[38;2;{r};{g};{b}m"),
        ColorDepth::Ansi256 => format!(
            "\x1b[38;5;{}m",
            [129, 45, 40, 220, 208, 203, 69, 129, 129][index.min(8)]
        ),
        ColorDepth::Ansi16 => format!(
            "\x1b[{}m",
            [35, 36, 32, 33, 33, 31, 34, 35, 35][index.min(8)]
        ),
    }
}
fn reset(depth: ColorDepth) -> &'static str {
    if depth == ColorDepth::None {
        ""
    } else {
        "\x1b[0m"
    }
}
fn paint(text: &str, index: usize, depth: ColorDepth) -> String {
    format!("{}{text}{}", start_color(index, depth), reset(depth))
}
fn colored_word(depth: ColorDepth) -> String {
    WORD.chars()
        .enumerate()
        .map(|(i, c)| paint(&c.to_string(), i, depth))
        .collect()
}
fn art_line(row: usize, depth: ColorDepth, doubled: bool) -> String {
    WORD.chars()
        .enumerate()
        .map(|(i, c)| {
            let g = match c {
                'U' => 0,
                'T' => 1,
                'H' => 2,
                'A' => 3,
                'R' => 4,
                'N' => 5,
                'E' => 6,
                _ => 7,
            };
            let s = if doubled {
                GLYPHS[g][row].replace('█', "██")
            } else {
                GLYPHS[g][row].into()
            };
            format!("{}{} ", start_color(i, depth), s)
        })
        .collect::<String>()
        + reset(depth)
}
fn icons(ascii: bool) -> [&'static str; 7] {
    if ascii {
        ["[A]", "[M]", "[S]", "[C]", "[D]", "[T]", ">_"]
    } else {
        ["◉", "◇", "</>", "⎇", "▤", "⚒", ">_"]
    }
}

fn terminal_block_line(row: usize, depth: ColorDepth, ascii: bool) -> String {
    let (top, side, bottom) = if ascii {
        ("+----------+", "|", "+----------+")
    } else {
        ("╔══════════╗", "║", "╚══════════╝")
    };
    match row {
        0 => paint(top, 0, depth),
        1 => format!(
            "{}{} {}>_{}      {}{}",
            start_color(0, depth),
            side,
            start_color(2, depth),
            start_color(0, depth),
            side,
            reset(depth)
        ),
        _ => paint(bottom, 0, depth),
    }
}

pub fn render_banner(width: u16, version: &str, ansi: bool) -> String {
    render_banner_with(width, version, ansi, BannerPreference::from_environment())
}
pub fn render_banner_with(
    width: u16,
    version: &str,
    ansi: bool,
    preference: BannerPreference,
) -> String {
    let layout = layout_for_width(width, preference);
    if layout == BannerLayout::Hidden {
        return String::new();
    }
    let depth = color_depth(ansi);
    let ascii = std::env::var_os("UTHARNESS_ASCII").is_some()
        || std::env::var("TERM").is_ok_and(|v| v == "dumb");
    let mut lines = Vec::new();
    match layout {
        BannerLayout::Minimal => lines.push(format!(
            "{} {} v{version}",
            colored_word(depth),
            paint(">_", 2, depth)
        )),
        BannerLayout::Compact => {
            lines.push(format!(
                "+- {} {} -+",
                colored_word(depth),
                paint(">_", 2, depth)
            ));
            lines.push("[A] [M] [S] [C] [D] [T] >_".into());
            lines.push("AUTONOMOUS AGENT HARNESS".into());
        }
        BannerLayout::Wrapped | BannerLayout::Compressed | BannerLayout::Full => {
            let n = usize::min(
                width.saturating_sub(2) as usize,
                if layout == BannerLayout::Full {
                    104
                } else {
                    82
                },
            );
            lines.push("-".repeat(n));
            for row in 0..3 {
                let wordmark = art_line(row, depth, layout == BannerLayout::Full);
                lines.push(if layout == BannerLayout::Full {
                    format!("{}  {wordmark}", terminal_block_line(row, depth, ascii))
                } else {
                    wordmark
                });
            }
            lines.push("-".repeat(n));
            let names = [
                "AGENTS", "MODELS", "SKILLS", "MCP", "MEMORY", "TOOLS", "TERMINAL",
            ];
            let glyphs = icons(ascii);
            let blocks = names
                .iter()
                .enumerate()
                .map(|(i, name)| paint(&format!("{} {name}", glyphs[i]), i, depth))
                .collect::<Vec<_>>();
            if layout == BannerLayout::Wrapped {
                lines.push(blocks[..4].join("  "));
                lines.push(blocks[4..].join("  "));
            } else {
                lines.push(blocks.join("  "));
            }
            lines.push(format!(
                "{} AUTONOMOUS AI AGENT TERMINAL HARNESS {} v{version}",
                paint(">", 2, depth),
                paint("<", 2, depth)
            ));
        }
        BannerLayout::Hidden => {}
    }
    lines.into_iter().map(|line| format!("{line}\n")).collect()
}
pub fn print_onboarding_tips() -> anyhow::Result<()> {
    let mut out = io::stdout();
    writeln!(
        out,
        "Tips: ask questions · use @file for context · /help for commands"
    )?;
    out.flush()?;
    Ok(())
}
pub fn print_startup_banner(version: &str) -> anyhow::Result<()> {
    let mut out = io::stdout();
    if !out.is_terminal() {
        return Ok(());
    }
    write!(out, "{}", render_banner(terminal_width(), version, true))?;
    out.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn plain(w: u16) -> String {
        render_banner_with(w, "0.2.13", false, BannerPreference::Full)
    }
    #[test]
    fn maps_required_widths() {
        assert_eq!(
            [20, 30, 40, 60, 80, 100, 120, 160, 200]
                .map(|w| layout_for_width(w, BannerPreference::Full)),
            [
                BannerLayout::Minimal,
                BannerLayout::Minimal,
                BannerLayout::Compact,
                BannerLayout::Wrapped,
                BannerLayout::Wrapped,
                BannerLayout::Compressed,
                BannerLayout::Full,
                BannerLayout::Full,
                BannerLayout::Full
            ]
        );
    }
    #[test]
    fn every_plain_line_fits() {
        for w in [20, 30, 40, 60, 80, 100, 120, 160, 200] {
            assert!(
                plain(w)
                    .lines()
                    .all(|line| line.chars().count() <= w as usize),
                "overflow at {w}"
            );
        }
    }
    #[test]
    fn hierarchy_is_present() {
        let o = plain(120);
        assert!(o.contains("AGENTS"));
        assert!(o.contains("MODELS"));
        assert!(o.contains("AUTONOMOUS AI AGENT TERMINAL HARNESS"));
    }
    #[test]
    fn minimal_never_disappears() {
        assert!(plain(20).contains("UTHARNESS"));
        assert!(render_banner_with(120, "x", false, BannerPreference::Hidden).is_empty());
    }
    #[test]
    fn truecolor_is_per_letter() {
        assert!(start_color(0, ColorDepth::TrueColor).contains("38;2;180;76;255"));
        assert!(start_color(1, ColorDepth::TrueColor).contains("38;2;32;214;244"));
    }
}
