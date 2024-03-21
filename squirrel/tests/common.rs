use std::path::PathBuf;
use std::sync::Arc;

use reqwest::Url;
use reqwest_eventsource::EventSource;
use rexpect::session::PtySession;

use squirrel::UserAccount;

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("The client requires an account, which is missing")]
    MissingAccount
}

pub struct Client {
    account: Option<UserAccount>,
    http_client: reqwest::Client,
    squirrel_gw: Arc<Url>,
}

impl Client {

    pub fn new(squirrel_gw: Arc<Url>) -> Self {
        Self {
            account: None,
            http_client: reqwest::Client::new(),
            squirrel_gw
        }
    }

    pub async fn register(&mut self) -> Result<bool, reqwest::Error> {

        let res = self.http_client.post(self.squirrel_gw.join("/register").unwrap()).send().await?;
        
        let res = res.text().await.unwrap();
        let user_account = serde_json::from_str::<UserAccount>(res.as_str()).unwrap();
        self.account = Some(user_account);

        Ok(true)
    }

    pub fn watch_events(&self) -> Result<EventSource, ClientError> {

        match &self.account {
            Some(account) => Ok(EventSource::get(self.squirrel_gw.join(&format!("/events/{}", account.id)).unwrap())),
            None => Err(ClientError::MissingAccount)
        }
    } 
}

pub struct SquirrelAdmin {
    http_client: reqwest::Client,
    squirrel_gw: Arc<Url>
}

impl SquirrelAdmin {
    pub fn new(squirrel_gw: Arc<Url>) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            squirrel_gw,
        }
    }

    pub async fn start_registration(&self) -> Result<reqwest::Response, reqwest::Error> {
        self.http_client.post(self.squirrel_gw.join("/admin/registration/start").unwrap()).send().await
    }
}

fn get_crate_bin(bin_name: &str) -> PathBuf {

    let mut path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    path.push("../target/debug");
    path.push(bin_name);

    path
}

pub fn start_squirrel() -> PtySession {

    let path = get_crate_bin("squirrel");
    rexpect::session::spawn(path.to_str().unwrap(), Some(300)).unwrap()
}

pub fn start_fake_reg() -> PtySession {

    let path = get_crate_bin("fake_reg");
    rexpect::session::spawn(path.to_str().unwrap(), Some(300)).unwrap()
}