use std::{net::TcpStream, io::{Write, BufReader}, io::BufRead, cell::{RefCell, Cell}, rc::Rc, process::Child};

static DATA_PREFIX: &str = "data:";

trait CommandBuilder<TOutput> {
    fn execute(&self, stream: &mut TcpStream) -> Result<TOutput, std::io::Error> {

        // get the client message and prepare to send it
        let client_message = self.get_client_message();
        let client_message = format!("{}\n", client_message);

        // send the client message to the running/connected program
        println!("CommandBuilder: execute: sending client message: \"{}\"", &client_message);
        let write_all_result = stream.write_all(client_message.as_bytes());
        if let Err(error) = write_all_result {
            println!("CommandBuilder: execute: write_all: error: {}", error);
            return Err(error);
        }
        let flush_result = stream.flush();
        if let Err(error) = flush_result {
            println!("CommandBuilder: execute: flush: error: {}", error);
            return Err(error);
        }

        // begin reading the response from the running/connected program
        let mut reader = BufReader::new(stream.try_clone().unwrap());

        let mut is_finished_reading = false;
        let mut is_status_message_received = false;
        let mut iteration_count = 0;
        while !is_finished_reading {
            println!("read iteration: {}", iteration_count);
            iteration_count += 1;

            let mut line = String::new();
            let read_line_result = reader.read_line(&mut line);
            if let Err(error) = read_line_result {
                println!("CommandBuilder: execute: read_line: error: {}", error);
                return Err(error);
            }

            println!("line: {line}");

            if line.starts_with(DATA_PREFIX) {
                // the line contains data to be processed by the command
                line = line.replacen(DATA_PREFIX, "", 1);
                self.append_client_data_response(line);
            }
            else if !is_status_message_received {
                self.set_client_status_response(line);
                is_status_message_received = true;
            }
            else {
                self.set_client_conclusion_response(line);
                is_finished_reading = true;
            }
        }
        
        Ok(self.build())
    }
    fn get_client_message(&self) -> String;
    fn append_client_data_response(&self, data: String);
    fn set_client_status_response(&self, status: String);
    fn set_client_conclusion_response(&self, conclusion: String);
    fn build(&self) -> TOutput;
}

struct GetTextCommand {
    row: u8,
    column: u8,
    length: u8,
    client_data: Rc<RefCell<Option<String>>>
}

impl GetTextCommand {
    fn new(row: u8, column: u8, length: u8) -> Self {
        GetTextCommand {
            row,
            column,
            length,
            client_data: Rc::new(RefCell::new(None))
        }
    }
}

impl CommandBuilder<String> for GetTextCommand {
    fn get_client_message(&self) -> String {
        format!("Ascii({},{},{})", self.row, self.column, self.length)
    }
    fn append_client_data_response(&self, data: String) {
        let client_data: &mut Option<String> = &mut self.client_data.borrow_mut();
        if client_data.is_some() {
            panic!("Unexpected additional client data response with \"{}\" while already having \"{}\".", data, client_data.as_ref().unwrap());
        }
        *client_data = Some(data);
    }
    fn set_client_status_response(&self, status: String) {
        // NOP
    }
    fn set_client_conclusion_response(&self, conclusion: String) {
        // NOP
    }
    fn build(&self) -> String {
        let client_data: &Option<String> = &self.client_data.borrow();
        client_data.as_ref().expect("The client data should have been received from the client.").clone()
    }
}

struct GetTextRangeCommand {
    row: u8,
    column: u8,
    width: u8,
    height: u8,
    lines: Rc<RefCell<Option<Vec<String>>>>
}

impl GetTextRangeCommand {
    fn new(row: u8, column: u8, width: u8, height: u8) -> Self {
        GetTextRangeCommand {
            row,
            column,
            width,
            height,
            lines: Rc::new(RefCell::new(None))
        }
    }
}

impl CommandBuilder<Vec<String>> for GetTextRangeCommand {
    fn get_client_message(&self) -> String {
        format!("Ascii({},{},{},{})", self.row, self.column, self.width - 1, self.height - 1)
    }
    fn append_client_data_response(&self, data: String) {
        if self.lines.borrow().is_none() {
            *self.lines.borrow_mut() = Some(Vec::<String>::new());
        }
        let mut borrowed_lines = self.lines.borrow_mut();
        let lines: &mut Vec<String> = &mut borrowed_lines.as_mut().unwrap();
        lines.push(data);
    }
    fn set_client_status_response(&self, status: String) {
        // NOP
    }
    fn set_client_conclusion_response(&self, conclusion: String) {
        // NOP
    }
    fn build(&self) -> Vec<String> {
        let lines: &Option<Vec<String>> = &self.lines.borrow();
        lines.as_ref().expect("The lines should have been received from the client.").clone()
    }
}

