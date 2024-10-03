use std::io::{stdout, Write, Error};
use crossterm::{
    cursor, event::{self, Event, KeyCode, KeyEvent}, style::{self, Color, Stylize}, terminal, ExecutableCommand, QueueableCommand
};
use std::time::Duration;

fn main() -> Result<(), Error> {
    terminal::enable_raw_mode()?;
    stdout().execute(cursor::Hide)?;
    
    clear()?;
    draw_rect(0, 0, 20, 10, Color::Red)?;
    stdout().flush()?;

    let mut pos: (u16, u16) = (0, 0);

    loop {
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(event) => input(event, &mut pos),
                Event::FocusGained => Ok(()),
                Event::FocusLost => Ok(()),
                Event::Mouse(_) => Ok(()),
                Event::Paste(_) => Ok(()),
                Event::Resize(_, _) => Ok(()),
            }?;
        }

        draw_char(pos.0, pos.1, 'x', Color::Yellow)?;
        stdout().flush()?;
    }
}

fn input(event: KeyEvent, pos: &mut (u16, u16)) -> Result<(), Error> {
    if event.code == KeyCode::Char('q') {
        quit()?;
    } else if event.code == KeyCode::Left {
        if pos.0 > 0 {pos.0 -= 1};
    } else if event.code == KeyCode::Right {
        if pos.0 < 20 {pos.0 += 1};
    } else if event.code == KeyCode::Up {
        if pos.1 > 0 {pos.1 -= 1};
    } else if event.code == KeyCode::Down {
        if pos.1 < 10 {pos.1 += 1};
    }
    stdout().flush()?;
    return Ok(());
}

fn clear() -> Result<(), Error> {
    stdout().execute(terminal::Clear(terminal::ClearType::All))?;
    return Ok(());
}

fn draw_char(x: u16, y: u16, c: char, color: Color) -> Result<(), Error> {
    stdout()
        .queue(cursor::MoveTo(x, y))?
        .queue(style::PrintStyledContent(c.with(color)))?
    ;
    return Ok(());
}

fn draw_rect(x1: u16, y1: u16, x2: u16, y2: u16, color: Color) -> Result<(), Error> {
    for x in x1..=x2 {
        for y in y1..=y2 {
            if x == x1 || x == x2 || y == y1 || y == y2 {
                let c: &str;
                if x == x1 && y == y1 {
                    c = "┌";
                } else if x == x1 && y == y2 {
                    c = "└";
                } else if x == x2 && y == y1 {
                    c = "┐";
                } else if x == x2 && y == y2 {
                    c = "┘";
                } else if x == x1 || x == x2 {
                    c = "│";
                } else if y == y1 || y == y2 {
                    c = "─";
                } else {
                    c = " ";
                }
                stdout()
                    .queue(cursor::MoveTo(x, y))?
                    .queue(style::PrintStyledContent(c.with(color)))?
                ;
            }
        }
    }
    return Ok(());
}

fn quit() -> Result<(), Error> {
    stdout().execute(cursor::Show)?;
    std::process::exit(0);
}
