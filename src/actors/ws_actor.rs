//! `ClientWebSocketConnection` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ClientWebSocketConnection`.

use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use uuid::Uuid;

/// Chat server sends this messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

/// Message for chat server communications

/// New chat session is created
#[derive(Message)]
#[rtype(result = "Result<Uuid, std::io::Error>")]
pub struct Connect {
    pub addr: Recipient<Message>,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: Uuid,
}

/// Send message to specific room
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    /// Id of the client session
    pub id: Uuid,
    /// Peer message
    pub msg: String,
    /// Room name
    pub room: String,
}

/// List of available rooms
pub struct ListRooms;

impl actix::Message for ListRooms {
    type Result = Vec<String>;
}

/// Join room, if room does not exists create new one.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    /// Client ID
    pub id: Uuid,

    /// Room name
    pub name: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Listen {
    /// Client ID
    pub id: Uuid,

    ///Prefix
    pub key_prefix: String,
}

/// New chat session is created
#[derive(Message)]
#[rtype(String)]
pub struct Test {}

/// `ClientWebSocketConnection` manages chat rooms and responsible for coordinating chat session.
///
/// Implementation is very naïve.
#[derive(Debug)]
pub struct ClientWebSocketConnection {
    sessions: HashMap<Uuid, Recipient<Message>>,
    pub rooms: HashMap<String, HashSet<Uuid>>,
    rng: ThreadRng,
    visitor_count: Arc<AtomicUsize>,
    pub prefix_listners: HashMap<String, HashSet<Uuid>>,
}

impl ClientWebSocketConnection {
    pub fn new(visitor_count: Arc<AtomicUsize>) -> ClientWebSocketConnection {
        // default room
        let mut rooms = HashMap::new();
        rooms.insert("main".to_owned(), HashSet::new());

        ClientWebSocketConnection {
            sessions: HashMap::new(),
            rooms,
            rng: rand::thread_rng(),
            visitor_count,
            prefix_listners: HashMap::new(),
        }
    }
}

impl ClientWebSocketConnection {
    /// Send message to all users in the room
    fn send_message(&self, room: &str, message: &str, skip_id: Uuid) {
        if let Some(sessions) = self.rooms.get(room) {
            for id in sessions {
                if *id != skip_id {
                    if let Some(addr) = self.sessions.get(id) {
                        addr.do_send(Message(message.to_owned()));
                    }
                }
            }
        }
    }
}

/// Make actor from `ClientWebSocketConnection`
impl Actor for ClientWebSocketConnection {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for ClientWebSocketConnection {
    type Result = Result<Uuid, std::io::Error>;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("Someone joined");

        // notify all users in same room
        //        self.send_message("main", "Someone joined", 0);

        // register session with random id
        let id = Uuid::new_v4();

        self.sessions.insert(id, msg.addr);

        // auto join session to main room
        self.rooms
            .entry("main".to_owned())
            .or_insert_with(HashSet::new)
            .insert(id);

        let count = self.visitor_count.fetch_add(1, Ordering::SeqCst);
        //        self.send_message("main", &format!("Total visitors {count}"), 0);

        // send id back
        Ok(id)
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for ClientWebSocketConnection {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected");

        let mut rooms: Vec<String> = Vec::new();

        // remove address
        if self.sessions.remove(&msg.id).is_some() {
            // remove session from all rooms
            for (name, sessions) in &mut self.rooms {
                if sessions.remove(&msg.id) {
                    rooms.push(name.to_owned());
                }
            }
        }
        // send message to other users
        for room in rooms {
            //            self.send_message(&room, "Someone disconnected", 0);
        }
    }
}

/// Handler for Message message.
impl Handler<ClientMessage> for ClientWebSocketConnection {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        self.send_message(&msg.room, msg.msg.as_str(), msg.id);
    }
}

/// Handler for `ListRooms` message.
impl Handler<ListRooms> for ClientWebSocketConnection {
    type Result = MessageResult<ListRooms>;

    fn handle(&mut self, _: ListRooms, _: &mut Context<Self>) -> Self::Result {
        let mut rooms = Vec::new();

        for key in self.rooms.keys() {
            rooms.push(key.to_owned())
        }

        MessageResult(rooms)
    }
}

/// Join room, send disconnect message to old room
/// send join message to new room
impl Handler<Join> for ClientWebSocketConnection {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let Join { id, name } = msg;
        let mut rooms = Vec::new();

        // remove session from all rooms
        for (n, sessions) in &mut self.rooms {
            if sessions.remove(&id) {
                rooms.push(n.to_owned());
            }
        }
        // send message to other users
        for room in rooms {
            //            self.send_message(&room, "Someone disconnected", 0);
        }

        self.rooms
            .entry(name.clone())
            .or_insert_with(HashSet::new)
            .insert(id);

        self.send_message(&name, "Someone connected", id);
    }
}

// Handler for listening to rooms
impl Handler<Listen> for ClientWebSocketConnection {
    type Result = ();

    fn handle(&mut self, msg: Listen, _: &mut Context<Self>) {
        let Listen { id, key_prefix } = msg;

        //        // remove session from all rooms
        //        for (n, sessions) in &mut self.rooms {
        //            if sessions.remove(&id) {
        //                rooms.push(n.to_owned());
        //            }
        //        }
        //        // send message to other users
        //        for room in rooms {
        //            self.send_message(&room, "Someone disconnected", 0);
        //        }

        self.prefix_listners.insert(key_prefix, HashSet::from([id]));

        //        self.send_message(&prefix_clone, "Someone connected", id);
    }
}

impl Handler<Test> for ClientWebSocketConnection {
    type Result = String;

    fn handle(&mut self, msg: Test, _: &mut Context<Self>) -> Self::Result {
        //        // remove session from all rooms
        //        for (n, sessions) in &mut self.rooms {
        //            if sessions.remove(&id) {
        //                rooms.push(n.to_owned());
        //            }
        //        }
        //        // send message to other users
        //        for room in rooms {
        //            self.send_message(&room, "Someone disconnected", 0);
        //        }
        //            self.send_message("main".to_string(), "Someone connected", id);
        //            let values = self.prefix_listners.values();
        return "That worked".to_string();

        //        self.send_message(&prefix_clone, "Someone connected", id);
    }
}