struct MoveCursorCommand {
    row: u8,
    column: u8
}

impl MoveCursorCommand {
    fn new(row: u8, column: u8) -> Self {
        MoveCursorCommand {
            row,
            column
        }
    }
}

impl CommandBuilder<()> for MoveCursorCommand {
    fn get_client_message(&self) -> String {
        format!("MoveCursor({},{})", self.row, self.column)
    }
    fn append_client_data_response(&self, data: String) {
        // NOP
    }
    fn set_client_status_response(&self, status: String) {
        // NOP
    }
    fn set_client_conclusion_response(&self, conclusion: String) {
        // NOP
    }
    fn build(&self) -> () {
        // NOP
    }
}

struct SetTextCommand {
    text: String
}

impl SetTextCommand {
    fn new<T: Into<String>>(text: T) -> Self {
        SetTextCommand {
            text: text.into()
        }
    }
}

impl CommandBuilder<()> for SetTextCommand {
    fn get_client_message(&self) -> String {
        format!("String(\"{}\")", self.text)
    }
    fn append_client_data_response(&self, data: String) {
        // NOP
    }
    fn set_client_status_response(&self, status: String) {
        // NOP
    }
    fn set_client_conclusion_response(&self, conclusion: String) {
        // NOP
    }
    fn build(&self) -> () {
        // NOP
    }
}

enum Keys {
    Enter,
    Disconnect,
    DeleteField,
}

struct SendKeysCommand {
    keys: Keys
}

impl SendKeysCommand {
    fn new(keys: Keys) -> Self {
        SendKeysCommand {
            keys
        }
    }
}

impl CommandBuilder<()> for SendKeysCommand {
    fn get_client_message(&self) -> String {
        String::from(match self.keys {
            Keys::Enter => "Enter",
            Keys::Disconnect => "Disconnect",
            Keys::DeleteField => "DeleteField",
        })
    }
    fn append_client_data_response(&self, data: String) {
        // NOP
    }
    fn set_client_status_response(&self, status: String) {
        // NOP
    }
    fn set_client_conclusion_response(&self, conclusion: String) {
        // NOP
    }
    fn build(&self) -> () {
        // NOP
    }
}

enum WaitFor {
    InputField,
    Unlock,
    NvtMode,
    Disconnect,
}

struct WaitCommand {
    wait_for: WaitFor,
    is_successful: Rc<RefCell<Option<bool>>>
}

impl WaitCommand {
    fn new(wait_for: WaitFor) -> Self {
        WaitCommand {
            wait_for,
            is_successful: Rc::new(RefCell::new(None))
        }
    }
}

impl CommandBuilder<bool> for WaitCommand {
    fn get_client_message(&self) -> String {
        let what = match &self.wait_for {
            WaitFor::InputField => "InputField",
            WaitFor::Unlock => "Unlock",
            WaitFor::NvtMode => "NVTMode",
            WaitFor::Disconnect => "Disconnect",
        };
        format!("Wait({})", what)
    }
    fn append_client_data_response(&self, data: String) {
        // NOP
    }
    fn set_client_status_response(&self, status: String) {
        // NOP
    }
    fn set_client_conclusion_response(&self, conclusion: String) {
        *self.is_successful.borrow_mut() = Some(conclusion.as_str() == "ok");
    }
    fn build(&self) -> bool {
        let is_successful: &Option<bool> = &self.is_successful.borrow();
        *is_successful.as_ref().expect("The client response should have contained if it was successful or not in waiting.")
    }
}

struct Client {
    process: Child
}

impl Client {
    fn new(process: Child) -> Self {
        Client {
            process
        }
    }
    pub fn kill(&mut self) -> Result<(), std::io::Error> {
        self.process.kill()
    }
}


struct ClientAddress {
    mainframe_address: String,
    client_address: String
}

struct ClientInterface {
    stream: TcpStream,
    client_address: String
}

impl ClientAddress {
    pub fn new<T: Into<String>>(mainframe_address: T, client_port: u16) -> Self {
        ClientAddress {
            mainframe_address: mainframe_address.into(),
            client_address: format!("localhost:{}", client_port)
        }
    }
}

