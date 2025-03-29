pub mod rope;

/**************************************************************/

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
use rope::Rope;
use std::collections::BTreeSet;
use std::io;
use std::vec;
use tui_textarea::{Input, Key, TextArea};

/**************************************************************/

fn f<'a>(input: &Rope<'a>) -> Rope<'a> {
    let re_indent = Regex::new(r#"Indent: ".*""#).unwrap();
    let re_period = Regex::new(r#"\."#).unwrap();

    let mut modified = Rope::new();
    modified.append(input.re_replaces(&re_period, &"??".into()));
    modified.append("\n".into());
    modified.append("\n".into());
    modified.append(input.re_replaces(&re_period, &"!!".into()));

    // Find indent and apply it
    if let Some(indent) = modified.re_slice(&re_indent) {
        modified = modified.indent(&indent.slice(9..indent.len() - 1));
    }

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

    let mut output = Rope::new();
    output.append(header(" INPUT "));
    output.append(input.clone());
    output.append(header(" OUTPUT "));
    output.append(modified);

    output
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

        Indent: \"    \""};
    let mut textarea = TextArea::default();
    textarea.insert_str(input);
    textarea.set_block(Block::bordered().title(" Input "));

    let block = Block::bordered().title(" Output ");

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2); 2].as_ref());

    let mut xys = BTreeSet::<(usize, usize)>::new();

    loop {
        // inputs
        let input_str = textarea.lines().join("\n");
        let input_rope = Rope::from(input_str.as_str());

        // outputs
        let output_rope = f(&input_rope);
        let output = output_rope.to_string();

        // get all selected addresses based on the current xys (mouse positions).
        let mut addresses = BTreeSet::<usize>::new();
        let mut x = 0;
        let mut y = 0;
        for data in &output_rope.data {
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
        for data in &output_rope.data {
            let mut addr = data.as_ptr() as usize;
            for c in data.chars() {
                if c == '\n' {
                    y += 1;
                    x = 0;
                    lines.push(Line::from(spans.clone()));
                    spans.clear();
                } else {
                    let span = if xys.contains(&(x, y)) {
                        Span::styled(
                            (if c == ' ' { '·' } else { c }).to_string(),
                            Style::default().fg(Color::Red),
                        )
                    } else if addresses.contains(&addr) {
                        Span::styled(
                            (if c == ' ' { '·' } else { c }).to_string(),
                            Style::default().fg(Color::Blue),
                        )
                    } else {
                        Span::from(c.to_string())
                    };
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
                    if let Some(line) = output.lines().nth(mouse_y) {
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
