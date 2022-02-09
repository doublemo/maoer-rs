use std::sync::Arc;
use tokio::sync::{RwLock};
use dashmap::DashMap;
use crate::peer::Peer;
pub struct Context{
     pub peers:Arc<DashMap<String, Arc<RwLock<Peer>>>>
}

impl Context {
    pub fn new() -> Self {
        Context{
            peers: Arc::new(DashMap::new())
        }
    }
}

impl AsRef<Context> for Context {
    fn as_ref(&self) -> &Context{
        self
    }
}