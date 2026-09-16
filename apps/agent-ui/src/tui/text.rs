use super::theme::*;
use ratatui::prelude::*;

pub(super) fn number(n: Option<u64>) -> String {
    let Some(n) = n else {
        return "Not reported".into();
    };
    let raw = n.to_string();
    raw.chars()
        .enumerate()
        .fold(String::new(), |mut s, (i, c)| {
            if i > 0 && (raw.len() - i).is_multiple_of(3) {
                s.push(',');
            }
            s.push(c);
            s
        })
}

pub(super) fn duration(seconds: Option<f64>) -> String {
    seconds
        .map(|s| {
            if s < 60.0 {
                format!("{s:.1}s")
            } else {
                format!("{}m {:02}s", s as u64 / 60, s as u64 % 60)
            }
        })
        .unwrap_or_else(|| "—".into())
}

pub(super) fn safe_text(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_control() || *c == '\n')
        .collect()
}

pub(super) fn section(lines: &mut Vec<Line<'static>>, width: u16) {
    lines.push(Line::default());
    lines.push(Line::from("─".repeat(width as usize)).fg(BORDER));
    lines.push(Line::default());
}
