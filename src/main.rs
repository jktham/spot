use std::io::{stdout, Write, Error};
use crossterm::{
    ExecutableCommand, QueueableCommand,
    terminal, cursor, style::{self, Stylize, Color}
};

fn main() -> Result<(), Error> {
    clear()?;
    draw_rect(0, 0, 40, 20, Color::Red)?;
    stdout().flush()?;
    return Ok(());
}

fn clear() -> Result<(), Error> {
    stdout().execute(terminal::Clear(terminal::ClearType::All))?;
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
