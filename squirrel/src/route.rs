#[allow(unused)]
use std::time::{Duration, Instant};

use rocket::response::stream::{Event, EventStream};
use rocket::serde::json::Json;

use squirrel::{PublicEvent, SingleUserEvent, UserAccount, UserId};
use uuid::Uuid;

use crate::{State, TicketCount, Event as SquirrelEvent};
use crate::waiting_list::{MAX_USERS_PER_REG_BATCHES, WaitingList};

#[get("/")]
pub(crate) async fn wall() -> String {
    // TODO Display twig frontend
    "Hello, World".to_string()
}

#[get("/events/<user_id>")]
pub(crate) async fn events(
    #[allow(unused)]
    user_id: UserId,
    state: &rocket::State<State>
) -> EventStream![Event + '_] {

    let mut evt_rx = state.user_evt_tx.subscribe();

    EventStream! {

        loop {
            // Loop over user-only events
            let evt = evt_rx.recv().await.unwrap();

            match evt {
                SquirrelEvent::PublicEvent(public_evt) => {
                    yield Event::json::<PublicEvent>(&public_evt);
                },
                SquirrelEvent::SingleUser(user_evt) => {
                    if user_evt.user_id == user_id {
                        yield Event::json::<PublicEvent>(&user_evt.evt);
                    }
                },
            }

        }
    } 
}

#[post("/register")]
pub(crate) async fn register(
    state: &rocket::State<State>
) -> Json<UserAccount> {

    let new_account: UserAccount = UserAccount { id: Uuid::new_v4() };
    state.prereg_list.lock().await.insert(new_account.id, new_account.clone());

    Json(new_account)
}

#[post("/registration/start")]
pub(crate) async fn start_registration(
    state: &rocket::State<State>
) {

    let mut ticket_crawler_sub = state.reg_crawler_tx.subscribe();
    let user_evt_tx = state.user_evt_tx.clone();

    // Compiling random waiting queue from registration list
    user_evt_tx.send(SquirrelEvent::PublicEvent(PublicEvent::RegistrationStarting)).unwrap();
    let registration_list = &state.prereg_list.lock().await;
    let mut final_waiting_list: WaitingList = WaitingList::from_iter(registration_list.iter().map(|(k, _)| { k.clone() }));
    user_evt_tx.send(SquirrelEvent::PublicEvent(PublicEvent::RegistrationStarted)).unwrap();

    tokio::spawn(async move {

        // Each batch has TIME_BETWEEN_BATCHES to complete the registration process
        /*
        const TIME_BETWEEN_BATCHES: Duration = Duration::from_secs(30);
        let mut last_batch_entered_reg_at: Option<Instant> = None;
        */

        loop {

            if let Ok(TicketCount { count }) = ticket_crawler_sub.recv().await {

                log::trace!("Received new ticket count: {count}");

                // If registration has no more tickets left, inform the user
                if count <= 0 {
                    let evt = SquirrelEvent::PublicEvent(PublicEvent::RegistrationClosed);
                    user_evt_tx.send(evt).unwrap();
                    continue;
                }
                // Below this point, count > 0   
                // The registration process is continuing

                let evt = SquirrelEvent::PublicEvent(PublicEvent::TicketsLeft(count.clone()));
                user_evt_tx.send(evt).unwrap();

                let open_reg_access = true;
                /* feat: timeout between batches
                let open_reg_access: bool = match last_batch_entered_reg_at {
                    Some(last_batch_enter_time) => {
                        last_batch_enter_time.elapsed() > TIME_BETWEEN_BATCHES
                    },
                    None => true
                };
                */

                log::trace!("Open reg access ? {open_reg_access}");

                // The reg is opened if the last batch of attendees did have enough time to complete the registration process
                // (Which corresponds to TIME_BETWEEN_BATCHES)
                if open_reg_access {

                    log::debug!("Registration is opening {MAX_USERS_PER_REG_BATCHES} spots...");

                    // Open reg access only for a batch of users
                    for _ in 0..MAX_USERS_PER_REG_BATCHES {

                        let next_user_who_can_access_reg = final_waiting_list.pop_front();

                        log::debug!("Next user to access reg: {:?}", next_user_who_can_access_reg);

                        if next_user_who_can_access_reg.is_some() {
                            let user_id = next_user_who_can_access_reg.unwrap();
                            user_evt_tx.send(SquirrelEvent::SingleUser(SingleUserEvent { user_id, evt: PublicEvent::AllowedToRegister })).unwrap();
                        }
                    }

                    //last_batch_entered_reg_at = Some(Instant::now());
                }
            }
            else {

                log::warn!("Error: Ticket recv channel has been cutted out");
                break;
            }
        }
    });

    let last_amount_of_tickets_left = state.last_seen_tickets_left.lock().await;
    state.reg_crawler_tx.send(TicketCount { count: *last_amount_of_tickets_left }).unwrap();
}