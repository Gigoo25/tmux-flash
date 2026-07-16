use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

mod config;
mod search;
mod view;

fn tmux(args: &[&str]) -> String {
    let out = Command::new("tmux")
        .args(args)
        .output()
        .expect("failed to run tmux");
    String::from_utf8_lossy(&out.stdout).trim_end().to_string()
}

fn arg_val(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--view") {
        let content = arg_val(&args, "--content").expect("--content required");
        let out = arg_val(&args, "--tmp").expect("--tmp required");
        view::run(&content, &out);
    } else {
        orchestrate();
    }
}

/// Runs on the keybind. Captures the target pane, shows the flash UI in a
/// swapped-in pane (mirroring tmux-thumbs so the UI owns a real pty), then
/// drives copy-mode to the chosen position.
fn orchestrate() {
    let info = tmux(&[
        "display-message",
        "-p",
        "#{pane_id}:#{pane_in_mode}:#{pane_height}:#{scroll_position}:#{window_zoomed_flag}",
    ]);
    let f: Vec<&str> = info.split(':').collect();
    let pane = f[0].to_string();
    let in_mode = f.get(1).is_some_and(|v| *v == "1");
    let height: i32 = f.get(2).and_then(|v| v.parse().ok()).unwrap_or(0);
    let scroll: i32 = f.get(3).and_then(|v| v.parse().ok()).unwrap_or(0);
    let zoomed = f.get(4).is_some_and(|v| *v == "1");

    // Capture the visible region (following scrollback if the pane is scrolled).
    let mut cap: Vec<String> = ["capture-pane", "-t", &pane, "-p"].map(String::from).into();
    if in_mode {
        for a in [
            "-S",
            &(-scroll).to_string(),
            "-E",
            &(height - scroll - 1).to_string(),
        ] {
            cap.push(a.to_string());
        }
    }
    let cap_refs: Vec<&str> = cap.iter().map(String::as_str).collect();
    let content = tmux(&cap_refs);
    // Keep at most `height` physical lines so row indices match copy-mode rows.
    let lines: Vec<&str> = content.lines().collect();
    let start = lines.len().saturating_sub(height.max(0) as usize);
    let content = lines[start..].join("\n");

    let pid = std::process::id();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let dir = std::env::temp_dir();
    let content_file = dir.join(format!("tmux-flash-content-{}", pid));
    let out_file = dir.join(format!("tmux-flash-out-{}", pid));
    fs::write(&content_file, &content).ok();
    fs::write(&out_file, "").ok();

    let exe = std::env::current_exe().unwrap();
    let signal = format!("tmux-flash-{}-{}", pid, now);

    let zoom_cmd = if zoomed {
        format!("tmux resize-pane -t {} -Z; ", pane)
    } else {
        String::new()
    };
    let pane_command = format!(
        "{exe} --view --content {content} --tmp {out}; \
         tmux swap-pane -t {pane}; {zoom}tmux wait-for -S {signal}",
        exe = exe.display(),
        content = content_file.display(),
        out = out_file.display(),
        pane = pane,
        zoom = zoom_cmd,
        signal = signal,
    );

    // Read @flash-* user options in one round trip and forward them to the
    // view process as env vars on the helper window (avoids shell quoting).
    let opt_fmt: String = config::OPTIONS
        .iter()
        .map(|(opt, _)| format!("#{{{}}}", opt))
        .collect::<Vec<_>>()
        .join("\n");
    let opt_vals = tmux(&["display-message", "-p", &opt_fmt]);
    let vals: Vec<&str> = opt_vals.split('\n').collect();

    let mut nw: Vec<String> = [
        "new-window",
        "-P",
        "-F",
        "#{pane_id}",
        "-d",
        "-n",
        "[flash]",
    ]
    .map(String::from)
    .into();
    for (i, (_, var)) in config::OPTIONS.iter().enumerate() {
        let val = vals.get(i).copied().unwrap_or("");
        if !val.is_empty() {
            nw.push("-e".to_string());
            nw.push(format!("{}={}", var, val));
        }
    }
    nw.push(pane_command);
    let nw_refs: Vec<&str> = nw.iter().map(String::as_str).collect();
    let flash_pane = tmux(&nw_refs);
    tmux(&["swap-pane", "-d", "-s", &pane, "-t", &flash_pane]);
    if zoomed {
        tmux(&["resize-pane", "-t", &flash_pane, "-Z"]);
    }
    tmux(&["wait-for", &signal]);

    if let Ok(payload) = fs::read_to_string(&out_file)
        && let Some((row, col)) = parse_target(&payload)
    {
        jump(&pane, row, col);
    }
    fs::remove_file(&content_file).ok();
    fs::remove_file(&out_file).ok();
}

fn parse_target(s: &str) -> Option<(u32, u32)> {
    let mut it = s.split_whitespace();
    let row = it.next()?.parse().ok()?;
    let col = it.next()?.parse().ok()?;
    Some((row, col))
}

fn jump(pane: &str, row: u32, col: u32) {
    tmux(&["copy-mode", "-t", pane]);
    tmux(&["send-keys", "-X", "-t", pane, "top-line"]);
    tmux(&["send-keys", "-X", "-t", pane, "start-of-line"]);
    if row > 0 {
        tmux(&[
            "send-keys",
            "-X",
            "-N",
            &row.to_string(),
            "-t",
            pane,
            "cursor-down",
        ]);
    }
    if col > 0 {
        tmux(&[
            "send-keys",
            "-X",
            "-N",
            &col.to_string(),
            "-t",
            pane,
            "cursor-right",
        ]);
    }
}
