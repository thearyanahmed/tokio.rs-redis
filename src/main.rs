use tokio::net::{TcpListener, TcpStream};
use mini_redis::{Connection, Frame};

#[tokio::main]
async fn main() {

    let listener = TcpListener::bind("localhost:6379").await.unwrap();
    
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        process(socket).await;
    }
}

async fn process(socket: TcpStream) {
    let mut con = Connection::new(socket);

    if let Some(frame) = con.read_frame().await.unwrap() {
        println!("GOT : {:?}",frame);

        let res = Frame::Error("unimplemented".to_string());

        con.write_frame(&res).await.unwrap();
    }
}