impl ClientAddress {
    pub fn try_start_client_process(&self) -> Option<Client> {
        let process_result = std::process::Command::new("x3270")
            .arg("-scriptport")
            .arg(&self.client_address)
            .arg("-model")
            .arg("3279-4")
            .arg(&self.mainframe_address)
            .spawn();
        match process_result {
            Ok(process) => {
                Some(Client { process })
            },
            Err(error) => {
                println!("try_start_client_process: error starting client process with mainframe address {} and client address {} via error: {}", self.mainframe_address, self.client_address, error);
                None
            }
        }
    }
    pub fn try_connect_to_client_process(&self) -> Option<ClientInterface> {
        let stream_result = TcpStream::connect(&self.client_address);
        match stream_result {
            Ok(stream) => {
                Some(ClientInterface {
                    stream,
                    client_address: self.client_address.clone()
                })
            },
            Err(error) => {
                println!("try_connect_to_client_process: error connecting to {} via error: {}", self.mainframe_address, error);
                None
            }
        }
    }
}

impl ClientInterface {
    fn execute<TOutput>(&mut self, command: impl CommandBuilder<TOutput>) -> Result<TOutput, std::io::Error> {
        // TODO return?
        return command.execute(&mut self.stream);
    }
    pub fn get_text(&mut self, row: u8, column: u8, length: u8) -> Result<String, std::io::Error> {
        let command = GetTextCommand::new(row, column, length);
        self.execute(command)
    }
    pub fn get_text_range(&mut self, row: u8, column: u8, width: u8, height: u8) -> Result<Vec<String>, std::io::Error> {
        let command = GetTextRangeCommand::new(row, column, width, height);
        self.execute(command)
    }
    pub fn set_text<T: Into<String>>(&mut self, text: T) -> Result<(), std::io::Error> {
        let command = SetTextCommand::new(text);
        self.execute(command)
    }
    pub fn move_cursor(&mut self, row: u8, column: u8) -> Result<(), std::io::Error> {
        let command = MoveCursorCommand::new(row, column);
        self.execute(command)
    }
    pub fn send_keys(&mut self, keys: Keys) -> Result<(), std::io::Error> {
        let command = SendKeysCommand::new(keys);
        self.execute(command)
    }
    pub fn disconnect(&mut self) {
        let shutdown_result = self.stream.shutdown(std::net::Shutdown::Both);
        if let Err(shutdown_error) = shutdown_result {
            println!("Failed to disconnect via shutdown: {}", shutdown_error);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn init() {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    #[test]
    fn start_client_then_wait_then_kill() {
        init();

        let client_address = ClientAddress::new("localhost:3270", 3271);
        
        // spawn client
        let client = client_address.try_start_client_process();
        assert!(client.is_some());
        let mut client = client.unwrap();
        
        // wait a second
        std::thread::sleep(Duration::from_secs(1));

        // kill client
        let kill_result = client.kill();
        assert!(kill_result.is_ok());
    }

    #[test]
    fn start_client_then_read_screen_then_kill() {
        init();

        let client_address = ClientAddress::new("localhost:3270", 3271);
        
        // spawn client
        let client = client_address.try_start_client_process();
        assert!(client.is_some());
        let mut client = client.unwrap();
        
        // wait a second
        println!("waiting for client to be ready...");
        std::thread::sleep(Duration::from_secs(1));

        // create interface
        let interface = client_address.try_connect_to_client_process();

        if interface.is_none() {
            // kill client
            let kill_result = client.kill();
            assert!(kill_result.is_ok());
        }

        assert!(interface.is_some());
        let mut interface = interface.unwrap();
        
        let execute_result = interface.get_text_range(0, 0, 24, 80);

        // wait a second
        println!("waiting after getting text...");
        std::thread::sleep(Duration::from_secs(1));

        if execute_result.is_err() {
            // kill client
            let kill_result = client.kill();
            assert!(kill_result.is_ok());

            let error = execute_result.err().unwrap();
            println!("error: {}", error);
            panic!("Error getting text from screen: {}", error);
        }

        assert!(execute_result.is_ok());
        // TODO verify some of the text
        for line in execute_result.unwrap().iter() {
            println!("{}", line);
        }

        // kill client
        let kill_result = client.kill();
        assert!(kill_result.is_ok());
    }
}