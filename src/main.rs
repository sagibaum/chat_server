use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;

fn main() {
    let server = TcpListener::bind(LOCAL).expect("Listener failed to bind");//127.0.0.1 for loop back and 6000 to the port
    server.set_nonblocking(true).expect("Failed to initialize non-blocking");// wrote to avoid blocking the message channel

    let mut clients = vec![];//vector for clients
    let (tx, rx) = mpsc::channel::<String>();//channels sender and receiver that will send and receive strings
    loop {
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connected", addr);
            let same_user=addr;
            let tx = tx.clone();
            clients.push(socket.try_clone().expect("Failed to clone client"));

            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_SIZE];
                match socket.read_exact(&mut buff) {
                    Ok(_) => {

                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();//taking only message that not equal to 0
                        let msg = String::from_utf8(msg).expect("Invalid utf8 message");//converting from UTF8 message to string
                        //let msg = String::from_utf8(msg).expect("Invalid utf8 message");//converting from UTF8 message to string

                        println!("{}: {:?}", addr, msg);
                        if addr!=same_user {  tx.send(msg).expect("Failed to send message to rx");}
                    },
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("Closing connection with {}", addr);
                        break;
                    }
                }

                sleep();
            });
        }
// collecting all those messages, resizing them and sending them to all of our clients !
        if let Ok(msg) = rx.try_recv() {
            clients = clients.into_iter().filter_map(|mut client| {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).map(|_| client).ok()
            })
                .collect::<Vec<_>>();
        }

        sleep();
    }
}

// I don't want the server to listen constantly to messages so i will sleep him
fn sleep() {
    thread::sleep(Duration::from_millis(100));
}
