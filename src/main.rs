use std::io::{Write, BufReader, BufRead, Read};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::net::TcpStream;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// FTP server address
    #[arg(short, long, default_value = "127.0.0.1")]
    address: String,
    
    /// FTP server port
    #[arg(short, long, default_value = "21")]
    port: u16,
    
    /// FTP username
    #[arg(short, long, default_value = "anonymous")]
    username: String,
    
    /// FTP password
    #[arg(short = 'P', long, default_value = "anonymous")]
    password: String,
    
    /// Chat database filename
    #[arg(short, long, default_value = "chat.txt")]
    database: String,
    
    /// Your nickname for the chat
    #[arg(short, long)]
    nickname: Option<String>,
}

#[derive(Clone)]
struct ChatClient {
    server: String,
    port: u16,
    username: String,
    password: String,
    chat_file: String,
    nickname: String,
    last_message_count: Arc<Mutex<usize>>,
    last_modification_time: Arc<Mutex<Option<String>>>,
}

impl ChatClient {
    fn new(args: Args) -> Result<Self, Box<dyn std::error::Error>> {
        println!("=== Welcome to ChatFTP ===");
        
        // Get nickname from args or prompt for it
        let nickname = match args.nickname {
            Some(nick) => nick,
            None => {
                println!("Enter your nickname: ");
                read_input()?
            }
        };
        
        println!("Connecting as '{}' to {}:{}", nickname, args.address, args.port);
        println!("Using chat database: {}", args.database);
        
        Ok(ChatClient {
            server: args.address,
            port: args.port,
            username: args.username,
            password: args.password,
            chat_file: args.database,
            nickname,
            last_message_count: Arc::new(Mutex::new(0)),
            last_modification_time: Arc::new(Mutex::new(None)),
        })
    }
    
