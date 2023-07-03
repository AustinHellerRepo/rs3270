use std::{net::TcpStream, io::{Write, BufReader}, io::{Error, BufRead}};

static DATA_PREFIX: &str = " data:";


trait Command<TOutput> {
    fn execute(&self, stream: &mut TcpStream, reader: &mut BufReader<TcpStream>) -> Result<TOutput, Error> {
        let output_message = self.get_output_message();
        stream.write_all(output_message.as_bytes())?;
        stream.flush()?;

        let mut is_finished_reading = false;
        let mut is_status_message_received = false;
        while !is_finished_reading {
            let mut line = String::new();
            let bytes_count = reader.read_line(&mut line)?;

            if line.starts_with(DATA_PREFIX) {
                // the line contains data to be processed by the command
                line = line.replacen(DATA_PREFIX, "", 1);
                self.process_client_message(line);
            }
            else if !is_status_message_received {
                self.process_client_status(line);
                is_status_message_received = true;
            }
            else {
                self.process_client_conclusion(line);
                is_finished_reading = true;
            }
        }
        
        Ok(self.get_output())
    }
    fn get_output_message(&self) -> String;
    fn process_client_message(&self, message: String);
    fn process_client_status(&self, status: String);
    fn process_client_conclusion(&self, conclusion: String);
    fn get_output(&self) -> TOutput;
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
    fn execute(&mut self, command: impl Command) {
        // TODO return?
        command.execute(&mut self.stream);
    }

    fn disconnect(&mut self) {
        let shutdown_result = self.stream.shutdown(std::net::Shutdown::Both);
        if let Err(shutdown_error) = shutdown_result {
            println!("Failed to disconnect via shutdown: {}", shutdown_error);
        }
    }
}