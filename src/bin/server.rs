use tokio::net::{TcpListener, TcpStream};
use mini_redis::{Connection, Frame};
use mini_redis::Command::{Get,Set,self};
use std::collections::HashMap;
use bytes::Bytes;
use std::sync::{Arc, Mutex};

type Db = Arc<Mutex<HashMap<String,Bytes>>>;
type ShardDb = Arc<Vec<Mutex<HashMap<String,Vec<u8>>>>>;

struct CanIncrement {
    mutex: Mutex<i32>,
}
impl CanIncrement {
    // This function is not marked async.
    fn increment(&self) {
        let mut lock = self.mutex.lock().unwrap();
        *lock += 1;
    }
}

async fn increment_and_do_stuff(can_incr: &CanIncrement) {
    can_incr.increment();
    do_something_async().await;
}


#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("localhost:6379").await.unwrap();

    let db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        let db = db.clone();

        let handle = tokio::spawn(async move{
            process(socket,db).await;
        });

        let out = handle.await.unwrap();
        println!("got {:?}",out)
    }
}

fn insert(db: ShardDb,key: &str, value: &str) {
    let shard = db[hash(key) % db.len()].lock().unwrap();

    shard.insert(key,value);
}

fn new_shard_db(n : usize) -> ShardDb {
    let mut db = Vec::with_capacity(n);

    for _ in 0..n {
        db.push(Mutex::new(HashMap::new()));
    }

    return Arc::new(db);
}

async fn process(socket: TcpStream, db: Db) {
    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                let mut db = db.lock().unwrap();
                // The value is stored as `Vec<u8>`
                db.insert(cmd.key().to_string(), cmd.value().clone());

                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                let db = db.lock().unwrap();

                if let Some(value) = db.get(cmd.key()) {
                    // `Frame::Bulk` expects data to be of type `Bytes`. This
                    // type will be covered later in the tutorial. For now,
                    // `&Vec<u8>` is converted to `Bytes` using `into()`.
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };

        // Write the response to the client
        connection.write_frame(&response).await.unwrap();
    }
}