    fn connect(&self) -> Result<TcpStream, Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.server, self.port);
        println!("Connecting to {}...", addr);
        
        let mut stream = TcpStream::connect(&addr)?;
        
        // Read welcome message
        let _ = self.read_ftp_response(&mut stream)?;
        
        // Send USER command
        self.send_ftp_command(&mut stream, &format!("USER {}", self.username))?;
        let user_response = self.read_ftp_response(&mut stream)?;
        if !user_response.starts_with("331") && !user_response.starts_with("230") {
            return Err(format!("FTP USER command failed: {}", user_response).into());
        }
        
        // Send PASS command
        self.send_ftp_command(&mut stream, &format!("PASS {}", self.password))?;
        let pass_response = self.read_ftp_response(&mut stream)?;
        if !pass_response.starts_with("230") {
            return Err(format!("FTP authentication failed: {}", pass_response).into());
        }
        
        println!("Successfully connected to FTP server!");
        Ok(stream)
    }
    
    fn send_ftp_command(&self, stream: &mut TcpStream, command: &str) -> Result<(), Box<dyn std::error::Error>> {
        let cmd = format!("{}\r\n", command);
        stream.write_all(cmd.as_bytes())?;
        stream.flush()?;
        Ok(())
    }
    
    fn read_ftp_response(&self, stream: &mut TcpStream) -> Result<String, Box<dyn std::error::Error>> {
        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_line(&mut response)?;
        Ok(response.trim().to_string())
    }
    
    fn establish_passive_connection(&self, stream: &mut TcpStream) -> Result<TcpStream, Box<dyn std::error::Error>> {
        self.send_ftp_command(stream, "PASV")?;
        let response = self.read_ftp_response(stream)?;
        
        if !response.starts_with("227") {
            return Err(format!("PASV command failed: {}", response).into());
        }
        
        // Parse PASV response to get IP and port
        let start = response.find('(').ok_or("Invalid PASV response")?;
        let end = response.find(')').ok_or("Invalid PASV response")?;
        let pasv_data = &response[start + 1..end];
        let parts: Vec<&str> = pasv_data.split(',').collect();
        
        if parts.len() != 6 {
            return Err("Invalid PASV response format".into());
        }
        
        let ip = format!("{}.{}.{}.{}", parts[0], parts[1], parts[2], parts[3]);
        let port = parts[4].parse::<u16>()? * 256 + parts[5].parse::<u16>()?;
        let pasv_addr = format!("{}:{}", ip, port);
        
        Ok(TcpStream::connect(pasv_addr)?)
    }
    
    fn send_message(&self, stream: &mut TcpStream, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let formatted_message = format!("{} {}: {}\n", timestamp, self.nickname, message);
        
        let mut pasv_stream = self.establish_passive_connection(stream)?;
        
        self.send_ftp_command(stream, &format!("APPE {}", self.chat_file))?;
        let response = self.read_ftp_response(stream)?;
        
        if response.starts_with("150") {
            pasv_stream.write_all(formatted_message.as_bytes())?;
            pasv_stream.shutdown(std::net::Shutdown::Both)?;
            
            // Read completion response
            let _ = self.read_ftp_response(stream)?;
            
            // Don't display immediately - wait for server update via polling
            // This way we see our message with timestamp when it's confirmed on server
            
            Ok(())
        } else {
            Err(format!("Failed to append to file: {}", response).into())
        }
    }
    
    fn get_file_modification_time(&self, stream: &mut TcpStream) -> Result<Option<String>, Box<dyn std::error::Error>> {
        self.send_ftp_command(stream, &format!("MDTM {}", self.chat_file))?;
        let response = self.read_ftp_response(stream)?;
        
        if response.starts_with("213") {
            // MDTM response format: "213 YYYYMMDDHHMMSS"
            let parts: Vec<&str> = response.split_whitespace().collect();
            if parts.len() >= 2 {
                Ok(Some(parts[1].to_string()))
            } else {
                Ok(None)
            }
        } else {
            // MDTM not supported or file doesn't exist
            Ok(None)
        }
    }

    fn retrieve_messages(&self, stream: &mut TcpStream) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut pasv_stream = self.establish_passive_connection(stream)?;
        
        self.send_ftp_command(stream, &format!("RETR {}", self.chat_file))?;
        let response = self.read_ftp_response(stream)?;
        
        if response.starts_with("150") {
            let mut content = String::new();
            pasv_stream.read_to_string(&mut content)?;
            
            // Read completion response
            let _ = self.read_ftp_response(stream)?;
            
            let messages: Vec<String> = content
                .lines()
                .map(|line| line.to_string())
                .filter(|line| !line.trim().is_empty())
                .collect();
            
            Ok(messages)
        } else if response.starts_with("550") {
            // File doesn't exist yet, that's okay
            Ok(Vec::new())
        } else {
            Err(format!("Failed to retrieve file: {}", response).into())
        }
    }
    
    fn display_chat_history(&self, stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n=== Chat History ===");
        match self.retrieve_messages(stream) {
            Ok(messages) => {
                if messages.is_empty() {
                    println!("No messages yet. Start the conversation!");
                } else {
                    for message in &messages {
                        println!("{}", message);
                    }
                    // Update last message count
                    if let Ok(mut count) = self.last_message_count.lock() {
                        *count = messages.len();
                    }
                }
            }
            Err(e) => {
                println!("Could not load chat history: {}", e);
            }
        }
        println!("===================\n");
        Ok(())
    }
    
    fn poll_for_new_messages(&self, stream: TcpStream) {
        let client = self.clone();
        
        thread::spawn(move || {
            let mut stream = stream;
            loop {
                thread::sleep(Duration::from_millis(2000)); // Poll every 2 seconds
                
                // Check if file has been modified using MDTM (if supported)
                let should_download = match client.get_file_modification_time(&mut stream) {
                    Ok(Some(current_mod_time)) => {
                        let last_mod_time = {
                            let mod_time_guard = client.last_modification_time.lock().unwrap();
                            mod_time_guard.clone()
                        };
                        
                        // If we don't have a stored mod time or it has changed, download
                        match last_mod_time {
                            None => true, // First time, download
                            Some(last_time) => last_time != current_mod_time, // Download if changed
                        }
                    }
                    Ok(None) => {
                        // MDTM not supported or file doesn't exist, fallback to always downloading
                        true
                    }
                    Err(_) => {
                        // Error checking MDTM, fallback to downloading
                        true
                    }
                };
                
                if should_download {
                    match client.retrieve_messages(&mut stream) {
                        Ok(messages) => {
                            // Update modification time after successful download
                            if let Ok(Some(current_mod_time)) = client.get_file_modification_time(&mut stream) {
                                if let Ok(mut mod_time_guard) = client.last_modification_time.lock() {
                                    *mod_time_guard = Some(current_mod_time);
                                }
                            }
                            
                            let current_count = messages.len();
                            let last_count = {
                                let count_guard = client.last_message_count.lock().unwrap();
                                *count_guard
                            };
                            
                            if current_count > last_count {
                                // Display ALL new messages (including our own from server confirmation)
                                let new_messages: Vec<_> = messages.iter().skip(last_count).collect();
                                
                                if !new_messages.is_empty() {
                                    // Clear the current line and move cursor to beginning
                                    print!("\r\x1b[K");
                                    std::io::stdout().flush().unwrap();
                                    
                                    // Display all new messages with timestamps
                                    for message in &new_messages {
                                        // Parse the message to display it IRC-style with timestamp
                                        if let Some(colon_pos) = message.find(": ") {
                                            let (timestamp_and_user, content) = message.split_at(colon_pos + 2);
                                            let timestamp = &timestamp_and_user[0..19]; // "YYYY-MM-DD HH:MM:SS"
                                            let user_part = &timestamp_and_user[20..]; // Skip timestamp "YYYY-MM-DD HH:MM:SS "
                                            let username = user_part.trim_end_matches(": ");
                                            println!("[{}] <{}> {}", timestamp, username, content);
                                        } else {
                                            // Fallback for system messages or malformed messages
                                            println!("{}", message);
                                        }
                                    }
                                    
                                    // Reprint the prompt on a new line
                                    print!("Enter a message: ");
                                    std::io::stdout().flush().unwrap();
                                }
                                
                                // Update count
                                if let Ok(mut count) = client.last_message_count.lock() {
                                    *count = current_count;
                                }
                            }
                        }
                        Err(_) => {
                            // Silently continue if there's an error polling
                            // This prevents spam when connection issues occur
                        }
                    }
                }
            }
        });
    }
    
    fn start_chat(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut stream = self.connect()?;
        
        // Display existing chat history
        self.display_chat_history(&mut stream)?;
        
        // Send join notification
        self.send_message(&mut stream, &format!("*** {} has joined the chat ***", self.nickname))?;
        
        // Start polling for new messages in a separate thread
        let poll_stream = stream.try_clone()?;
        self.poll_for_new_messages(poll_stream);
        
        println!("Type your messages below. Use /exit or /quit to leave the chat.");
        println!("Commands: /help - show this help, /history - refresh chat history\n");
        
        loop {
            print!("Enter a message: ");
            std::io::stdout().flush()?;
            
            let input = read_input()?;
            
            // Clear the input line that shows "Enter a message: [user input]"
            // Move cursor up one line, then clear it
            print!("\x1b[1A\r\x1b[K");
            std::io::stdout().flush()?;
            
            match input.as_str() {
                "/exit" | "/quit" => {
                    self.send_message(&mut stream, &format!("*** {} has left the chat ***", self.nickname))?;
                    break;
                }
                "/help" => {
                    println!("\nAvailable commands:");
                    println!("/exit, /quit - Leave the chat");
                    println!("/help - Show this help");
                    println!("/history - Refresh and display chat history\n");
                }
                "/history" => {
                    self.display_chat_history(&mut stream)?;
                }
                _ => {
                    if !input.trim().is_empty() {
                        self.send_message(&mut stream, &input)?;
                    }
                }
            }
        }
        
        println!("Disconnected from chat. Goodbye!");
        Ok(())
    }
}

fn read_input() -> Result<String, Box<dyn std::error::Error>> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn main() {
    let args = Args::parse();
    
    match ChatClient::new(args) {
        Ok(client) => {
            if let Err(e) = client.start_chat() {
                eprintln!("Chat error: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Setup error: {}", e);
            std::process::exit(1);
        }
    }
}
