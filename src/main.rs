use std::{io::stdout, time::{Duration, SystemTime, UNIX_EPOCH}};
use crossterm::{cursor, event::{self, Event, KeyCode, KeyEvent, KeyModifiers}, terminal, ExecutableCommand};
use reqwest::{self, blocking::Client};
use toml;

mod ui;
use ui::*;

mod spotify;
use spotify::*;

pub type Err = Box<dyn std::error::Error>;

fn main() -> Result<(), Err> {
    let client = Client::new();
    let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    if !std::fs::exists("./creds.toml")? {
        std::fs::write("./creds.toml", "client_id = \"\"\nclient_secret = \"\"\naccess_token = \"\"\nrefresh_token = \"\"\nexpires_at = 0\n")?;
    }

    let creds_string = std::fs::read_to_string("./creds.toml")?;
    let mut creds: Creds = toml::from_str(&creds_string)?;
    // println!("file creds: {creds:?}");

    if creds.client_id == "" || creds.client_secret == "" {
        println!("no api client creds");
        stdout().execute(terminal::EnterAlternateScreen)?;
        quit()?;

    } else if creds.refresh_token == "" {
        creds = new_auth(&client, &creds)?;
        let creds_string = toml::to_string(&creds)?;
        std::fs::write("./creds.toml", creds_string)?;

    } else if creds.expires_at < time {
        creds = refresh_auth(&client, &creds)?;
        let creds_string = toml::to_string(&creds)?;
        std::fs::write("./creds.toml", creds_string)?;

    }
    // println!("new creds: {creds:?}");

    terminal::enable_raw_mode()?;
    stdout().execute(cursor::Hide)?;
    stdout().execute(terminal::EnterAlternateScreen)?;

    let mut help: bool = false;
    let mut frame: i32 = 0;
    loop {
        let mut playback = get_playback(&client, &creds)?;
        draw_ui(&playback, &frame, &help)?;

        if event::poll(Duration::from_millis(1000))? {
            match event::read()? {
                Event::Key(event) => input(event, &client, &creds, &mut playback, &frame, &mut help)?,
                Event::FocusGained => (),
                Event::FocusLost => (),
                Event::Mouse(_) => (),
                Event::Paste(_) => (),
                Event::Resize(_, _) => (),
            };
        }
        draw_ui(&playback, &frame, &help)?;

        frame += 1;
    }
}

fn input(event: KeyEvent, client: &Client, creds: &Creds, playback: &mut Playback, frame: &i32, help: &mut bool) -> Result<(), Err> {
    match (event.code, event.modifiers) {
        (KeyCode::Char('q'), _) => quit()?,
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => quit()?,
        (KeyCode::Char(' '), _) => playback_toggle(&client, &creds, playback, frame, help)?,
        (KeyCode::Char('d'), _) => playback_next(&client, &creds)?,
        (KeyCode::Char('a'), _) => playback_prev(&client, &creds)?,
        (KeyCode::Char('r'), _) => playback_repeat(&client, &creds, playback, frame, help)?,
        (KeyCode::Char('s'), _) => playback_shuffle(&client, &creds, playback, frame, help)?,
        (KeyCode::Char('h'), _) => *help = !*help,
        (KeyCode::Char('y'), _) => test_request(&client, &creds)?,
        (KeyCode::Char('x'), _) => test_request_playback(&client, &creds)?,
        _ => (),
    };

    return Ok(());
}

fn quit() -> Result<(), Err> {
    stdout().execute(terminal::LeaveAlternateScreen)?;
    stdout().execute(cursor::Show)?;
    terminal::disable_raw_mode()?;
    std::process::exit(0);
}
