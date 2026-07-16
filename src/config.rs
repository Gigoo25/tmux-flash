use crossterm::style::Color;

/// tmux user options (`set -g @flash-*`), read by the orchestrator and handed
/// to the view process as TMUX_FLASH_* env vars on the helper window.
pub struct Config {
    pub labels: Vec<char>,
    pub autojump: bool,
    pub min_pattern_length: usize,
    pub label_fg: Color,
    pub label_bg: Color,
    pub match_fg: Color,
    pub current_fg: Color,
    pub backdrop_fg: Color,
    pub query_fg: Color,
}

pub const DEFAULT_LABELS: &str = "asdfjklghqwertyuiopzxcvbnm";

/// Option names as (tmux user option, env var) pairs, in the order the
/// orchestrator queries them.
pub const OPTIONS: &[(&str, &str)] = &[
    ("@flash-labels", "TMUX_FLASH_LABELS"),
    ("@flash-label-exclude", "TMUX_FLASH_LABEL_EXCLUDE"),
    ("@flash-autojump", "TMUX_FLASH_AUTOJUMP"),
    ("@flash-min-pattern-length", "TMUX_FLASH_MIN_PATTERN_LENGTH"),
    ("@flash-label-fg", "TMUX_FLASH_LABEL_FG"),
    ("@flash-label-bg", "TMUX_FLASH_LABEL_BG"),
    ("@flash-match-fg", "TMUX_FLASH_MATCH_FG"),
    ("@flash-current-fg", "TMUX_FLASH_CURRENT_FG"),
    ("@flash-backdrop-fg", "TMUX_FLASH_BACKDROP_FG"),
    ("@flash-query-fg", "TMUX_FLASH_QUERY_FG"),
];

impl Config {
    pub fn from_env() -> Config {
        let get = |var: &str| std::env::var(var).unwrap_or_default();

        let mut labels: Vec<char> = {
            let s = get("TMUX_FLASH_LABELS");
            if s.is_empty() {
                DEFAULT_LABELS.to_string()
            } else {
                s
            }
            .chars()
            .collect()
        };
        let exclude = get("TMUX_FLASH_LABEL_EXCLUDE");
        labels.retain(|c| !exclude.contains(*c));

        let color = |var: &str, default: Color| parse_color(&get(var)).unwrap_or(default);

        Config {
            labels,
            autojump: !matches!(get("TMUX_FLASH_AUTOJUMP").as_str(), "0" | "off" | "false"),
            min_pattern_length: get("TMUX_FLASH_MIN_PATTERN_LENGTH").parse().unwrap_or(0),
            label_fg: color("TMUX_FLASH_LABEL_FG", Color::Black),
            label_bg: color("TMUX_FLASH_LABEL_BG", Color::Red),
            match_fg: color("TMUX_FLASH_MATCH_FG", Color::White),
            current_fg: color("TMUX_FLASH_CURRENT_FG", Color::Green),
            backdrop_fg: color("TMUX_FLASH_BACKDROP_FG", Color::DarkGrey),
            query_fg: color("TMUX_FLASH_QUERY_FG", Color::Yellow),
        }
    }
}

/// Accepts "#rrggbb", tmux-style "colour255"/"color255", a bare 0-255 index,
/// or a basic color name.
pub fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 6 {
            let v = u32::from_str_radix(hex, 16).ok()?;
            return Some(Color::Rgb {
                r: (v >> 16) as u8,
                g: (v >> 8) as u8,
                b: v as u8,
            });
        }
        return None;
    }
    let idx = s
        .strip_prefix("colour")
        .or_else(|| s.strip_prefix("color"))
        .unwrap_or(s);
    if let Ok(n) = idx.parse::<u8>() {
        return Some(Color::AnsiValue(n));
    }
    match s.to_ascii_lowercase().as_str() {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "white" => Some(Color::White),
        "darkred" => Some(Color::DarkRed),
        "darkgreen" => Some(Color::DarkGreen),
        "darkyellow" => Some(Color::DarkYellow),
        "darkblue" => Some(Color::DarkBlue),
        "darkmagenta" => Some(Color::DarkMagenta),
        "darkcyan" => Some(Color::DarkCyan),
        "darkgrey" | "darkgray" => Some(Color::DarkGrey),
        "grey" | "gray" => Some(Color::Grey),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_index_and_names() {
        assert_eq!(
            parse_color("#ff8000"),
            Some(Color::Rgb {
                r: 255,
                g: 128,
                b: 0
            })
        );
        assert_eq!(parse_color("colour214"), Some(Color::AnsiValue(214)));
        assert_eq!(parse_color("42"), Some(Color::AnsiValue(42)));
        assert_eq!(parse_color("DarkGrey"), Some(Color::DarkGrey));
        assert_eq!(parse_color(""), None);
        assert_eq!(parse_color("bogus"), None);
    }
}
