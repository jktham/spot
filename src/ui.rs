use std::io::{stdout, Write};
use crossterm::{cursor, style::{self, Color, Stylize}, terminal, ExecutableCommand, QueueableCommand};

use crate::{spotify::Playback, Err};

pub fn draw_ui(playback: &Playback, frame: &i32, help: &bool) -> Result<(), Err> {
    let size = (51, 9);
    let bg: Color = Color::Red;
    let fg: Color = match (playback.is_active, playback.is_playing) {
        (false, _) => Color::Red,
        (true, false) => Color::Cyan,
        (true, true) => Color::Green,
    };

    clear()?;

    if *help {
        draw_text(size.0 - 3, size.1-2, "H", 0, fg)?;
        draw_text(size.0 - 6, 1, "spot", 0, fg)?;
        draw_text(size.0 - 8, 2, "v0.1.0", 0, fg)?;
        draw_text(2, 1, "[space]       play/pause", 0, fg)?;
        draw_text(2, 2, "[a]           prev", 0, fg)?;
        draw_text(2, 3, "[d]           next", 0, fg)?;
        draw_text(2, 4, "[r]           repeat", 0, fg)?;
        draw_text(2, 5, "[s]           shuffle", 0, fg)?;
        draw_text(2, 6, "[h]           help", 0, fg)?;
        draw_text(2, 7, "[q]           quit", 0, fg)?;

    } else {
        draw_text(size.0 - 3, size.1-2, "h", 0, fg)?;

        let indicator: &str = match frame % 2 {
            0 => "/",
            1 => "\\",
            _ => "",
        };
        draw_text(48, 1, indicator, 0, fg)?;

        let state: &str = match (playback.is_active, playback.is_playing) {
            (false, _) => "inactive",
            (true, false) => "paused",
            (true, true) => "playing",
        };
        draw_text(2, 1, state, 0, fg)?;
    
        if playback.is_active {
            draw_text(2, 2, &format!("{}:{:0>2} / {}:{:0>2}", &playback.progress/1000/60, &playback.progress/1000%60, &playback.duration/1000/60, &playback.duration/1000%60), 0, fg)?;
            draw_text(2, 4, &playback.title, size.0 - 4, fg)?;
            draw_text(2, 5, &playback.album, size.0 - 4, fg)?;
            draw_text(2, 6, &playback.artist, size.0 - 4, fg)?;
            let repeat = match playback.repeat_state.as_str() {
                "off" => "r ",
                "track" => "R'",
                "context" => "R ",
                _ => "",
            };
            let shuffle = match playback.shuffle_state {
                false => "s",
                true => "S",
            };
            draw_text(2, 7, &format!("{}/ {}", repeat, shuffle), 0, fg)?;
        }

        draw_fill(size.0 - 2, 4, size.0 + 50, 6, ' ', bg)?;
    }

    draw_rect(0, 0, size.0 - 1, size.1 - 1, bg)?;

    stdout().flush()?;
    return Ok(());
}

pub fn clear() -> Result<(), Err> {
    stdout().execute(terminal::Clear(terminal::ClearType::All))?;
    return Ok(());
}

pub fn draw_text(x: i32, y: i32, text: &str, length: i32, color: Color) -> Result<(), Err> {
    if x < 0 || y < 0 {
        return Ok(());
    }
    let mut t = String::from(text);
    if length > 0 {
        t = text.chars().into_iter().take(length as usize).collect::<String>();
        if length > 3 && text.chars().into_iter().count() > length as usize {
            t = text.chars().into_iter().take((length-3) as usize).collect::<String>();
            t += "...";
        }
    }
    stdout()
        .queue(cursor::MoveTo(x as u16, y as u16))?
        .queue(style::PrintStyledContent(t.with(color)))?;
    return Ok(());
}

pub fn draw_rect(x1: i32, y1: i32, x2: i32, y2: i32, color: Color) -> Result<(), Err> {
    if x1 < 0 || y1 < 0 || x1 > x2 || y1 > y2 {
        return Ok(());
    }
    for x in x1..=x2 {
        for y in y1..=y2 {
            let c: &str = match (x, y) {
                (x, y) if (x, y) == (x1, y1) => "┌",
                (x, y) if (x, y) == (x1, y2) => "└",
                (x, y) if (x, y) == (x2, y1) => "┐",
                (x, y) if (x, y) == (x2, y2) => "┘",
                (x, _) if x == x1 || x == x2 => "│",
                (_, y) if y == y1 || y == y2 => "─",
                (_, _) => "",
            };

            if c != "" {
                stdout()
                    .queue(cursor::MoveTo(x as u16, y as u16))?
                    .queue(style::PrintStyledContent(c.with(color)))?;
            }

        }
    }
    return Ok(());
}

pub fn draw_fill(x1: i32, y1: i32, x2: i32, y2: i32, c: char, color: Color) -> Result<(), Err> {
    if x1 < 0 || y1 < 0 || x1 > x2 || y1 > y2 {
        return Ok(());
    }
    for x in x1..=x2 {
        for y in y1..=y2 {
            stdout()
                .queue(cursor::MoveTo(x as u16, y as u16))?
                .queue(style::PrintStyledContent(c.with(color)))?;
        }
    }
    return Ok(());
}
