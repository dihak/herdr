//! Paint a visible-screen ANSI dump into a ratatui buffer without mutating PTYs.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use unicode_width::UnicodeWidthChar;

pub(super) fn paint_ansi(buffer: &mut Buffer, area: Rect, ansi: &str, default: Style) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let fitted = fit_rows(
        parse_ansi_rows(ansi, default),
        area.height as usize,
        area.width as usize,
    );
    for (row_index, row) in fitted.iter().enumerate() {
        let y = area.y.saturating_add(row_index as u16);
        if y >= area.bottom() {
            break;
        }
        let mut x = area.x;
        for (ch, style) in row {
            let width = UnicodeWidthChar::width(*ch).unwrap_or(1) as u16;
            if width == 0 {
                continue;
            }
            if x.saturating_add(width) > area.right() {
                break;
            }
            buffer.set_stringn(x, y, ch.to_string(), width as usize, *style);
            x = x.saturating_add(width);
        }
    }
}

fn parse_ansi_rows(ansi: &str, default: Style) -> Vec<Vec<(char, Style)>> {
    let mut rows = vec![Vec::new()];
    let mut style = default;
    let mut chars = ansi.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\n' => rows.push(Vec::new()),
            '\r' => {}
            '\u{1b}' => match chars.peek().copied() {
                Some('[') => {
                    chars.next();
                    let mut seq = String::new();
                    for next in chars.by_ref() {
                        seq.push(next);
                        if next.is_ascii_alphabetic() {
                            break;
                        }
                    }
                    if seq.ends_with('m') {
                        style = apply_sgr(style, default, &seq[..seq.len() - 1]);
                    }
                }
                Some(']') => {
                    chars.next();
                    while let Some(next) = chars.next() {
                        if next == '\u{7}' {
                            break;
                        }
                        if next == '\u{1b}' && chars.peek() == Some(&'\\') {
                            chars.next();
                            break;
                        }
                    }
                }
                Some(_) => {
                    chars.next();
                }
                None => {}
            },
            ch if !ch.is_control() => {
                if let Some(row) = rows.last_mut() {
                    row.push((ch, style));
                }
            }
            _ => {}
        }
    }
    rows
}

fn fit_rows(
    mut rows: Vec<Vec<(char, Style)>>,
    height: usize,
    width: usize,
) -> Vec<Vec<(char, Style)>> {
    if height == 0 {
        return Vec::new();
    }
    while rows.last().is_some_and(|row| row.is_empty()) {
        rows.pop();
    }
    let pad = leading_pad(&rows);
    if pad > 0 {
        rows = rows
            .into_iter()
            .map(|row| skip_display_cols(&row, pad))
            .collect();
    }
    let mut fitted = if rows.len() <= height {
        pin_bottom(rows, height)
    } else {
        rows[rows.len() - height..].to_vec()
    };
    if width > 0 {
        for row in &mut fitted {
            *row = take_display_cols(row, width);
        }
    }
    fitted
}

fn pin_bottom(rows: Vec<Vec<(char, Style)>>, height: usize) -> Vec<Vec<(char, Style)>> {
    if rows.len() >= height {
        return rows[rows.len() - height..].to_vec();
    }
    let mut out = vec![Vec::new(); height - rows.len()];
    out.extend(rows);
    out
}

fn row_is_blank(row: &[(char, Style)]) -> bool {
    row.iter().all(|(ch, _)| ch.is_whitespace())
}

fn leading_pad(rows: &[Vec<(char, Style)>]) -> usize {
    rows.iter()
        .filter(|row| !row_is_blank(row))
        .map(|row| {
            let mut pad = 0usize;
            for (ch, _) in row {
                if !ch.is_whitespace() {
                    return pad;
                }
                pad += UnicodeWidthChar::width(*ch).unwrap_or(1);
            }
            usize::MAX
        })
        .min()
        .filter(|pad| *pad != usize::MAX)
        .unwrap_or(0)
}

fn skip_display_cols(row: &[(char, Style)], cols: usize) -> Vec<(char, Style)> {
    let mut used = 0usize;
    let mut index = 0usize;
    while index < row.len() && used < cols {
        used += UnicodeWidthChar::width(row[index].0).unwrap_or(1);
        index += 1;
    }
    row[index.min(row.len())..].to_vec()
}

fn take_display_cols(row: &[(char, Style)], cols: usize) -> Vec<(char, Style)> {
    let mut out = Vec::new();
    let mut used = 0usize;
    for (ch, style) in row {
        let width = UnicodeWidthChar::width(*ch).unwrap_or(1);
        if used + width > cols {
            break;
        }
        out.push((*ch, *style));
        used += width;
    }
    out
}

