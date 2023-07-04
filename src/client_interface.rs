#![allow(dead_code)]

use std::{net::TcpStream, io::{Write, BufReader}, io::BufRead, cell::RefCell, process::Child};

// TODO always check the status for "ok" or "error"

#[derive(Debug)]
pub enum ExecutionResult<T> {
    Unset,
    Success(T),
    IoError(std::io::Error),
    CommandFailure(Option<String>),
}

fn unwrap_failed(msg: &str, error: &dyn std::fmt::Debug) -> ! {
    panic!("{msg}: {error:?}")
}

impl<T: std::fmt::Debug> ExecutionResult<T> {
    pub fn unwrap(self) -> T {
        match self {
            ExecutionResult::Success(item) => {
                item
            },
            ExecutionResult::IoError(e) => {
                unwrap_failed("called `ExecutionResult::unwrap()` on an `IoError` value", &e)
            },
            ExecutionResult::CommandFailure(cf) => {
                unwrap_failed("called `ExecutionResult::unwrap()` on a `CommandFailure` value", &cf)
            }
            ExecutionResult::Unset => {
                unwrap_failed("called `ExecutionResult::unwrap()` on an `Unset` value", &self)
            }
        }
    }
    pub fn expect(self, message: &str) -> T {
        match self {
            ExecutionResult::Success(item) => {
                item
            },
            ExecutionResult::IoError(e) => {
                unwrap_failed(message, &e)
            },
            ExecutionResult::CommandFailure(cf) => {
                unwrap_failed(message, &cf)
            },
            ExecutionResult::Unset => {
                unwrap_failed(message, &self)
            }
        }
    }
    pub fn is_err(&self) -> bool {
        match self {
            ExecutionResult::Success(_) => {
                false
            },
            ExecutionResult::IoError(_) => {
                true
            },
            ExecutionResult::CommandFailure(_) => {
                true
            },
            ExecutionResult::Unset => {
                true
            }
        }
    }
    pub fn is_ok(&self) -> bool {
        !self.is_err()
    }
    pub fn err(self) -> Option<String> {
        match self {
            ExecutionResult::Success(_) => {
                None
            },
            ExecutionResult::IoError(e) => {
                Some(format!("{e:?}"))
            }
            ExecutionResult::CommandFailure(cf) => {
                Some(format!("{cf:?}"))
            }
            ExecutionResult::Unset => {
                Some(format!("ExecutionResult::Unset"))
            }
        }
    }
}

