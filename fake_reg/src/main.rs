#[macro_use] extern crate rocket;

use std::sync::Arc;

use tokio::sync::Mutex;
use rocket::{config::{Config, LogLevel}, serde::json::Json};

pub(crate) struct FakeRegistrationState {
    pub tickets_count: Arc<Mutex<i16>>
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct TicketCount {
    pub count: i16
}

#[get("/")]
pub(crate) fn home() -> String {
    "This is a fake registration website".to_string()
}

#[get("/tickets_count")]
pub(crate) async fn get_ticket_count(
    reg_state: &rocket::State<FakeRegistrationState>
) -> Json<TicketCount> {

    let ticket_count = reg_state.tickets_count.lock().await;

    Json(TicketCount {
        count: ticket_count.clone()
    })
}

#[post("/buy_ticket?<_access_token>")]
pub(crate) async fn buy_ticket(
    _access_token: String,
    reg_state: &rocket::State<FakeRegistrationState>
) -> Json<bool> {

    let mut ticket_count = reg_state.tickets_count.lock().await;
    if *ticket_count > 0 {
        *ticket_count -= 1;
        Json(true)
    }
    else {
        Json(false)
    }
}

#[launch]
async fn rocket() -> _ {

    simple_logger::SimpleLogger::new().env().init().unwrap();

    let mut config = Config::default();
    config.log_level = LogLevel::Off;
    config.port = 8081;

    let fake_reg_routes = routes![
        home,
        get_ticket_count,
        buy_ticket
    ];

    rocket::custom(config)
        .manage(FakeRegistrationState {
             tickets_count: Arc::new(Mutex::new(10))
        })
        .mount("/", fake_reg_routes)
}