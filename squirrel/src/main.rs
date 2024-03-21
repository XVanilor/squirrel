#[macro_use]
extern crate rocket;

mod route;
mod waiting_list;

use std::{sync::Arc, time::Duration};

use tokio::sync::broadcast::Sender;
use tokio::{sync::Mutex, time::interval};
use rocket::{launch, routes, Config};

use crate::waiting_list::RegistrationList;
use squirrel::{Event, TicketAmount};

const DEFAULT_MAX_USERS_IN_REG_SIMULTANEOUSLY: i16 = 2;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct TicketCount {
    count: TicketAmount
}

pub(crate) struct State {
    #[allow(unused)]
    max_user_in_reg_simultaneously: i16,
    user_evt_tx: Arc<tokio::sync::broadcast::Sender<squirrel::Event>>,
    prereg_list: Arc<Mutex<RegistrationList>>,
    reg_crawler_tx: Sender<TicketCount>,
    last_seen_tickets_left: Arc<Mutex<TicketAmount>>
}

#[launch]
async fn rocket() -> _ {

    simple_logger::SimpleLogger::new()
        .with_module_level("reqwest::connect", log::LevelFilter::Off)
        .init()
        .unwrap();

    let (reg_crawler_tx, _) = tokio::sync::broadcast::channel::<TicketCount>(10);
    let reg_crawler = reg_crawler_tx.clone();

    let last_seen_tickets_left = Arc::new(Mutex::new(0));
    let last_ticket_count_seen_poller = Arc::clone(&last_seen_tickets_left);

    let user_evt_tx = Arc::new(tokio::sync::broadcast::Sender::<Event>::new(10));
    let prereg_list = Arc::new(Mutex::new(RegistrationList::new()));

    // Start polling on registration for number of tickets left
    tokio::spawn(async move {

        let mut interval = interval(Duration::from_secs(1));

        loop {

            let ticket_count = reqwest::get("http://127.0.0.1:8081/tickets_count")
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
            let ticket_count = serde_json::from_slice::<TicketCount>(ticket_count.as_bytes()).unwrap();

            {
                let mut last_count_lock = last_ticket_count_seen_poller.lock().await;
                if *last_count_lock != ticket_count.count {

                    *last_count_lock = ticket_count.count;
                    let _ = reg_crawler.send(TicketCount{ count: ticket_count.count });
                }
            }

            interval.tick().await;
        }
    });

    // Log user events in app logs
    let mut user_evt_sub = user_evt_tx.subscribe();
    tokio::spawn(async move {
        loop {
            // Loop over user-only events
            let evt = user_evt_sub.recv().await.unwrap();
            log::info!("Received user event: {evt:?}");
        }
    });

    let mut config = Config::default();
    config.port = 8080;

    let routes = routes![
        route::events, // Backend
        route::wall, // Front-end
        route::register, // Pre-registration
    ];

    let admin_routes = routes![
        route::start_registration
    ];

    let init_state = State {
        max_user_in_reg_simultaneously: DEFAULT_MAX_USERS_IN_REG_SIMULTANEOUSLY,
        user_evt_tx,
        prereg_list,
        reg_crawler_tx,
        last_seen_tickets_left
    };

    // Start rocket
    rocket::custom(config)
        .manage(init_state)
        .mount("/", routes)
        .mount("/admin", admin_routes)
}
