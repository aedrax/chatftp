use std::io::{Read, Write};

fn main() {
    // prompt the user for a server ip or hostname
    println!("Enter a server ip or hostname: ");
    let mut server = String::new();
    std::io::stdin().read_line(&mut server).unwrap();
    
    // remove the newline character
    server = server.trim().to_string();

    // prompt the user for a port
    println!("Enter a port: ");
    let mut port = String::new();
    std::io::stdin().read_line(&mut port).unwrap();

    // remove the newline character
    port = port.trim().to_string();

    // create a socket address
    let addr = format!("{}:{}", server, port);
    let addr = addr.parse::<std::net::SocketAddr>().unwrap();

    // create a TCP stream to the server
    let mut stream = std::net::TcpStream::connect(addr).unwrap();

    // print a welcome message
    println!("Connected to {}", stream.peer_addr().unwrap());

    // prompt for a username
    println!("Enter a username: ");
    let mut username = String::new();
    std::io::stdin().read_line(&mut username).unwrap();

    // remove the newline character
    username = username.trim().to_string();

    // prompt for a password
    println!("Enter a password: ");
    let mut password = String::new();
    std::io::stdin().read_line(&mut password).unwrap();

    // remove the newline character
    password = password.trim().to_string();

    // send the username and password to the server as FTP commands
    stream.write(format!("USER {}\r\n", username).as_bytes()).unwrap();
    stream.write(format!("PASS {}\r\n", password).as_bytes()).unwrap();

    // create a new thread to continuously read from the server
    let mut stream_clone = stream.try_clone().unwrap();
    std::thread::spawn(move || {
        loop {
            // send the FTP command to download the chat.db file or create it if it doesn't exist
            stream_clone.write("RETR chat.db\r\n ".as_bytes()).unwrap();

            // read the response from the server
            let mut response = String::new();
            stream_clone.read_to_string(&mut response).unwrap();

            // print the response
            println!("{}", response);

            // sleep for 500 milliseconds
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });

    loop {
        // prompt the user for a message
        println!("Enter a message: ");
        let mut message = String::new();
        std::io::stdin().read_line(&mut message).unwrap();

        // remove the newline character
        message = message.trim().to_string();

        // if the message is "/exit or /quit", break out of the loop
        if message == "/exit" || message == "/quit" {
            break;
        }

        // create a tmp file to store the message
        let mut file = std::fs::File::create("tmp.txt").unwrap();

        // write the message to the file
        file.write_all(message.as_bytes()).unwrap();

        // append the message file to the chat.db file on the FTP server
        stream.write("APPE chat.db\r ".as_bytes()).unwrap();
    }

    // close the connection
    stream.shutdown(std::net::Shutdown::Both).unwrap();

    // print a goodbye message
    println!("Disconnected from {}", stream.peer_addr().unwrap());
}