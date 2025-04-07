pub mod arena;
pub mod relation;
pub mod rope;

/**************************************************************/

use arena::Arena;
use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event, MouseEvent, MouseEventKind,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use indoc::indoc;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Terminal;
use regex::Regex;
use relation::{str_addrs, Relation};
use rope::Rope;
use std::collections::BTreeSet;
use std::io;
use std::vec;
use tui_textarea::{Input, Key, TextArea};

/**************************************************************/

fn header(s: &str) -> Rope<'_> {
    let mut header = Rope::new();
    header.append("\n".into());
    header.append("\n".into());
    header.append("###".into());
    header.append(s.into());
    header.append("###".into());
    header.append("\n".into());
    header.append("\n".into());
    header
}

fn reverse(s: &str) -> String {
    s.chars().rev().collect()
}

fn filter(s: &str) -> String {
    s.chars().filter(|c| !"aeiouyAEIOUY".contains(*c)).collect()
}

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture,)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let input = indoc! {"
        ← Left  : A simple text editor.
        → Right : Shows modified text. Select text to view str reuse.
        ␛ Exit  : Press `Esc` to exit...

        Indent: \"    \"
        Modify: \"abcdefghi\""};
    let mut textarea = TextArea::default();
    textarea.insert_str(input);
    textarea.set_block(Block::bordered().title(" Input "));

    let block = Block::bordered().title(" Output ");

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2); 2].as_ref());

    let mut xys = BTreeSet::<(usize, usize)>::new();

    let re_indent = Regex::new(r#"Indent: ".*""#).unwrap();
    let re_period = Regex::new(r#"\."#).unwrap();
    let re_modify = Regex::new(r#"Modify: ".*""#).unwrap();

    loop {
        // set up string tracking context
        let mut arena = Arena::default();
        let mut rel = Relation::default();

        // inputs
        let input_str = textarea.lines().join("\n");
        let input = Rope::from(input_str.as_str());

        // outputs
        let mut part1 = input.re_replaces(&re_period, &"??".into());
        let mut part2 = input.re_replaces(&re_period, &"!!".into());
        if let Some(mat) = input.re_slice(&re_modify) {
            let input = mat.slice(9..mat.len() - 1);
            let input_str = input.to_string();

            let reversed = reverse(&input_str);
            let filtered = filter(&input_str);

            let reversed_id = arena.allocate(reversed);
            let filterd_id = arena.allocate(filtered);

            let reversed = arena.get(reversed_id).unwrap();
            let filtered = arena.get(filterd_id).unwrap();

            rel.add_n_n(&input.addrs(), &str_addrs(reversed));
            rel.add_n_n(&input.addrs(), &str_addrs(filtered));

            let mut reversed_rope = mat.slice(0..9);
            reversed_rope.append(reversed.as_str().into());
            reversed_rope.append(mat.slice(mat.len() - 1..mat.len()));
            part1 = part1.re_replace(&re_modify, &reversed_rope);

            let mut filtered_rope = mat.slice(0..9);
            filtered_rope.append(filtered.as_str().into());
            filtered_rope.append(mat.slice(mat.len() - 1..mat.len()));
            part2 = part2.re_replace(&re_modify, &filtered_rope);
        }

        let mut modified = Rope::new();
        modified.append(part1);
        modified.append("\n".into());
        modified.append("\n".into());
        modified.append(part2);

        // Find indent and apply it
        if let Some(indent) = input.re_slice(&re_indent) {
            modified = modified.indent(&indent.slice(9..indent.len() - 1));
        }

        let mut output = Rope::new();
        output.append(header(" INPUT "));
        output.append(input.clone());
        output.append(header(" OUTPUT "));
        output.append(modified);
        let output_str = output.to_string();

        // get all selected addresses based on the current xys (mouse positions).
        let mut addresses = BTreeSet::<usize>::new();
        let mut x = 0;
        let mut y = 0;
        for data in &output.data {
            let mut addr = data.as_ptr() as usize;
            for c in data.chars() {
                if c == '\n' {
                    y += 1;
                    x = 0;
                } else {
                    if xys.contains(&(x, y)) {
                        addresses.insert(addr);
                    }
                    x += 1;
                }
                addr += c.len_utf8();
            }
        }

        // create the lines for the output based on the addresses and xys.
        let mut lines = vec![];
        let mut spans = vec![];
        let mut x = 0;
        let mut y = 0;
        for data in &output.data {
            let mut addr = data.as_ptr() as usize;
            for c in data.chars() {
                if c == '\n' {
                    y += 1;
                    x = 0;
                    lines.push(Line::from(spans.clone()));
                    spans.clear();
                } else {
                    let mut span = Span::from(c.to_string());
                    if xys.contains(&(x, y)) {
                        // selected
                        span = Span::styled(
                            (if c == ' ' { '·' } else { c }).to_string(),
                            Style::default().fg(Color::Red),
                        );
                    } else if addresses.contains(&addr) {
                        // same address as selected
                        span = Span::styled(
                            (if c == ' ' { '·' } else { c }).to_string(),
                            Style::default().fg(Color::Blue),
                        );
                    } else if let Some(out) = rel.rel(addr) {
                        if !out.is_disjoint(&addresses) {
                            // upstream of selected
                            span = Span::styled(
                                (if c == ' ' { '·' } else { c }).to_string(),
                                Style::default().fg(Color::Green),
                            );
                        }
                    } else if let Some(out) = rel.inv(addr) {
                        if !out.is_disjoint(&addresses) {
                            // downstream of selected
                            span = Span::styled(
                                (if c == ' ' { '·' } else { c }).to_string(),
                                Style::default().fg(Color::Yellow),
                            );
                        }
                    }
                    spans.push(span);
                    x += 1;
                }
                addr += c.len_utf8();
            }
        }
        if !spans.is_empty() {
            lines.push(Line::from(spans.clone()));
        }

        // needed for mouse event offsets
        let mut chunk_x = 0;
        let mut chunk_y = 0;

        // draw the terminal
        term.draw(|f| {
            let chunks = layout.split(f.area());
            chunk_x = chunks[1].x;
            chunk_y = chunks[1].y;

            f.render_widget(&textarea, chunks[0]);
            f.render_widget(
                Paragraph::new(Text::from(lines)).block(block.clone()),
                chunks[1],
            );
        })?;

        // handle mouse events and input
        match crossterm::event::read()? {
            Event::Mouse(MouseEvent {
                column,
                row,
                kind,
                modifiers,
            }) => {
                if matches!(kind, MouseEventKind::Down(_)) && modifiers.is_empty() {
                    xys.clear();
                }
                if kind != MouseEventKind::Moved {
                    let mouse_x = column.saturating_sub(chunk_x + 1) as usize;
                    let mouse_y = row.saturating_sub(chunk_y + 1) as usize;
                    if let Some(line) = output_str.lines().nth(mouse_y) {
                        if line.len() > mouse_x {
                            xys.insert((mouse_x, mouse_y));
                        }
                    }
                }
            }
            event => match event.into() {
                Input { key: Key::Esc, .. } => break,
                input => {
                    textarea.input(input);
                    xys.clear();
                }
            },
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;

    Ok(())
}
