use super::{user::User, wallet::WalletEntry};
use crate::error_handler::SoliteError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct UserDb {
    inner: Arc<RwLock<UserDbInner>>,
}

struct UserDbInner {
    // user_id → User
    by_id: HashMap<String, User>,

    // username → user_id
    by_username: HashMap<String, String>,

    // email → user_id
    by_email: HashMap<String, String>,

    // google_id → user_id
    by_google: HashMap<String, String>,

    // address → user_id
    by_address: HashMap<String, String>,
}

impl UserDbInner {
    fn new() -> Self {
        Self {
            by_id: HashMap::new(),
            by_username: HashMap::new(),
            by_email: HashMap::new(),
            by_google: HashMap::new(),
            by_address: HashMap::new(),
        }
    }
}

impl UserDb {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(UserDbInner::new())),
        }
    }

    /// insert a brand new user
    pub async fn insert(&self, user: User) -> Result<(), SoliteError> {
        let mut db = self.inner.write().await;

        if db.by_username.contains_key(&user.username) {
            return Err(SoliteError::UsernameTaken(user.username));
        }
        if let Some(email) = &user.email {
            if db.by_email.contains_key(email) {
                return Err(SoliteError::EmailTaken(email.clone()));
            }
        }

        db.by_username
            .insert(user.username.clone(), user.id.clone());

        if let Some(ref email) = user.email {
            db.by_email.insert(email.clone(), user.id.clone());
        }

        if let Some(ref gid) = user.google_id {
            db.by_google.insert(gid.clone(), user.id.clone());
        }

        for wallet in &user.wallets {
            db.by_address
                .insert(wallet.address.clone(), user.id.clone());
        }

        db.by_id.insert(user.id.clone(), user);
        Ok(())
    }

    // add a wallet to an existing user
    pub async fn add_wallet(&self, user_id: &str, entry: WalletEntry) -> Result<(), SoliteError> {
        let mut db = self.inner.write().await;

        // check address not already owned by someone else
        if db.by_address.contains_key(&entry.address) {
            return Err(SoliteError::AddressAlreadyExists(entry.address));
        }

        let address = entry.address.clone();
        {
            let user = db
                .by_id
                .get_mut(user_id)
                .ok_or_else(|| SoliteError::UserNotFound(user_id.into()))?;
            user.wallets.push(entry);
        }
        db.by_address.insert(address, user_id.to_string());
        Ok(())
    }

    // update google_id for an existing user (link Google account)
    pub async fn link_google(&self, user_id: &str, google_id: String) -> Result<(), SoliteError> {
        let mut db = self.inner.write().await;

        if db.by_google.contains_key(&google_id) {
            return Err(SoliteError::GoogleAlreadyLinked);
        }

        {
            let user = db
                .by_id
                .get_mut(user_id)
                .ok_or_else(|| SoliteError::UserNotFound(user_id.into()))?;
            user.google_id = Some(google_id.clone());
        }
        db.by_google.insert(google_id, user_id.to_string());
        Ok(())
    }

    pub async fn get_by_id(&self, user_id: &str) -> Result<User, SoliteError> {
        let db = self.inner.read().await;
        db.by_id
            .get(user_id)
            .cloned()
            .ok_or_else(|| SoliteError::UserNotFound(user_id.into()))
    }

    pub async fn get_by_username(&self, username: &str) -> Result<User, SoliteError> {
        let db = self.inner.read().await;
        let id = db
            .by_username
            .get(username)
            .ok_or_else(|| SoliteError::UserNotFound(username.into()))?;
        db.by_id
            .get(id)
            .cloned()
            .ok_or_else(|| SoliteError::UserNotFound(username.into()))
    }

    pub async fn get_by_email(&self, email: &str) -> Result<User, SoliteError> {
        let db = self.inner.read().await;
        let id = db
            .by_email
            .get(email)
            .ok_or_else(|| SoliteError::UserNotFound(email.into()))?;
        db.by_id
            .get(id)
            .cloned()
            .ok_or_else(|| SoliteError::UserNotFound(email.into()))
    }

    pub async fn get_by_google(&self, google_id: &str) -> Result<User, SoliteError> {
        let db = self.inner.read().await;
        let id = db
            .by_google
            .get(google_id)
            .ok_or_else(|| SoliteError::UserNotFound(google_id.into()))?;
        db.by_id
            .get(id)
            .cloned()
            .ok_or_else(|| SoliteError::UserNotFound(google_id.into()))
    }

    // check if a user owns a specific address
    pub async fn owns_address(&self, user_id: &str, address: &str) -> bool {
        let db = self.inner.read().await;
        db.by_address
            .get(address)
            .map(|id| id == user_id)
            .unwrap_or(false)
    }

    // get the user_id that owns an address
    pub async fn owner_of(&self, address: &str) -> Option<String> {
        let db = self.inner.read().await;
        db.by_address.get(address).cloned()
    }

    pub async fn username_exists(&self, username: &str) -> bool {
        let db = self.inner.read().await;
        db.by_username.contains_key(username)
    }

    pub async fn email_exists(&self, email: &str) -> bool {
        let db = self.inner.read().await;
        db.by_email.contains_key(email)
    }

    pub async fn google_exists(&self, google_id: &str) -> bool {
        let db = self.inner.read().await;
        db.by_google.contains_key(google_id)
    }
}
