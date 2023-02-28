# ChatFTP
This is a simple chat client written in Rust that integrates with an FTP server to read and write
chat messages to a text file.

### Features

- Connect to an FTP server to read and write chat messages to a text file
- Send and receive chat messages to/from other users connected to the same FTP server
- User-friendly command line interface

### Prerequisites

- Rust installed on your machine
- Access to an FTP server
- Username and password for the FTP server
- A text file on the FTP server to serve as the chat database

### Installation

1. Clone this repository to your machine.
2. Navigate to the project directory in your terminal.
3. Run the command `cargo build` to build the project.
4. If the build is successful, run the command `cargo run` to start the chat client.

### Usage

1. When the chat client starts, you will be prompted to enter the FTP server address, username, and
password. Enter the required information to connect to the server.
2. You will then be prompted to enter the name of the chat database file on the FTP server. Enter
the name of the file you will use to store chat messages.
3. Once you are connected, you can start sending and receiving chat messages. To send a message,
simply type your message and press enter. Your message will be sent to all other users connected to
the same FTP server.
4. To exit the chat client, type the command `/exit` and press enter.

### License

This project is licensed under the MIT License. See the LICENSE file for details.
