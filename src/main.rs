use libc::mkfifo;
use std::ffi::CString;
use std::fs::File;
use std::io::{stdin, stdout, Read, Write, ErrorKind};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread::spawn;
use std::time::Duration;

const PATH: &str = "/tmp/debug";
const LISTEN_PORT: u16 = 62100;
const DEBUG_MODE: bool = false;

fn create_listener() -> TcpListener {
    let listen_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), LISTEN_PORT);
    TcpListener::bind(listen_address).expect("Failed to setup listener")
}

fn handle_connection(mut conn: TcpStream) {
    conn.set_nonblocking(true)
        .expect("Failed to set non blocking");
    conn.set_nodelay(true)
        .expect("Failed to set socket to nodelay");

    let (tx_stdin, rx_stdin): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = channel();
    let (tx_file, rx_file): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = channel();
    let mut buffer = [0; 1];

    begin_reading_from_stdin(tx_stdin);
    begin_reading_from_file(tx_file);

    loop {
        std::thread::sleep(Duration::from_millis(10));

        if DEBUG_MODE {
            println!("Loop start");
        }

        let err = conn.take_error().expect("Checking for error on conn failed");
        if let Some(err) = err {
            println!("Error on socket: {}", err);
            break;
        }

        if DEBUG_MODE {
            println!("Receiving from file");
        }
        match rx_file.try_recv() {
            Ok(data) => conn
                .write_all(&data)
                .expect("Failed to write file data back to socket"),
            Err(e) => match e {
                TryRecvError::Empty => {}
                TryRecvError::Disconnected => println!("File disconnected"),
            },
        }

        if DEBUG_MODE {
            println!("Receiving from stdin");
        }
        match rx_stdin.try_recv() {
            Ok(data) => conn
                .write_all(&data)
                .expect("Failed to write stdin data back to socket"),
            Err(e) => match e {
                TryRecvError::Empty => {}
                TryRecvError::Disconnected => println!("Stdin disconnected"),
            },
        }

        if DEBUG_MODE {
            println!("Receiving from socket");
        }
        match conn.read(&mut buffer) {
            Ok(_count) => {
                print!("{}", buffer[0] as char);
                stdout().flush().expect("Failed to flush stdout");
            },
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {},
            Err(ref e) if e.kind() == ErrorKind::ConnectionReset => break,
            Err(ref e) if e.kind() == ErrorKind::ConnectionAborted => break,
            Err(ref e) if e.kind() == ErrorKind::NotConnected => break,
            Err(e) => println!("Failed to read from socket: {}", e)
        }
    }
}

fn begin_reading_from_stdin(tx_stdin: Sender<Vec<u8>>) {
    spawn(move || {
        let mut stdin = stdin();
        let mut buf = [0; 1];
        loop {
            std::thread::sleep(Duration::from_millis(10));
            match stdin.read(&mut buf) {
                Ok(count) => {
                    let data = &buf[0..count];
                    tx_stdin.send(data.to_vec()).expect("Failed to send stdin");
                }
                Err(e) => println!("Failed to read from stdin: {}", e),
            }
        }
    });
}

fn begin_reading_from_file(tx_stdin: Sender<Vec<u8>>) {
    spawn(move || {
        create_debug_file();

        let mut file = File::open(PATH).unwrap();
        let mut buf = [0; 1];
        loop {
            std::thread::sleep(Duration::from_millis(10));
            match file.read(&mut buf) {
                Ok(count) => {
                    let data = &buf[0..count];
                    tx_stdin.send(data.to_vec()).expect("Failed to send file");
                }
                Err(e) => println!("Failed to read from stdin: {}", e),
            }
        }
    });
}

fn create_debug_file() {
    unsafe {
        let filename = CString::new(PATH).unwrap();
        mkfifo(filename.as_ptr(), 0o644);
    }
}

fn main() {
    println!("Starting listener on port '{}'...", LISTEN_PORT);

    let listener = create_listener();

    for incoming in listener.incoming() {
        println!("Received new connection");
        match incoming {
            Ok(stream) => {
                spawn(move || {
                    handle_connection(stream);
                });
            },
            Err(e) => println!("Connection failed: {}", e),
        }
    }
}
