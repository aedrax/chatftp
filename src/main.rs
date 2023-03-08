use std::io::{Write, BufReader, BufRead};

fn read_input() -> String {
    let mut message = String::new();
    std::io::stdin().read_line(&mut message).unwrap();

    // remove the newline character
    return message.trim().to_string();
}

fn send_message(stream: &mut std::net::TcpStream, username: &str, message: &str) {
    // prepend the timestamp and username to the message
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let formatted_message = format!("{} {}: {}\n", timestamp, username, message);

    // send the FTP command to initiate a passive data connection
    stream.write("PASV\n".as_bytes()).unwrap();

    // read the response from the server
    let mut reader = BufReader::new(&*stream);
    let mut response = String::new();

    // keep reading a line until we get a 227 response
    while !response.starts_with("227") {
        response = String::new();
        reader.read_line(&mut response).unwrap();
    }

    // parse the response to get the ip and port
    // and example response would look like the following:
    // 227 Entering Passive Mode (127,0,0,1,82,13).
    let pasv_ip_port = response
        .split("(").collect::<Vec<&str>>()[1]
        .split(")").collect::<Vec<&str>>()[0]
        .split(",").collect::<Vec<&str>>();

    // create a socket address from the ip and port
    // Parse the pasv_ip_port array, and convert it to a string
    let pasv_ip = format!("{}.{}.{}.{}", pasv_ip_port[0], pasv_ip_port[1], pasv_ip_port[2], pasv_ip_port[3]);

    // need to parse the port as a u16
    let pasv_port = (pasv_ip_port[4].parse::<u16>().unwrap() * 256) + pasv_ip_port[5].parse::<u16>().unwrap();
    let pasv_addr = format!("{}:{}", pasv_ip, pasv_port);

    // create a TCP stream to the server
    let mut pasv_stream = std::net::TcpStream::connect(pasv_addr).unwrap();

    // append the message file to the chat.db file on the FTP server
    stream.write("APPE chat.db\n".as_bytes()).unwrap();    

    // write the message to the passive data connection
    pasv_stream.write(formatted_message.as_bytes()).unwrap();

    // close the passive data connection
    pasv_stream.shutdown(std::net::Shutdown::Both).unwrap();
}

// fn poll_for_messages(stream: &mut std::net::TcpStream) {
//     // create a new thread to continuously read from the server
//     let mut stream = stream.try_clone().unwrap();
//     std::thread::spawn(move || {
//         loop {
//             // send the FTP command to initiate a passive data connection
//             stream.write("PASV\n".as_bytes()).unwrap();

//             // read the response from the server
//             let mut reader = BufReader::new(&stream);
//             let mut response = String::new();
//             reader.read_line(&mut response).unwrap();

//             // parse the response to get the ip and port
//             // and example response would look like the following:
//             // 227 Entering Passive Mode (127,0,0,1,82,13).
//             let pasv_ip_port = response
//                 .split("(").collect::<Vec<&str>>()[1]
//                 .split(")").collect::<Vec<&str>>()[0]
//                 .split(",").collect::<Vec<&str>>();

//             // create a socket address from the ip and port
//             // Parse the pasv_ip_port array, and convert it to a string
//             let pasv_ip = format!("{}.{}.{}.{}", pasv_ip_port[0], pasv_ip_port[1], pasv_ip_port[2], pasv_ip_port[3]);

//             // need to parse the port as a u16
//             let pasv_port = (pasv_ip_port[4].parse::<u16>().unwrap() * 256) + pasv_ip_port[5].parse::<u16>().unwrap();
//             let pasv_addr = format!("{}:{}", pasv_ip, pasv_port);

//             // create a TCP stream to the server
//             let mut pasv_stream = std::net::TcpStream::connect(pasv_addr).unwrap();



//             // send the FTP command to download the chat.db file or create it if it doesn't exist
//             stream.write("RETR chat.d\n ".as_bytes()).unwrap();

//             // read the response from the server
//             let mut response = String::new();
//             stream.read_to_string(&mut response).unwrap();

//             // print the response
//             println!("{}", response);

//             // sleep for 500 milliseconds
//             std::thread::sleep(std::time::Duration::from_millis(500));
//         }
//     });
// }

fn main() {
    // prompt the user for a server ip or hostname
    println!("Enter a server ip or hostname: ");
    let server = read_input();

    // prompt the user for a port
    println!("Enter a port: ");
    let port = read_input();

    // create a socket address
    let addr = format!("{}:{}", server, port);
    // let addr = addr.parse::<std::net::SocketAddr>().unwrap();

    // create a TCP stream to the server
    let mut stream = std::net::TcpStream::connect(addr).unwrap();

    // print a welcome message
    println!("Connected to {}", stream.peer_addr().unwrap());

    // prompt for a username
    println!("Enter FTP server username: ");
    let server_user = read_input();

    // prompt for a password
    println!("Enter FTP server password: ");
    let password = read_input();

    // send the username and password to the server as FTP commands
    stream.write(format!("USER {}\n", server_user).as_bytes()).unwrap();
    stream.write(format!("PASS {}\n", password).as_bytes()).unwrap();

    // prompt the user for a nickname
    println!("Enter a nickname: ");
    let username = read_input();

    // notify the chat that the user has joined
    send_message(&mut stream, "Note", format!("{} has joined the chat", username).as_str());

    loop {
        // prompt the user for a message
        println!("Enter a message: ");
        let message = read_input();

        // if the message is "/exit or /quit", break out of the loop
        if message == "/exit" || message == "/quit" {
            break;
        }

        send_message(&mut stream, &username, &message);
    }
    
    // notify the chat that the user has left
    send_message(&mut stream, "Note", format!("{} has left the chat", username).as_str());

    // close the connection
    stream.shutdown(std::net::Shutdown::Both).unwrap();

    // print a goodbye message
    println!("Disconnected from {}", stream.peer_addr().unwrap());
}
