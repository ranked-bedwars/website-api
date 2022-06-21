use std::sync::RwLock;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use actix_web::{get, rt, web, App, HttpServer, Responder};

#[derive(Serialize, Deserialize, Clone)]
struct LeaderboardProfile {
    rating: u32,
}

#[derive(Serialize, Deserialize, Clone)]
struct LeaderboardPlayer {
    discord_id: String,
    username: String,
    uuid: String,
    profile: LeaderboardProfile,
    position: u32,
}

#[derive(Clone)]
struct Manager {
    last_updated: Instant,
    players: Vec<LeaderboardPlayer>,
}

fn leaderboard_players() -> Vec<LeaderboardPlayer> {
    (0..10).into_iter().map(|i| fetch_leaderboard_segment(i * 10)).flatten().collect()
}

impl Manager {
    fn new() -> Manager {
        Manager {
            last_updated: Instant::now() - Duration::from_secs(60 * 5),
            players: Vec::new(),
        }
    }
    fn update_data(&mut self) {
        self.players = leaderboard_players();
        self.last_updated = Instant::now();
    }
    fn get_data(&mut self) -> Vec<LeaderboardPlayer> {
        if self.last_updated.elapsed().as_secs() > 60 * 5 {
            self.update_data();
        }
        self.players.clone()
    }
}
fn fetch_leaderboard_segment(skip: u32) -> Vec<LeaderboardPlayer> {
    let request = match reqwest::blocking::get(format!(
        "https://api.rankedbedwars.org/leaderboard/ranked/rating?skip={}&ascending=false&limit=10",
        skip.to_owned()
    )) {
        Ok(req) => req,
        Err(_) => return Vec::new(),
    };

    let players: Vec<LeaderboardPlayer> = match request.json::<Vec<LeaderboardPlayer>>() {
        Ok(players) => players,
        Err(_) => return Vec::new(),
    };

    players
}

#[derive(Serialize, Deserialize)]
struct Response {
    success: bool,
    data: Vec<LeaderboardPlayer>,
}

#[get("/leaderboard")]
async fn leaderboard(manager: web::Data<RwLock<Manager>>) -> impl Responder {
    let data = manager.write().unwrap().get_data();

    let response = Response {
        success: true,
        data: data.clone(),
    };

    web::Json(response)
}

fn main() -> std::io::Result<()> {
    let data = web::Data::new(RwLock::new(Manager::new()));

    rt::System::new().block_on(
        HttpServer::new(move || App::new().app_data(data.clone()).service(leaderboard))
            .bind(("127.0.0.1", 2022))?
            .run()
    )
}