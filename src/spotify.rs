use std::{io::stdin, time::{SystemTime, UNIX_EPOCH}};
use base64::{prelude::BASE64_STANDARD, Engine};
use reqwest::{self, blocking::Client};
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};

use crate::{ui::draw_ui, Err};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Creds {
    pub client_id: String,
    pub client_secret: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: u64,
}

pub struct Playback {
    pub is_active: bool,
    pub is_playing: bool,
    pub title: String,
    pub album: String,
    pub artist: String,
    pub progress: i64,
    pub duration: i64,
    pub repeat_state: String,
    pub shuffle_state: bool,
}

pub fn playback_toggle(client: &Client, creds: &Creds, playback: &mut Playback, frame: &i32, help: &bool) -> Result<(), Err> {
    if playback.is_playing {
        playback.is_playing = false;
        draw_ui(playback, frame, help)?;
        let _res = client.put("https://api.spotify.com/v1/me/player/pause")
            .bearer_auth(&creds.access_token)
            .body("")
            .send()?
            .text()?;

    } else {
        playback.is_playing = true;
        draw_ui(playback, frame, help)?;
        let _res = client.put("https://api.spotify.com/v1/me/player/play")
            .bearer_auth(&creds.access_token)
            .body("")
            .send()?
            .text()?;

    }

    return Ok(());
}

pub fn playback_next(client: &Client, creds: &Creds) -> Result<(), Err> {
    let _res = client.post("https://api.spotify.com/v1/me/player/next")
        .bearer_auth(&creds.access_token)
        .body("")
        .send()?
        .text()?;

    return Ok(());
}

pub fn playback_prev(client: &Client, creds: &Creds) -> Result<(), Err> {
    let _res = client.post("https://api.spotify.com/v1/me/player/previous")
        .bearer_auth(&creds.access_token)
        .body("")
        .send()?
        .text()?;

    return Ok(());
}

pub fn playback_repeat(client: &Client, creds: &Creds, playback: &mut Playback, frame: &i32, help: &bool) -> Result<(), Err> {
    let state = match playback.repeat_state.as_str() {
        "off" => "context",
        "context" => "track",
        "track" => "off",
        _ => "off",
    };
    playback.repeat_state = state.to_string();
    draw_ui(playback, frame, help)?;

    let _res = client.put(format!("https://api.spotify.com/v1/me/player/repeat?state={state}"))
        .bearer_auth(&creds.access_token)
        .body("")
        .send()?
        .text()?;

    return Ok(());
}

pub fn playback_shuffle(client: &Client, creds: &Creds, playback: &mut Playback, frame: &i32, help: &bool) -> Result<(), Err> {
    let state = match playback.shuffle_state {
        false => true,
        true => false,
    };
    playback.shuffle_state = state;
    draw_ui(playback, frame, help)?;

    let _res = client.put(format!("https://api.spotify.com/v1/me/player/shuffle?state={state}"))
        .bearer_auth(&creds.access_token)
        .body("")
        .send()?
        .text()?;

    return Ok(());
}

pub fn get_playback(client: &Client, creds: &Creds) -> Result<Playback, Err> {
    let res = client.get("https://api.spotify.com/v1/me/player")
        .bearer_auth(&creds.access_token)
        .send()?
        .text()?;

    let v: Value = serde_json::from_str(&res).unwrap_or(Value::String("inactive".to_string()));

    let playback = Playback {
        is_active: v.as_str().unwrap_or("") != "inactive",
        is_playing:  v["is_playing"].as_bool().unwrap_or(false),
        title: v["item"]["name"].as_str().unwrap_or("").to_string(),
        album: v["item"]["album"]["name"].as_str().unwrap_or("").to_string(),
        artist: v["item"]["artists"][0]["name"].as_str().unwrap_or("").to_string(),
        progress: v["progress_ms"].as_i64().unwrap_or(0),
        duration: v["item"]["duration_ms"].as_i64().unwrap_or(0),
        repeat_state: v["repeat_state"].as_str().unwrap_or("").to_string(),
        shuffle_state:  v["shuffle_state"].as_bool().unwrap_or(false),
    };

    return Ok(playback);
}

pub fn new_auth(client: &Client, creds: &Creds) -> Result<Creds, Err> {
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
        .text()?;

    let v: Value = serde_json::from_str(&res).unwrap();
    // println!("new_auth: {v}");

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

pub fn refresh_auth(client: &Client, creds: &Creds) -> Result<Creds, Err> {
    let res = client.post("https://accounts.spotify.com/api/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Authorization", format!("Basic {}", BASE64_STANDARD.encode(format!("{}:{}", creds.client_id, creds.client_secret))))
        .body(format!("grant_type=refresh_token&refresh_token={}&client_id={}", creds.refresh_token, creds.client_id))
        .send()?
        .text()?;

    let v: Value = serde_json::from_str(&res).unwrap();
    // println!("refresh_auth: {v}");

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

pub fn test_request(client: &Client, creds: &Creds) -> Result<(), Err> {
    let res = client.get("https://api.spotify.com/v1/artists/4Z8W4fKeB5YxbusRsdQVPb")
        .bearer_auth(&creds.access_token)
        .send()?
        .text()?;

    let v: Value = serde_json::from_str(&res).unwrap();
    println!("test_request: {v}");

    return Ok(());
}

pub fn test_request_playback(client: &Client, creds: &Creds) -> Result<(), Err> {
    let res = client.get("https://api.spotify.com/v1/me/player")
        .bearer_auth(&creds.access_token)
        .send()?
        .text()?;

    let v: Value = serde_json::from_str(&res).unwrap_or(Value::String("inactive".to_string()));
    println!("test_request_playback: {v}");

    return Ok(());
}
