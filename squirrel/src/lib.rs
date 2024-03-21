pub type AccessToken = uuid::Uuid;
pub type TicketAmount = i64;


#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "content")]
pub enum Event {
    // These events 
    PublicEvent(PublicEvent),
    SingleUser(SingleUserEvent)
}

/// These events are transmitted to only one user, while they does fetch events in websocket
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SingleUserEvent {
    pub user_id: UserId,
    pub evt: PublicEvent
}

/// These events can be seen by all users
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum PublicEvent {
    AllowedToRegister,
    TicketsLeft(TicketAmount),
    RegistrationClosed,
    RegistrationStarting,
    RegistrationStarted
}

pub type UserId = uuid::Uuid;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UserAccount {
    pub id: UserId
}