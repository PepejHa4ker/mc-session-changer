use once_cell::sync::Lazy;
use std::sync::Mutex;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

pub static ASYNC_RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Runtime::new().expect("Failed to create tokio runtime")
});

#[derive(Debug, Clone)]
pub enum AuthResult {
    Success {
        account_name: Option<String>,
        access_token: String,
        profile: String,
        username: String,
    },
    Error {
        account_name: Option<String>,
        message: String,
    },
}

type AuthResultSender = mpsc::UnboundedSender<AuthResult>;
type AuthResultReceiver = mpsc::UnboundedReceiver<AuthResult>;

static AUTH_CHANNEL: Lazy<Mutex<(AuthResultSender, Option<AuthResultReceiver>)>> = Lazy::new(|| {
    let (tx, rx) = mpsc::unbounded_channel();
    Mutex::new((tx, Some(rx)))
});

pub fn get_auth_sender() -> AuthResultSender {
    AUTH_CHANNEL.lock().unwrap().0.clone()
}

pub fn take_auth_receiver() -> Option<AuthResultReceiver> {
    AUTH_CHANNEL.lock().unwrap().1.take()
}

pub fn spawn_auth_task(username: String, password: String, account_name: Option<String>) {
    let sender = get_auth_sender();
    let account_name_clone = account_name.clone();
    let username_clone = username.clone();

    ASYNC_RUNTIME.spawn(async move {
        tracing::info!("Starting async auth for {}", username);

        match crate::auth::auth(&username, &password).await {
            Ok(auth_data) => {
                let _ = sender.send(AuthResult::Success {
                    account_name: account_name_clone,
                    access_token: auth_data.access_token,
                    profile: auth_data.profile,
                    username: username_clone,
                });
            }
            Err(e) => {
                let _ = sender.send(AuthResult::Error {
                    account_name: account_name_clone,
                    message: format!("{}", e),
                });
            }
        }
    });
}
