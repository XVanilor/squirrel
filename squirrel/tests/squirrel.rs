mod common;

use std::sync::Arc;

use tokio::task::JoinSet;
use tokio_stream::StreamExt;
use reqwest::Url;
use reqwest_eventsource::{Event, Error as EventSourceError, EventSource};

use common::*;

#[tokio::test]
async fn test_squirrel() {

    const CLIENTS_NB: i8 = 2;
    const SQUIRREL_URL: &str = "http://127.0.0.1:8080";

    let squirrel_gw = Arc::new(Url::parse(SQUIRREL_URL).unwrap());

    let _fake_reg_server = start_fake_reg();
    let _squirrel_server = start_squirrel();
    let admin = SquirrelAdmin::new(Arc::clone(&squirrel_gw));

    let mut handles = JoinSet::new();

    for _ in 0..CLIENTS_NB {

        let squirrel_client_gw = Arc::clone(&squirrel_gw);
        handles.spawn(async {

            let mut client = Client::new(squirrel_client_gw);
            let registration = client.register().await;
            assert!(registration.is_ok());

            let evt_watcher = client.watch_events();
            assert!(evt_watcher.is_ok());
            let mut evt_watcher = evt_watcher.unwrap();
            
            // Check that the connection is open
            assert!(next_event_is(&mut evt_watcher, Event::Open).await);
            assert!(next_event_is(&mut evt_watcher, Event::Open).await);
        });
    }

    let res = admin.start_registration().await;
    assert!(res.is_ok());        

    // Check that clients processes went without error
    let mut clients_has_error = false;
    while let Some(res) = handles.join_next().await {
        if let Err(err) = res {
            println!("Client error: {:?}", err.to_string());
            clients_has_error = true;
        }
    }
    assert!(!clients_has_error);
}

async fn next_event_is(stream: &mut EventSource, expected_event: Event) -> bool {

    let next: Option<Result<Event, EventSourceError>> = stream.next().await;
    assert!(&next.is_some());
    let next: Result<Event, EventSourceError> = next.unwrap();
    assert!(&next.is_ok());
    let _next = next.unwrap();
    assert!(matches!(expected_event, _next));

    true
}