fn apply_sgr(mut style: Style, default: Style, params: &str) -> Style {
    let mut values = params
        .split(';')
        .map(|part| part.parse::<u8>().unwrap_or(0))
        .collect::<Vec<_>>();
    if values.is_empty() {
        values.push(0);
    }
    let mut index = 0;
    while index < values.len() {
        match values[index] {
            0 => style = default,
            1 => style = style.add_modifier(Modifier::BOLD),
            2 => style = style.add_modifier(Modifier::DIM),
            3 => style = style.add_modifier(Modifier::ITALIC),
            4 => style = style.add_modifier(Modifier::UNDERLINED),
            7 => style = style.add_modifier(Modifier::REVERSED),
            22 => style = style.remove_modifier(Modifier::BOLD | Modifier::DIM),
            23 => style = style.remove_modifier(Modifier::ITALIC),
            24 => style = style.remove_modifier(Modifier::UNDERLINED),
            27 => style = style.remove_modifier(Modifier::REVERSED),
            39 => style = style.fg(default.fg.unwrap_or(Color::Reset)),
            49 => style = style.bg(default.bg.unwrap_or(Color::Reset)),
            value @ 30..=37 => style = style.fg(basic_color(value - 30, false)),
            value @ 90..=97 => style = style.fg(basic_color(value - 90, true)),
            value @ 40..=47 => style = style.bg(basic_color(value - 40, false)),
            value @ 100..=107 => style = style.bg(basic_color(value - 100, true)),
            38 | 48 => {
                let is_fg = values[index] == 38;
                if let Some(color) = extended_color(&values, &mut index) {
                    style = if is_fg {
                        style.fg(color)
                    } else {
                        style.bg(color)
                    };
                }
            }
            _ => {}
        }
        index += 1;
    }
    style
}

fn extended_color(values: &[u8], index: &mut usize) -> Option<Color> {
    let kind = values.get(*index + 1)?;
    match kind {
        5 => {
            let color = values.get(*index + 2).copied()?;
            *index += 2;
            Some(Color::Indexed(color))
        }
        2 => {
            let r = *values.get(*index + 2)?;
            let g = *values.get(*index + 3)?;
            let b = *values.get(*index + 4)?;
            *index += 4;
            Some(Color::Rgb(r, g, b))
        }
        _ => None,
    }
}

fn basic_color(index: u8, bright: bool) -> Color {
    match (index, bright) {
        (0, false) => Color::Black,
        (1, false) => Color::Red,
        (2, false) => Color::Green,
        (3, false) => Color::Yellow,
        (4, false) => Color::Blue,
        (5, false) => Color::Magenta,
        (6, false) => Color::Cyan,
        (7, false) => Color::Gray,
        (0, true) => Color::DarkGray,
        (1, true) => Color::LightRed,
        (2, true) => Color::LightGreen,
        (3, true) => Color::LightYellow,
        (4, true) => Color::LightBlue,
        (5, true) => Color::LightMagenta,
        (6, true) => Color::LightCyan,
        _ => Color::White,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_rows(rows: &[Vec<(char, Style)>]) -> Vec<String> {
        rows.iter()
            .map(|row| row.iter().map(|(ch, _)| ch).collect())
            .collect()
    }

    #[test]
    fn indexed_foreground_survives_sgr() {
        let default = Style::default().fg(Color::White).bg(Color::Black);
        let rows = parse_ansi_rows("\u{1b}[38;5;81mPIX\u{1b}[0m", default);
        assert_eq!(rows[0].len(), 3);
        assert_eq!(rows[0][0].0, 'P');
        assert_eq!(rows[0][0].1.fg, Some(Color::Indexed(81)));
    }

    #[test]
    fn live_bottom_keeps_prompt_when_card_is_shorter() {
        let default = Style::default();
        let mut lines = vec!["PIX".to_owned(), "ready".to_owned()];
        lines.extend(std::iter::repeat_n(" ".to_owned(), 8));
        lines.push("> hello".to_owned());
        lines.push(".pi | grok-4.6".to_owned());
        let fitted = fit_rows(parse_ansi_rows(&lines.join("\n"), default), 4, 20);
        let text = text_rows(&fitted);
        assert_eq!(fitted.len(), 4);
        assert!(text.iter().any(|line| line.contains("hello")));
        assert!(text.last().unwrap().contains("grok-4.6"));
    }

    #[test]
    fn conversation_uses_latest_rows_and_fills_card() {
        let default = Style::default();
        let mut lines = Vec::new();
        for index in 0..20 {
            lines.push(format!("msg {index}"));
        }
        lines.push("> prompt".to_owned());
        let fitted = fit_rows(parse_ansi_rows(&lines.join("\n"), default), 8, 20);
        let text = text_rows(&fitted);
        assert_eq!(fitted.len(), 8);
        assert!(text[0].contains("msg 13") || text[0].contains("msg 14"));
        assert!(text.last().unwrap().contains("prompt"));
    }

    #[test]
    fn short_content_pins_prompt_to_bottom() {
        let default = Style::default();
        let fitted = fit_rows(parse_ansi_rows("hi\n\n> prompt", default), 5, 20);
        let text = text_rows(&fitted);
        assert_eq!(fitted.len(), 5);
        assert!(text[4].contains("prompt"));
        assert!(text[0].is_empty());
    }
}
