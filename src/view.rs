use std::fs;
use std::io::{Write, stdout};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{Event, KeyCode, KeyEventKind, KeyModifiers, read},
    execute, queue,
    style::{Attribute, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};

use crate::config::Config;
use crate::search::{Match, assign_labels, find};

/// Interactive UI. Runs inside the swapped-in tmux pane (real pty), reads the
/// captured pane text from `content_path`, and writes the chosen `row col` to
/// `out_path` (empty on abort).
pub fn run(content_path: &str, out_path: &str) {
    let cfg = Config::from_env();
    let content = fs::read_to_string(content_path).unwrap_or_default();
    let grid: Vec<Vec<char>> = content.lines().map(|l| l.chars().collect()).collect();

    enable_raw_mode().ok();
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, Hide).ok();

    let result = event_loop(&mut out, &grid, &cfg);

    execute!(out, Show, LeaveAlternateScreen).ok();
    disable_raw_mode().ok();

    let payload = match result {
        Some((row, col)) => format!("{} {}", row, col),
        None => String::new(),
    };
    fs::write(out_path, payload).ok();
}

fn event_loop(out: &mut impl Write, grid: &[Vec<char>], cfg: &Config) -> Option<(usize, usize)> {
    let mut query: Vec<char> = Vec::new();
    let mut pending = String::new();

    loop {
        let mut matches = find(grid, &query);
        if cfg.autojump && matches.len() == 1 {
            let m = &matches[0];
            return Some((m.row, m.col));
        }
        if query.len() >= cfg.min_pattern_length.max(1) {
            matches = assign_labels(grid, matches, &cfg.labels);
        }
        draw(out, grid, &matches, &query, &pending, cfg);

        let ev = match read() {
            Ok(Event::Key(k)) if k.kind == KeyEventKind::Press => k,
            Ok(_) => continue,
            Err(_) => return None,
        };

        match ev.code {
            KeyCode::Esc => return None,
            KeyCode::Char('c') if ev.modifiers.contains(KeyModifiers::CONTROL) => return None,
            KeyCode::Enter => {
                if let Some(m) = matches.first() {
                    return Some((m.row, m.col));
                }
            }
            KeyCode::Backspace => {
                if pending.pop().is_none() {
                    query.pop();
                }
            }
            KeyCode::Char(c) => {
                let cand = format!("{}{}", pending, c);
                if let Some(m) = matches
                    .iter()
                    .find(|m| m.label.as_deref() == Some(cand.as_str()))
                {
                    return Some((m.row, m.col));
                }
                let is_prefix = matches
                    .iter()
                    .any(|m| m.label.as_deref().is_some_and(|l| l.starts_with(&cand)));
                if is_prefix {
                    pending = cand;
                } else if pending.is_empty() {
                    query.push(c);
                } else {
                    // Dead-end label prefix: drop it and treat this key fresh.
                    pending.clear();
                    if matches
                        .iter()
                        .all(|m| m.label.as_deref() != Some(&c.to_string()[..]))
                    {
                        query.push(c);
                    }
                }
            }
            _ => {}
        }
    }
}

fn draw(
    out: &mut impl Write,
    grid: &[Vec<char>],
    matches: &[Match],
    query: &[char],
    pending: &str,
    cfg: &Config,
) {
    queue!(out, Clear(ClearType::All)).ok();

    for (row, chars) in grid.iter().enumerate() {
        let line: String = chars.iter().collect();
        queue!(
            out,
            MoveTo(0, row as u16),
            SetForegroundColor(cfg.backdrop_fg),
            Print(line),
            ResetColor
        )
        .ok();
    }

    for (i, m) in matches.iter().enumerate() {
        let text: String = grid[m.row][m.col..m.col + m.len].iter().collect();
        // First match is the Enter target; give it its own color.
        let fg = if i == 0 { cfg.current_fg } else { cfg.match_fg };
        queue!(
            out,
            MoveTo(m.col as u16, m.row as u16),
            SetForegroundColor(fg),
            SetAttribute(Attribute::Bold),
            Print(&text),
            SetAttribute(Attribute::Reset)
        )
        .ok();
        if let Some(label) = &m.label {
            // Draw only the still-selectable suffix of the label so 2-char
            // narrowing shows what to press next.
            let shown = label.strip_prefix(pending).unwrap_or(label);
            if !shown.is_empty() && (pending.is_empty() || label.starts_with(pending)) {
                queue!(
                    out,
                    MoveTo(m.col as u16, m.row as u16),
                    SetForegroundColor(cfg.label_fg),
                    SetBackgroundColor(cfg.label_bg),
                    Print(shown),
                    ResetColor
                )
                .ok();
            }
        }
    }

    // Query echo, bottom-left.
    let rows = grid.len().max(1) as u16;
    let q: String = query.iter().collect();
    queue!(
        out,
        MoveTo(0, rows.saturating_sub(1)),
        SetForegroundColor(cfg.query_fg),
        Print(format!("/{}", q)),
        ResetColor
    )
    .ok();

    out.flush().ok();
}
