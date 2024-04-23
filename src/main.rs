use std::io::{copy, Error, Read, Result, Write};
use std::net::{TcpListener, TcpStream};
use std::{env, thread};

const SOCKS_VERSION: u8 = 0x05;
const AUTHENTICATION_VERSION: u8 = 0x01;

fn print_help(binary_name: &str) {
    println!("Usage:\n  {} <bind_address>:<bind_port> <remote_address>:<remote_port> <username> <password>\n", binary_name);
}

fn main() {
    // - Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.contains(&String::from("--help")) {
        print_help(&args[0]);
        return;
    }

    if args.len() != 5 {
        println!("Invalid number of arguments. Use --help for usage information.");
        return;
    }

    // - Create a listener
    let local_listener = local(&args[1]).unwrap();
    println!("Listening on {}", args[1]);

    // - Handle incoming connections
    for stream in local_listener.incoming() {
        if let Err(e) = stream {
            println!("Failed to handle incoming connection: {}", e);
            continue;
        }
        client(stream.unwrap(), &args[2], &args[3], &args[4]).unwrap();
    }
}

fn local(bind_address: &str) -> Result<TcpListener> {
    // Bind to the local address
    TcpListener::bind(bind_address).map_err(|e| {
        Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to bind to local address: {}", e),
        )
    })
}

fn client(
    mut local_stream: TcpStream,
    remote_address: &str,
    username: &str,
    password: &str,
) -> Result<()> {
    // Greeting header
    let mut buffer: [u8; 2] = [0; 2];
    local_stream.read(&mut buffer[..])?;
    let _version = buffer[0]; // should be the same as SOCKS_VERSION
    let number_of_methods = buffer[1];

    // Authentication methods
    let mut methods: Vec<u8> = vec![];
    for _ in 0..number_of_methods {
        let mut next_method: [u8; 1] = [0; 1];
        local_stream.read(&mut next_method[..])?;
        methods.push(next_method[0]);
    }

    // Only accept no authentication
    if !methods.contains(&0x00) {
        // no acceptable methods were offered
        local_stream.write(&[SOCKS_VERSION, 0xFF])?;
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Method not supported",
        ))?
    }

    // We choose no authentication
    local_stream.write(&[SOCKS_VERSION, 0x00])?;

    // Create a TcpStream to the remote server
    let mut remote_stream: TcpStream = remote(remote_address, username, password)?;

    // Clone our streams
    let mut incoming_local = local_stream.try_clone()?;
    let mut incoming_remote = remote_stream.try_clone()?;

    // Copy the data from one to the other
    let handle_outgoing = thread::spawn(move || -> std::io::Result<()> {
        copy(&mut local_stream, &mut remote_stream)?;
        Ok(())
    });
    let handle_incoming = thread::spawn(move || -> std::io::Result<()> {
        copy(&mut incoming_remote, &mut incoming_local)?;
        Ok(())
    });

    // If we get any errors now its not our problem
    _ = handle_outgoing.join();
    _ = handle_incoming.join();

    // Quod Erat Demonstrandum
    Ok(())
}

fn remote(address: &str, username: &str, password: &str) -> Result<TcpStream> {
    // create a connection
    let mut remote_stream: TcpStream = TcpStream::connect(address).map_err(|e| {
        Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to connect to remote proxy: {}", e),
        )
    })?;

    // greeting header
    remote_stream.write(&[
        SOCKS_VERSION, // SOCKS version
        0x01,          // Number of authentication methods
        0x02,          // Username/password authentication
    ])?;

    // Receive the servers reply
    let mut buffer: [u8; 2] = [0; 2];
    remote_stream.read(&mut buffer)?;

    // Check the SOCKS version
    if buffer[0] != SOCKS_VERSION {
        Err(Error::new(
            std::io::ErrorKind::Other,
            format!("Server does not support socks version: {}", SOCKS_VERSION),
        ))?
    }

    // Check the authentication method
    if buffer[1] != 0x02 {
        Err(Error::new(
            std::io::ErrorKind::Other,
            "Server does not support username/password authentication",
        ))?
    }

    // Create a username/password negotiation request
    let mut auth_request = vec![
        AUTHENTICATION_VERSION, // Username/password authentication version
    ];
    auth_request.push(username.len() as u8); // Username length
    auth_request.extend_from_slice(username.as_bytes());
    auth_request.push(password.len() as u8); // Password length
    auth_request.extend_from_slice(password.as_bytes());

    // Send the username/password negotiation request
    remote_stream.write(&auth_request)?;

    // Receive the username/password negotiation reply/welcome message
    let mut buffer: [u8; 2] = [0; 2];
    remote_stream.read(&mut buffer)?;

    // Check the username/password authentication version
    if buffer[0] != AUTHENTICATION_VERSION {
        Err(Error::new(
            std::io::ErrorKind::Other,
            format!(
                "Unsupported username/password authentication version: {}",
                buffer[0]
            ),
        ))?
    }

    // Check the username/password authentication status
    if buffer[1] != 0x00 {
        Err(Error::new(
            std::io::ErrorKind::Other,
            "Server did not accept username/password",
        ))?
    }

    // Return the stream
    Ok(remote_stream)
}
