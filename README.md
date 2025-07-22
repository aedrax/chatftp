# ChatFTP
You've heard of chatGPT, well now you have chatFTP!
Unfortunately, the only relevance is the funny name.

This is a simple chat client written in Rust that integrates with an FTP server to read and write
chat messages to a text file.

### Features

- **Real-time messaging**: Send and receive chat messages to/from other users connected to the same FTP server
- **Automatic message polling**: Background thread continuously checks for new messages every 2 seconds
- **Chat history**: Displays existing conversation history when joining the chat
- **User-friendly interface**: Clean command line interface with clear prompts and feedback
- **Configurable chat database**: Specify any filename for the chat storage file
- **Join/leave notifications**: Automatic announcements when users enter or exit the chat

### Prerequisites

- Rust installed on your machine (1.70+ recommended)
- Access to an FTP server that supports passive mode (PASV)
- Valid FTP username and password
- Network connectivity to the FTP server

### Installation

1. Clone this repository to your machine:
   ```bash
   git clone git@github.com:aedrax/chatftp.git
   cd chatftp
   ```
2. Build the project:
   ```bash
   cargo build --release
   ```
3. Run the chat client:
   ```bash
   cargo run
   ```

### Usage

ChatFTP uses command line arguments for configuration. Here are the available options:

```bash
Usage: chatftp [OPTIONS]

Options:
  -a, --address <ADDRESS>    FTP server address [default: 127.0.0.1]
  -p, --port <PORT>          FTP server port [default: 21]
  -u, --username <USERNAME>  FTP username [default: anonymous]
  -P, --password <PASSWORD>  FTP password [default: anonymous]
  -d, --database <DATABASE>  Chat database filename [default: chat.txt]
  -n, --nickname <NICKNAME>  Your nickname for the chat
  -h, --help                 Print help
  -V, --version              Print version
```

**All Arguments are Optional with Defaults:**
- `--address` / `-a`: FTP server address (default: 127.0.0.1)
- `--port` / `-p`: FTP server port (default: 21)
- `--username` / `-u`: FTP server username (default: anonymous)
- `--password` / `-P`: FTP server password (default: anonymous)
- `--database` / `-d`: Chat database filename (default: chat.txt)
- `--nickname` / `-n`: Your chat nickname (if not provided, you'll be prompted)

**Example Usage:**

1. **Basic usage with local FTP server:**
   ```bash
   cargo run -- --username myuser --password mypass --nickname Alice
   ```

2. **Connect to remote FTP server:**
   ```bash
   cargo run -- --address ftp.example.com --username myuser --password mypass --nickname Bob
   ```

3. **Custom database file and port:**
   ```bash
   cargo run -- --address 192.168.1.100 --port 2121 --username ftpuser --password secret --database team_chat.txt --nickname Charlie
   ```

4. **Help command:**
   ```bash
   cargo run -- --help
   ```

**Chat Interface:**
- View existing chat history automatically when joining
- Type messages and press Enter to send (your messages appear immediately)
- New messages from other users appear automatically in IRC-style format: `<username> message`
- Messages include timestamps in the file but display cleanly in the interface
- You won't see your own messages echoed back (like IRC behavior)

**Available Commands:**
- `/exit` or `/quit` - Leave the chat
- `/help` - Display available commands
- `/history` - Refresh and display complete chat history

**Example Session:**
```
$ cargo run -- --username testuser --password testpass --nickname Alice

=== Welcome to ChatFTP ===
Connecting as 'Alice' to 127.0.0.1:21
Using chat database: chat.txt
Connecting to 127.0.0.1:21...
Successfully connected to FTP server!

=== Chat History ===
2025-01-20 14:30:15 bob: Hello everyone!
2025-01-20 14:31:22 charlie: Hey Bob!
===================

Type your messages below. Use /exit or /quit to leave the chat.
Commands: /help - show this help, /history - refresh chat history

[2025-01-20 14:31:25] <alice> *** alice has joined the chat ***
[2025-01-20 14:31:28] <alice> hey!
[2025-01-20 14:31:28] <bob> hey alice!
Enter a message: 
```

### Technical Details

- **FTP Protocol**: Uses standard FTP commands (USER, PASS, PASV, APPE, RETR, MDTM)
- **File Operations**: Appends new messages and retrieves entire chat history
- **Optimization**: Uses MDTM command to check file modification time before downloading (when supported)
- **Concurrency**: Separate thread for message polling to avoid blocking user input
- **Error Handling**: Graceful handling of network issues, authentication failures, and file errors
- **Message Format**: `YYYY-MM-DD HH:MM:SS Username: Message`

### Troubleshooting

- **Connection Issues**: Ensure FTP server allows passive mode connections
- **Authentication**: Verify username/password and account permissions
- **File Access**: Make sure the FTP account can read/write files in the working directory
- **Firewall**: Check that both control (usually 21) and data ports are accessible

### License

This project is licensed under the MIT License. See the LICENSE file for details.