macro_rules! command {
    ($command_name:ty,
        command: $client_message_block:block) => {
        paste::paste! {
            pub struct [<$command_name Command>] {
                [<$command_name:camel _is_successful>]: RefCell<ExecutionResult<()>>
            }

            impl [<$command_name Command>] {
                pub fn new() -> Self {
                    [<$command_name Command>] {
                        [<$command_name:camel _is_successful>]: RefCell::new(ExecutionResult::Unset)
                    }
                }
            }

            impl CommandBuilder<()> for [<$command_name Command>] {
                fn get_client_message(&self) -> String {
                    $client_message_block
                }
                fn append_client_data_response(&self, _: String) {
                    // NOP
                }
                fn build(self) -> () {
                    // NOP
                }
            }
        }
    };
    ($command_name:ty,
        [$($arg_name:ident: $arg_type:ty),*],
        command: $client_message_block:block) => {
        paste::paste! {
            pub struct [<$command_name Command>] {
                $(
                    $arg_name: $arg_type,
                )*
            }

            impl [<$command_name Command>] {
                pub fn new($($arg_name: $arg_type),*) -> Self {
                    [<$command_name Command>] {
                        $(
                            $arg_name,
                        )*
                    }
                }
            }

            impl CommandBuilder<()> for [<$command_name Command>] {
                fn get_client_message(&self) -> String {
                    $(
                        let $arg_name: &$arg_type = &self.$arg_name;
                    )*
                    $client_message_block
                }
                fn append_client_data_response(&self, _: String) {
                    // NOP
                }
                fn build(self) -> () {
                    // NOP
                }
            }
        }
    };
    ($command_name:ty,
        [$($arg_name:ident: $arg_type:ty),*],
        command: $client_message_block:block,
        output => $return_name:ident: $return_type:ty) => {
        paste::paste! {
            pub struct [<$command_name Command>] {
                $(
                    $arg_name: $arg_type,
                )*
                $return_name: RefCell<Option<$return_type>>
            }

            impl [<$command_name Command>] {
                pub fn new($($arg_name: $arg_type),*) -> Self {
                    [<$command_name Command>] {
                        $(
                            $arg_name,
                        )*
                        $return_name: RefCell::new(None)
                    }
                }
            }

            impl CommandBuilder<$return_type> for [<$command_name Command>] {
                fn get_client_message(&self) -> String {
                    $(
                        let $arg_name: &$arg_type = &self.$arg_name;
                    )*
                    $client_message_block
                }
                fn append_client_data_response(&self, _: String) {
                    // NOP
                }
                fn build(self) -> $return_type {
                    let $return_name: Option<$return_type> = self.$return_name.into_inner();
                    $return_name.unwrap()
                }
            }
        }
    };
    ($command_name:ty,
        [$($arg_name:ident: $arg_type:ty),*],
        command: $client_message_block:block,
        output => $return_name:ident: $return_type:ty,
        data: (
            $data_name:ident,
            $data_block:block
        )) => {
        paste::paste! {
            pub struct [<$command_name Command>] {
                $(
                    $arg_name: $arg_type,
                )*
                $return_name: RefCell<Option<$return_type>>
            }

            impl [<$command_name Command>] {
                pub fn new($($arg_name: $arg_type),*) -> Self {
                    [<$command_name Command>] {
                        $(
                            $arg_name,
                        )*
                        $return_name: RefCell::new(None)
                    }
                }
            }

            impl CommandBuilder<$return_type> for [<$command_name Command>] {
                fn get_client_message(&self) -> String {
                    $(
                        let $arg_name: &$arg_type = &self.$arg_name;
                    )*
                    $client_message_block
                }
                fn append_client_data_response(&self, $data_name: String) {
                    let $return_name: &mut Option<$return_type> = &mut self.$return_name.borrow_mut();
                    $data_block
                }
                fn build(self) -> $return_type {
                    let $return_name: Option<$return_type> = self.$return_name.into_inner();
                    $return_name.unwrap()
                }
            }
        }
    };
    ($command_name:ty,
        [$($arg_name:ident: $arg_type:ty),*],
        command: $client_message_block:block,
        output => $return_name:ident: $return_type:ty,
        data: (
            $data_name:ident,
            $data_block:block
        ),
        status: (
            $status_name:ident,
            $status_block:block
        )) => {
        paste::paste! {
            pub struct [<$command_name Command>] {
                $(
                    $arg_name: $arg_type,
                )*
                $return_name: RefCell<Option<$return_type>>
            }

            impl [<$command_name Command>] {
                pub fn new($($arg_name: $arg_type),*) -> Self {
                    [<$command_name Command>] {
                        $(
                            $arg_name,
                        )*
                        $return_name: RefCell::new(None)
                    }
                }
            }

            impl CommandBuilder<$return_type> for [<$command_name Command>] {
                fn get_client_message(&self) -> String {
                    $(
                        $arg_name: &$arg_type = &self.$arg_name;
                    )*
                    $client_message_block
                }
                fn append_client_data_response(&self, $data_name: String) {
                    let $return_name: &mut Option<$return_type> = &mut self.$return_name.borrow_mut();
                    $data_block
                }
                fn set_client_status_response(&self, $status_name: String) {
                    let $return_name: &mut Option<$return_type> = &mut self.$return_name.borrow_mut();
                    $status_block
                }
                fn set_client_conclusion_response(&self, _: String) {
                    // NOP
                }
                fn build(self) -> $return_type {
                    let $return_name: Option<$return_type> = self.$return_name.into_inner();
                    $return_name.unwrap()
                }
            }
        }
    };
    ($command_name:ty,
        command: $client_message_block:block,
        output => $return_name:ident: $return_type:ty,
        data: (
            $data_name:ident,
            $data_block:block
        )) => {
        paste::paste! {
            pub struct [<$command_name Command>] {
                $return_name: RefCell<Option<$return_type>>
            }

            impl [<$command_name Command>] {
                pub fn new() -> Self {
                    [<$command_name Command>] {
                        $return_name: RefCell::new(None)
                    }
                }
            }

            impl CommandBuilder<$return_type> for [<$command_name Command>] {
                fn get_client_message(&self) -> String {
                    $client_message_block
                }
                fn append_client_data_response(&self, $data_name: String) {
                    let $return_name: &mut Option<$return_type> = &mut self.$return_name.borrow_mut();
                    $data_block
                }
                fn build(self) -> $return_type {
                    let $return_name: Option<$return_type> = self.$return_name.into_inner();
                    $return_name.unwrap()
                }
            }
        }
    };
    ($command_name:ty,
        command: $client_message_block:block,
        output => $return_name:ident: $return_type:ty) => {
        paste::paste! {
            pub struct [<$command_name Command>] {
                $return_name: RefCell<Option<$return_type>>
            }

            impl [<$command_name Command>] {
                pub fn new() -> Self {
                    [<$command_name Command>] {
                        $return_name: RefCell::new(None)
                    }
                }
            }

            impl CommandBuilder<$return_type> for [<$command_name Command>] {
                fn get_client_message(&self) -> String {
                    $client_message_block
                }
                fn append_client_data_response(&self, _: String) {
                    // NOP
                }
                fn build(self) -> $return_type {
                    let $return_name: Option<$return_type> = self.$return_name.into_inner();
                    $return_name.unwrap()
                }
            }
        }
    };
    ($command_name:ty,
        [$($arg_name:ident: $arg_type:ty),*],
        command: $client_message_block:block,
        output => $return_name:ident: $return_type:ty,
        data: (
            $data_name:ident,
            $data_block:block
        )) => {
        paste::paste! {
            pub struct [<$command_name Command>] {
                $(
                    $arg_name: $arg_type,
                )*
                $return_name: RefCell<Option<$return_type>>
            }

            impl [<$command_name Command>] {
                pub fn new($($arg_name: $arg_type),*) -> Self {
                    [<$command_name Command>] {
                        $(
                            $arg_name,
                        )*
                        $return_name: RefCell::new(None)
                    }
                }
            }

            impl CommandBuilder<$return_type> for [<$command_name Command>] {
                fn get_client_message(&self) -> String {
                    $(
                        $arg_name: &$arg_type = &self.$arg_name;
                    )*
                    $client_message_block
                }
                fn append_client_data_response(&self, $data_name: String) {
                    let $return_name: &mut Option<$return_type> = &mut self.$return_name.borrow_mut();
                    $data_block
                }
                fn build(self) -> $return_type {
                    let $return_name: Option<$return_type> = self.$return_name.into_inner();
                    $return_name.unwrap()
                }
            }
        }
    };
}

