use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter},
    path::Path
};
use crate::jvm::SessionInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredAccount {
    pub name: String,
    pub username: String,
    pub player_id: String,
    pub access_token: String,
    pub session_type: String,
    pub created_at: u64,
    pub last_used: Option<u64>,
}

impl StoredAccount {
    pub fn new(name: String, session: SessionInfo) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            name,
            username: session.username,
            player_id: session.player_id,
            access_token: session.access_token,
            session_type: session.session_type,
            created_at: now,
            last_used: None,
        }
    }

    pub fn to_session_info(&self) -> SessionInfo {
        SessionInfo {
            username: self.username.clone(),
            player_id: self.player_id.clone(),
            access_token: self.access_token.clone(),
            session_type: self.session_type.clone(),
        }
    }

    pub fn update_last_used(&mut self) {
        self.last_used = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
    }

    pub fn format_created_date(&self) -> String {
        self.format_time(Some(self.created_at))
    }

    pub fn format_last_used(&self) -> String {
        self.format_time(self.last_used)
    }

    fn format_time(&self, time: Option<u64>) -> String {
        if let Some(time) = time {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let diff = now.saturating_sub(time);

            if diff < 60 {
                "Just now".to_string()
            } else if diff < 3600 {
                format!("{} minutes ago", diff / 60)
            } else if diff < 86400 {
                format!("{} hours ago", diff / 3600)
            } else if diff < 2592000 { // 30 days
                format!("{} days ago", diff / 86400)
            } else if diff < 31536000 { // 365 days
                format!("{} months ago", diff / 2592000)
            } else {
                format!("{} years ago", diff / 31536000)
            }
        } else {
            "Never".to_string()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountStorage {
    pub accounts: HashMap<String, StoredAccount>,
    pub version: u32,
}

impl Default for AccountStorage {
    fn default() -> Self {
        Self {
            accounts: HashMap::new(),
            version: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AccountManager {
    storage: AccountStorage,
    config_path: String,
}

impl AccountManager {
    pub fn new() -> Self {
        let config_path = "sessions.json".to_string();
        let mut manager = Self {
            storage: AccountStorage::default(),
            config_path,
        };

        if let Err(e) = manager.load_accounts() {
            tracing::warn!("Failed to load accounts: {}", e);
        }

        manager
    }

    pub fn load_accounts(&mut self) -> Result<(), String> {
        if !Path::new(&self.config_path).exists() {
            return Ok(());
        }

        let file = File::open(&self.config_path)
            .map_err(|e| format!("Failed to open config file: {}", e))?;

        let reader = BufReader::new(file);
        self.storage = serde_json::from_reader(reader)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        tracing::info!("Loaded {} accounts from config", self.storage.accounts.len());
        Ok(())
    }

    pub fn save_accounts(&self) -> Result<(), String> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.config_path)
            .map_err(|e| format!("Failed to create config file: {}", e))?;

        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.storage)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        tracing::info!("Saved {} accounts to config", self.storage.accounts.len());
        Ok(())
    }

    pub fn add_account(&mut self, name: String, session: SessionInfo) -> Result<(), String> {
        if self.storage.accounts.contains_key(&name) {
            return Err("Account with this name already exists".to_string());
        }

        let account = StoredAccount::new(name.clone(), session);
        self.storage.accounts.insert(name, account);
        self.save_accounts()?;

        Ok(())
    }

    pub fn remove_account(&mut self, name: &str) -> Result<(), String> {
        if !self.storage.accounts.contains_key(name) {
            return Err("Account not found".to_string());
        }

        self.storage.accounts.remove(name);
        self.save_accounts()?;

        Ok(())
    }

    pub fn get_account(&self, name: &str) -> Option<&StoredAccount> {
        self.storage.accounts.get(name)
    }


    pub fn update_account(&mut self, name: &str, session: SessionInfo) -> Result<(), String> {
        if let Some(account) = self.storage.accounts.get_mut(name) {
            account.username = session.username;
            account.player_id = session.player_id;
            account.access_token = session.access_token;
            account.session_type = session.session_type;
            account.update_last_used();
            self.save_accounts()?;
            Ok(())
        } else {
            Err("Account not found".to_string())
        }
    }

    pub fn use_account(&mut self, name: &str) -> Result<SessionInfo, String> {
        if let Some(account) = self.storage.accounts.get_mut(name) {
            account.update_last_used();
            let session = account.to_session_info();
            self.save_accounts()?;
            Ok(session)
        } else {
            Err("Account not found".to_string())
        }
    }

    pub fn get_all_accounts(&self) -> Vec<StoredAccount> {
        let mut accounts: Vec<StoredAccount> = self.storage.accounts.values().cloned().collect();
        accounts.sort_by(|a, b| {
            match (a.last_used, b.last_used) {
                (Some(a_time), Some(b_time)) => b_time.cmp(&a_time),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => b.created_at.cmp(&a.created_at),
            }
        });
        accounts
    }

    pub fn rename_account(&mut self, old_name: &str, new_name: String) -> Result<(), String> {
        if old_name == new_name {
            return Ok(());
        }

        if self.storage.accounts.contains_key(&new_name) {
            return Err("Account with this name already exists".to_string());
        }

        if let Some(mut account) = self.storage.accounts.remove(old_name) {
            account.name = new_name.clone();
            self.storage.accounts.insert(new_name, account);
            self.save_accounts()?;
            Ok(())
        } else {
            Err("Account not found".to_string())
        }
    }

    pub fn export_to_clipboard(&self, name: &str) -> Result<String, String> {
        if let Some(account) = self.storage.accounts.get(name) {
            let json = serde_json::to_string_pretty(account)
                .map_err(|e| format!("Failed to serialize account: {}", e))?;
            Ok(json)
        } else {
            Err("Account not found".to_string())
        }
    }

    pub fn get_account_count(&self) -> usize {
        self.storage.accounts.len()
    }
}