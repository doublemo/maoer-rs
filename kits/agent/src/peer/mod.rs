use std::io;
use std::sync::Arc;
use std::collections::HashMap;
use futures::executor;
use tokio::sync::{RwLock};
use tokio::sync::mpsc::{self,UnboundedSender, UnboundedReceiver};
use bytes::BytesMut;
use tracing;
use lazy_static::*;


lazy_static! {
    static ref PEERS:Arc<RwLock<HashMap<String, UnboundedSender<BytesMut>>>> = {
        Arc::new(RwLock::new(HashMap::new()))
    }; 
}

#[derive(Debug)]
pub struct Peer {
    id:String,
    pub rx:UnboundedReceiver<BytesMut>,
}

impl Peer {
   pub async fn new(id:String) -> Peer {
       let (tx, rx) = mpsc::unbounded_channel();
       let peers = PEERS.clone();
       {
           let mut state = peers.write().await;
           state.insert(id.clone().to_string(), tx);
       }
       Peer {id:id, rx:rx}
   }

   pub fn get_id(&self) -> &str {
       &self.id
   }

   pub async fn send(&self, message:BytesMut) -> io::Result<()> {
       let peer_id = &self.id;
       let tx:UnboundedSender<BytesMut> = {
           let peers= PEERS.clone();
           let state = peers.read().await;
           if let Some(rx) = state.get(peer_id) {
               rx.clone().to_owned()
           } else {
               return Err(std::io::Error::new(std::io::ErrorKind::Other, "Please use a vector with at least one element".to_owned()))
           }
       };
       
       tx.send(message).unwrap();
       Ok(())
   }
}


impl AsRef<Peer> for Peer {
    fn as_ref(&self) -> &Peer{
        self
    }
}

impl AsMut<Peer> for Peer {
    fn as_mut(&mut self) -> &mut Peer {
        self
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        let peer_id = self.id.clone();
        let peers = PEERS.clone();
        executor::block_on(async move {
            let mut state = peers.write().await;
            state.remove(&peer_id);
            let count = state.keys().count();
            tracing::debug!("drop peer: {}, counter: {}", self.id, count);
        });
    }
}


pub(crate) fn handle_msg(peer:&Peer) {
    println!("hand ms -> {}", peer.get_id())
}