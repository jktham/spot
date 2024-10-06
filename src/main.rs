use std::{io::{stdin, stdout, Write}, time::{Duration, SystemTime, UNIX_EPOCH}};
use crossterm::{cursor, event::{self, Event, KeyCode, KeyEvent}, style::{self, Color, Stylize}, terminal, ExecutableCommand, QueueableCommand};
use reqwest::{self, blocking::Client};
use serde_json::{self, Value};
use base64::prelude::*;
use serde::{Deserialize, Serialize};
use toml;

type Err = Box<dyn std::error::Error>;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Creds {
    client_id: String,
    client_secret: String,
    access_token: String,
    refresh_token: String,
    expires_at: u64,
}

struct Playback {
    is_active: bool,
    is_playing: bool,
    title: String,
    album: String,
    artist: String,
    progress: i64,
    duration: i64,
}

fn main() -> Result<(), Err> {
    let client = Client::new();
    let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    if !std::fs::exists("./creds.toml")? {
        std::fs::write("./creds.toml", "client_id = \"\"\nclient_secret = \"\"\naccess_token = \"\"\nrefresh_token = \"\"\nexpires_at = 0\n")?;
    }

    let creds_string = std::fs::read_to_string("./creds.toml")?;
    let mut creds: Creds = toml::from_str(&creds_string)?;
    println!("file creds: {creds:?}");

    if creds.client_id == "" || creds.client_secret == "" {
        println!("no api client creds");
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
    println!("new creds: {creds:?}");

    terminal::enable_raw_mode()?;
    stdout().execute(cursor::Hide)?;
    stdout().execute(terminal::EnterAlternateScreen)?;

    let mut frame: i32 = 0;
    loop {
        let playback = get_playback(&client, &creds)?;
        draw_ui(&playback, &frame)?;

        if event::poll(Duration::from_millis(1000))? {
            match event::read()? {
                Event::Key(event) => input(event, &client, &creds),
                Event::FocusGained => Ok(()),
                Event::FocusLost => Ok(()),
                Event::Mouse(_) => Ok(()),
                Event::Paste(_) => Ok(()),
                Event::Resize(_, _) => Ok(()),
            }?;
        }

        frame += 1;
    }
}

fn input(event: KeyEvent, client: &Client, creds: &Creds) -> Result<(), Err> {
    if event.code == KeyCode::Char('q') {
        quit()?;
    } else if event.code == KeyCode::Char('r') {
        test_request(&client, &creds)?;
    } else if event.code == KeyCode::Char('p') {
        test_request_playback(&client, &creds)?;
    }
    
    return Ok(());
}

fn draw_ui(playback: &Playback, frame: &i32) -> Result<(), Err> {
    let bg: Color = Color::Red;
    let fg: Color = match (playback.is_active, playback.is_playing) {
        (false, _) => Color::Red,
        (true, false) => Color::Cyan,
        (true, true) => Color::Green,
    };

    clear()?;
    draw_rect(0, 0, 50, 7, bg)?;

    let indicator: &str = match frame % 2 {
        0 => "/",
        1 => "\\",
        _ => "",
    };
    draw_text(48, 1, indicator, fg)?;

    let state: &str = match (playback.is_active, playback.is_playing) {
        (false, _) => "inactive",
        (true, false) => "paused",
        (true, true) => "playing",
    };
    draw_text(2, 1, state, fg)?;

    if playback.is_active {
        draw_text(2, 2, &format!("{}:{:0>2} / {}:{:0>2}", &playback.progress/1000/60, &playback.progress/1000%60, &playback.duration/1000/60, &playback.duration/1000%60), fg)?;
        draw_text(2, 4, &playback.title, fg)?;
        draw_text(2, 5, &playback.album, fg)?;
        draw_text(2, 6, &playback.artist, fg)?;
    }

    stdout().flush()?;
    return Ok(());
}

fn get_playback(client: &Client, creds: &Creds) -> Result<Playback, Err> {
    let res = client.get("https://api.spotify.com/v1/me/player")
        .bearer_auth(&creds.access_token)
        .send()?
        .text()?
    ;

    let v: Value = serde_json::from_str(&res).unwrap_or(Value::String("inactive".to_string()));

    let mut playback = Playback {
        is_active: false,
        is_playing: false,
        title: "".to_string(),
        album: "".to_string(),
        artist: "".to_string(),
        progress: 0,
        duration: 0,
    };
    
    playback.is_active = v.as_str().unwrap_or("") != "inactive";
    playback.is_playing =  v["is_playing"].as_bool().unwrap_or(false);
    playback.title = v["item"]["name"].as_str().unwrap_or("").to_string();
    playback.album = v["item"]["album"]["name"].as_str().unwrap_or("").to_string();
    playback.artist = v["item"]["artists"][0]["name"].as_str().unwrap_or("").to_string();
    playback.progress = v["progress_ms"].as_i64().unwrap_or(0);
    playback.duration = v["item"]["duration_ms"].as_i64().unwrap_or(0);

    return Ok(playback);
}

fn new_auth(client: &Client, creds: &Creds) -> Result<Creds, Err> {
    let scope = "user-read-playback-state%20user-modify-playback-state%20user-read-currently-playing%20user-library-read";
    let redirect_uri = "https://localhost:8888/callback";
    let url = format!("https://accounts.spotify.com/authorize?response_type=code&client_id={}&scope={scope}&redirect_uri={redirect_uri}", creds.client_id);
    println!("authorize: {url}");

    println!("enter code or uri: ");
    let mut input = String::new();
    stdin().read_line(&mut input)?;

    let code = input.strip_prefix(&format!("{redirect_uri}?code=")).unwrap_or(&input).trim();
    println!("code: {code}");

    let res = client.post("https://accounts.spotify.com/api/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Authorization", format!("Basic {}", BASE64_STANDARD.encode(format!("{}:{}", creds.client_id, creds.client_secret))))
        .body(format!("grant_type=authorization_code&code={code}&redirect_uri={redirect_uri}"))
        .send()?
        .text()?
    ;

    let v: Value = serde_json::from_str(&res).unwrap();
    println!("new_auth: {v}");

    let mut creds_new = creds.clone();
    creds_new.access_token = v["access_token"].as_str().unwrap_or("").to_string();
    creds_new.refresh_token = v["refresh_token"].as_str().unwrap_or("").to_string();
    
    let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let expires_in = v["expires_in"].as_u64().unwrap_or(0);
    if expires_in != 0{
        creds_new.expires_at = time + expires_in;
    }

    return Ok(creds_new);
}

fn refresh_auth(client: &Client, creds: &Creds) -> Result<Creds, Err> {
    let res = client.post("https://accounts.spotify.com/api/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Authorization", format!("Basic {}", BASE64_STANDARD.encode(format!("{}:{}", creds.client_id, creds.client_secret))))
        .body(format!("grant_type=refresh_token&refresh_token={}&client_id={}", creds.refresh_token, creds.client_id))
        .send()?
        .text()?
    ;

    let v: Value = serde_json::from_str(&res).unwrap();
    println!("refresh_auth: {v}");

    let mut creds_new = creds.clone();
    creds_new.access_token = v["access_token"].as_str().unwrap_or("").to_string();
    creds_new.refresh_token = v["refresh_token"].as_str().unwrap_or(creds_new.refresh_token.as_str()).to_string();

    let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let expires_in = v["expires_in"].as_u64().unwrap_or(0);
    if expires_in != 0{
        creds_new.expires_at = time + expires_in;
    }

    return Ok(creds_new);
}

fn test_request(client: &Client, creds: &Creds) -> Result<(), Err> {
    let res = client.get("https://api.spotify.com/v1/artists/4Z8W4fKeB5YxbusRsdQVPb")
        .bearer_auth(&creds.access_token)
        .send()?
        .text()?
    ;

    let v: Value = serde_json::from_str(&res).unwrap();
    println!("test_request: {v}");

    return Ok(());
}

fn test_request_playback(client: &Client, creds: &Creds) -> Result<(), Err> {
    let res = client.get("https://api.spotify.com/v1/me/player")
        .bearer_auth(&creds.access_token)
        .send()?
        .text()?
    ;

    let v: Value = serde_json::from_str(&res).unwrap_or(Value::String("inactive".to_string()));
    println!("test_request_playback: {v}");

    return Ok(());
}

fn clear() -> Result<(), Err> {
    stdout().execute(terminal::Clear(terminal::ClearType::All))?;
    return Ok(());
}

fn draw_text(x: u16, y: u16, t: &str, color: Color) -> Result<(), Err> {
    stdout()
        .queue(cursor::MoveTo(x, y))?
        .queue(style::PrintStyledContent(t.with(color)))?
    ;
    return Ok(());
}

fn draw_rect(x1: u16, y1: u16, x2: u16, y2: u16, color: Color) -> Result<(), Err> {
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
                    .queue(cursor::MoveTo(x, y))?
                    .queue(style::PrintStyledContent(c.with(color)))?
                ;
            }

        }
    }
    return Ok(());
}

fn quit() -> Result<(), Err> {
    stdout().execute(terminal::LeaveAlternateScreen)?;
    stdout().execute(cursor::Show)?;
    terminal::disable_raw_mode()?;
    std::process::exit(0);
}
