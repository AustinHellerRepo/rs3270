use std::{net::TcpStream, io::{Write, BufReader}, io::BufRead, cell::{RefCell, Cell}, rc::Rc};

static DATA_PREFIX: &str = " data:";


trait CommandBuilder<TOutput> {
    fn execute(&self, stream: &mut TcpStream, reader: &mut BufReader<TcpStream>) -> Result<TOutput, std::io::Error> {
        let client_message = self.get_client_message();
        stream.write_all(client_message.as_bytes())?;
        stream.flush()?;

        let mut is_finished_reading = false;
        let mut is_status_message_received = false;
        while !is_finished_reading {
            let mut line = String::new();
            let bytes_count = reader.read_line(&mut line)?;

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
    fn append_client_data_response(&self, message: String);
    fn set_client_status_response(&self, status: String);
    fn set_client_conclusion_response(&self, conclusion: String);
    fn build(&self) -> TOutput;
}

struct AsciiRclCommand {
    row: u8,
    column: u8,
    length: u8,
    client_data: Rc<RefCell<Option<String>>>
}

impl CommandBuilder<String> for AsciiRclCommand {
    fn get_client_message(&self) -> String {
        return format!("Ascii({},{},{})", self.row, self.column, self.length);
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
        return client_data.as_ref().expect("The client data should have been received from the client.").clone();
    }
}

struct ConnectCommand {
    host_name: String,
    is_connected: Rc<RefCell<Option<bool>>>
}

impl CommandBuilder<bool> for ConnectCommand {
    fn get_client_message(&self) -> String {
        return format!("Connect({})", self.host_name);
    }
    fn append_client_data_response(&self, message: String) {
        // NOP
    }
    fn set_client_status_response(&self, status: String) {
        // NOP
    }
    fn set_client_conclusion_response(&self, conclusion: String) {
        *self.is_connected.borrow_mut() = Some(conclusion.as_str() == "ok");
    }
    fn build(&self) -> bool {
        let is_connected: &Option<bool> = &self.is_connected.borrow();
        return *is_connected.as_ref().expect("The client response should have contained if it was able to connect.");
    }
}


struct ClientInterfaceAddress {
    client_address: String,
}

struct ClientInterface {
    stream: TcpStream,
    reader: BufReader<TcpStream>
}

impl ClientInterfaceAddress {
    pub fn new(client_address: String) -> Self {
        ClientInterfaceAddress {
            client_address
        }
    }
}

impl ClientInterfaceAddress {
    fn try_connect(&mut self) -> Option<ClientInterface> {
        let stream_result = TcpStream::connect(&self.client_address);
        match stream_result {
            Ok(stream) => {
                let reader = BufReader::new(stream.try_clone().expect("The stream should have been clonable into the reader."));
                return Some(ClientInterface { stream, reader });
            },
            Err(error) => {
                println!("try_connect: error connecting to {} via error: {}", self.client_address, error);
                return None;
            }
        }
    }
}

impl ClientInterface {
    fn execute<TOutput>(&mut self, command: impl CommandBuilder<TOutput>) -> TOutput {
        // TODO return?
        return command.execute(&mut self.stream, &mut self.reader).expect("The command should execute correctly.");
    }

    fn disconnect(&mut self) {
        let shutdown_result = self.stream.shutdown(std::net::Shutdown::Both);
        if let Err(shutdown_error) = shutdown_result {
            println!("Failed to disconnect via shutdown: {}", shutdown_error);
        }
    }
}