pub trait CommandBuilder<TOutput> {
    fn execute(self, stream: &mut TcpStream) -> ExecutionResult<TOutput> where Self:Sized {

        // get the client message and prepare to send it
        let client_message = self.get_client_message();
        let client_message = format!("{}\n", client_message);

        // send the client message to the running/connected program
        println!("CommandBuilder: execute: sending client message: \"{}\"", &client_message);
        let write_all_result = stream.write_all(client_message.as_bytes());
        if let Err(error) = write_all_result {
            println!("CommandBuilder: execute: write_all: error: {}", error);
            return ExecutionResult::IoError(error);
        }
        let flush_result = stream.flush();
        if let Err(error) = flush_result {
            println!("CommandBuilder: execute: flush: error: {}", error);
            return ExecutionResult::IoError(error);
        }

        // begin reading the response from the running/connected program
        let mut reader = BufReader::new(stream.try_clone().unwrap());

        let mut is_finished_reading = false;
        let mut is_status_message_received = false;
        let mut iteration_count = 0;
        let mut is_conclusion_successful = false;
        let mut first_line: Option<String> = None;
        while !is_finished_reading {
            println!("read iteration: {}", iteration_count);

            let mut line = String::new();
            let read_line_result = reader.read_line(&mut line);
            if let Err(error) = read_line_result {
                println!("CommandBuilder: execute: read_line: error: {}", error);
                return ExecutionResult::IoError(error);
            }

            println!("line: {line}");

            if line.starts_with("data: ") {
                // the line contains data to be processed by the command
                line = line
                    .replacen("data: ", "", 1)
                    .replace("\n", "");
                if iteration_count == 0 {
                    first_line = Some(line.clone());
                }
                self.append_client_data_response(line);
            }
            else if !is_status_message_received {
                // NOP
                is_status_message_received = true;
            }
            else {
                if line.as_str().trim() == "ok" {
                    is_conclusion_successful = true;
                }
                else {
                    println!("client_interface: CommandBuilder: execute: error: \"{}\"", line);
                }
                is_finished_reading = true;
            }

            iteration_count += 1;
        }

        if is_conclusion_successful {
            ExecutionResult::Success(self.build())
        }
        else {
            ExecutionResult::CommandFailure(first_line)
        }
    }
    fn get_client_message(&self) -> String;
    fn append_client_data_response(&self, data: String);
    fn build(self) -> TOutput;
}

command!(GetText, [
        row: u8,
        column: u8,
        length: u8
    ],
    command: {
        format!("Ascii({},{},{})", row, column, length)
    },
    output => text: String,
    data: (
        data, {
            if text.is_some() {
                panic!("Unexpected additional text from the client.");
            }
            *text = Some(data);
        }
    )
);

command!(GetTextRange, [
        row: u8,
        column: u8,
        width: u8,
        height: u8
    ],
    command: {
        format!("Ascii({},{},{},{})", row, column, height, width)
    },
    output => lines: Vec<String>,
    data: (
        data, {
            if lines.is_none() {
                *lines = Some(Vec::<String>::new());
            }
            let lines: &mut Vec<String> = &mut lines.as_mut().unwrap();
            lines.push(data);
        }
    )
);

command!(MoveCursor, [
        row: u8,
        column: u8
    ],
    command: {
        format!("MoveCursor({},{})", row, column)
    }
);

command!(SetText, [
        text: String
    ],
    command: {
        format!("String(\"{}\")", text)
    }
);

command!(MoveCursorToNextField,
    command: {
        format!("Tab")
    }
);

command!(MoveCursorToPreviousField,
    command: {
        format!("BackTab")
    }
);

command!(MoveCursorToFirstField,
    command: {
        String::from("Home")
    }
);

command!(SendEnterKey,
    command: {
        format!("Enter")
    }
);

command!(ClearTextFromField,
    command: {
        format!("DeleteField")
    }
);

command!(MoveCursorToFieldEnd,
    command: {
        format!("FieldEnd")
    }
);

command!(WaitForCurrentField,
    command: {
        format!("Wait(InputField)")
    },
    output => is_successful: bool
);

command!(WaitForUnlock,
    command: {
        format!("Wait(Unlock)")
    },
    output => is_successful: bool
);

command!(GetCursor,
    command: {
        format!("Query(Cursor)")
    },
    output => position: (u8, u8),
    data: (
        data, {
            let position_vector = data
                .split(" ")
                .map(|item| {
                    println!("GetCursorCommand: parsing \"{}\"", &item); 
                    item.parse::<u8>().expect("The coordinate should be parsable as a u8")
                })
                .collect::<Vec<u8>>();
            let position_tuple = (position_vector[0], position_vector[1]);
            
            if position.is_some() {
                panic!("Unexpected additional client data response with \"{}\" while already having \"{:?}\".", data, position.as_ref().unwrap());
            }
            *position = Some(position_tuple);
        }
    )
);

pub struct Client {
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


pub struct ClientAddress {
    mainframe_address: String,
    client_address: String
}

pub struct ClientInterface {
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

pub trait CommandExecutor {
    fn get_stream(&mut self) -> &mut TcpStream;
    fn execute<TOutput>(&mut self, command: impl CommandBuilder<TOutput>) -> ExecutionResult<TOutput> {
        let mut stream = self.get_stream();
        command.execute(&mut stream)
    }
    fn disconnect(&mut self) {
        let stream = self.get_stream();
        let shutdown_result = stream.shutdown(std::net::Shutdown::Both);
        if let Err(shutdown_error) = shutdown_result {
            println!("Failed to disconnect via shutdown: {}", shutdown_error);
        }
    }
}

impl CommandExecutor for ClientInterface {
    fn get_stream(&mut self) -> &mut TcpStream {
        &mut self.stream
    }
}


#[cfg(test)]
mod tests {
    use std::{time::Duration, sync::Mutex};

    use super::*;

    static is_previous_still_running: Mutex<bool> = Mutex::new(false);

    fn init() {
        std::env::set_var("RUST_BACKTRACE", "1");
        assert!(!*is_previous_still_running.lock().unwrap());
        let _ = std::mem::replace(&mut *is_previous_still_running.lock().unwrap(), true);
    }

    fn cleanup() {
        let _ = std::mem::replace(&mut *is_previous_still_running.lock().unwrap(), false);
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

        cleanup();
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
        
        let execute_result = interface.execute(GetTextRangeCommand::new(0, 0, 80, 24));

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

        cleanup();
    }

    #[test]
    fn start_client_then_next_field_then_previous_field_then_kill() {
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
        
        // move forward
        let execute_result = interface.execute(MoveCursorToNextFieldCommand::new());

        // wait a second
        println!("waiting after moving to next field...");
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

        // move backward
        let execute_result = interface.execute(MoveCursorToPreviousFieldCommand::new());

        // wait a second
        println!("waiting after moving to previous field...");
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

        // kill client
        let kill_result = client.kill();
        assert!(kill_result.is_ok());

        cleanup();
    }

    #[test]
    fn start_client_then_end_of_field_then_kill() {
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
        
        // move forward
        let execute_result = interface.execute(MoveCursorToFieldEndCommand::new());

        // wait a second
        println!("waiting after moving to end of field...");
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

        // kill client
        let kill_result = client.kill();
        assert!(kill_result.is_ok());

        cleanup();
    }

    #[test]
    fn start_client_then_get_cursor_position_then_kill() {
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
        
        let execute_result = interface.execute(GetCursorCommand::new());

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

        // kill client
        let kill_result = client.kill();
        assert!(kill_result.is_ok());

        cleanup();
    }
}