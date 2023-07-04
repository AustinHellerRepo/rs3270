#![allow(dead_code)]

use std::cell::RefCell;
use crate::client_interface::*;

trait ImmutableMainframeProvider {
    fn get_screen_text(&self) -> Vec<String>;
    fn get_text_at_location(&self, x: u8, y: u8, length: u8) -> String;
    fn get_fields_count(&self) -> u8;
    fn get_field_vector(&self) -> Option<(u8, u8, u8)>;
}

trait MutableMainframeProvider: ImmutableMainframeProvider {
    fn set_text_at_location(&self, x: u8, y: u8, text: &str) -> ();
    fn move_to_field_index(&self, index: u8) -> ();
}

struct MainframeProvider {
    client_interface: RefCell<ClientInterface>
}

impl MainframeProvider {
    pub fn new(client_interface: ClientInterface) -> Self {
        MainframeProvider {
            client_interface: RefCell::new(client_interface)
        }
    }
}

impl ImmutableMainframeProvider for MainframeProvider {
    fn get_screen_text(&self) -> Vec<String> {
        let lines = self.client_interface
            .borrow_mut()
            .execute(GetTextRangeCommand::new(0, 0, 80, 24))
            .expect("The lines should be returned from the client interface");
        lines
            .into_iter()
            .map(|mut line| {
                line.pop();
                line
            })
            .collect()
    }
    fn get_text_at_location(&self, x: u8, y: u8, length: u8) -> String {
        let line = self.client_interface
            .borrow_mut()
            .execute(GetTextCommand::new(y, x, length))
            .expect("The line should have been returned from the client interface.");
        line
    }
    fn get_fields_count(&self) -> u8 {
        // get the current cursor position so that it can be restored at the end
        let current_cursor_position = self.client_interface
            .borrow_mut()
            .execute(GetCursorCommand::new())
            .expect("The client interface should have returned the cursor position.");

        // move to the first field
        self.client_interface
            .borrow_mut()
            .execute(MoveCursorToFirstFieldCommand::new())
            .expect("The client interface should be able to find the first field.");

        let mut fields_count = 0;

        // TODO determine what should be done if there are no fields on the screen

        // get the first field cursor position so that we can determine when we've cycled back
        let first_field_cursor_position = self.client_interface
            .borrow_mut()
            .execute(GetCursorCommand::new())
            .expect("The client interface should be able to get the current cursor position.");

        let mut current_field_cursor_position: Option<(u8, u8)> = None;
        while current_field_cursor_position.is_none() || current_field_cursor_position.unwrap() != first_field_cursor_position {

            fields_count += 1;

            // move to the next field
            self.client_interface
                .borrow_mut()
                .execute(MoveCursorToNextFieldCommand::new())
                .expect("The client interface should be able to move the cursor to the next field.");

            // get the current field cursor position
            current_field_cursor_position = Some(self.client_interface
                .borrow_mut()
                .execute(GetCursorCommand::new())
                .expect("The client interface should be able to get the current cursor position."));
        }

        // move the cursor back to the original position
        self.client_interface
            .borrow_mut()
            .execute(MoveCursorCommand::new(current_cursor_position.0, current_cursor_position.1))
            .expect("The client nterface should be able to set the cursor position back to the original position.");

        fields_count
    }
    fn get_field_vector(&self) -> Option<(u8, u8, u8)> {
        // get the current cursor position
        let original_cursor_position = self.client_interface
            .borrow_mut()
            .execute(GetCursorCommand::new())
            .expect("The client interface should have returned the cursor position.");

        // move the cursor to the front of the field by going forward and backward
        self.client_interface
            .borrow_mut()
            .execute(MoveCursorToNextFieldCommand::new())
            .expect("The client interface should be permitted to move to the next field.");
        self.client_interface
            .borrow_mut()
            .execute(MoveCursorToPreviousFieldCommand::new())
            .expect("The client interface should be permitted to move back to the original field.");

        // get the cursor position as the beginning of the field
        let starting_cursor_position = self.client_interface
            .borrow_mut()
            .execute(GetCursorCommand::new())
            .expect("The client interface should return the starting position of the field.");

        if original_cursor_position.0 != starting_cursor_position.1 {
            // the original cursor position and the field are not on the same row

            // restore the cursor position
            self.client_interface
                .borrow_mut()
                .execute(MoveCursorCommand::new(original_cursor_position.0, original_cursor_position.1))
                .expect("The client interface should permit restoring the cursor to the original position.");

            return None;
        }
        // move the cursor to the end of the field
        self.client_interface
            .borrow_mut()
            .execute(MoveCursorToFieldEndCommand::new())
            .expect("The client interface should be able to move to the ending field position.");

        // get the cursor position at the end of the field
        let ending_cursor_position = self.client_interface
            .borrow_mut()
            .execute(GetCursorCommand::new())
            .expect("The client interface should return the ending position of the field.");

        // restore the cursor position
        self.client_interface
            .borrow_mut()
            .execute(MoveCursorCommand::new(original_cursor_position.0, original_cursor_position.1))
            .expect("The client interface should permit restoring the cursor to the original position.");

        // if the original cursor position is contained within the bounds, return the vector
        if starting_cursor_position.1 <= original_cursor_position.1 && original_cursor_position.1 <= ending_cursor_position.1 {
            return Some((starting_cursor_position.0, starting_cursor_position.1, (ending_cursor_position.1 - starting_cursor_position.1 + 1)));
        }

        // return None otherwise
        None
    }
}

impl MutableMainframeProvider for MainframeProvider {
    fn set_text_at_location(&self, x: u8, y: u8, text: &str) -> () {
        // get the current cursor position so that it can be restored at the end
        let current_cursor_position = self.client_interface
            .borrow_mut()
            .execute(GetCursorCommand::new())
            .expect("The client interface should have returned the cursor position.");
        
        // move the cursor to the appropriate location
        self.client_interface
            .borrow_mut()
            .execute(MoveCursorCommand::new(y, x))
            .expect("The client interface should have moved the cursor to where the text needs to go.");

        // set the text to the screen
        self.client_interface
            .borrow_mut()
            .execute(SetTextCommand::new(String::from(text)))
            .expect("The client interface should have set the text.");

        // restore the cursor to its original location
        self.client_interface
            .borrow_mut()
            .execute(MoveCursorCommand::new(current_cursor_position.0, current_cursor_position.1))
            .expect("The client interface should move the cursor back to where it started.");
    }
    fn move_to_field_index(&self, index: u8) -> () {
        // move to the 0th field
        self.client_interface
            .borrow_mut()
            .execute(MoveCursorToFirstFieldCommand::new())
            .expect("The client interface should move the cursor back to the first field initially.");

        if index > 0 {
            // iterate as needed
            for _ in 0..index {
                self.client_interface
                    .borrow_mut()
                    .execute(MoveCursorToNextFieldCommand::new())
                    .expect("The client interface should permit moving the cursor to the next field.");
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use std::{sync::Mutex, time::Duration};

    use crate::client_interface;

    use super::*;

    static cached_client: Mutex<Option<Client>> = Mutex::new(None);
    static is_previous_still_running: Mutex<bool> = Mutex::new(false);

    fn init() {
        std::env::set_var("RUST_BACKTRACE", "1");
        assert!(!*is_previous_still_running.lock().unwrap());
        let _ = std::mem::replace(&mut *is_previous_still_running.lock().unwrap(), true);
    }

    fn cleanup() {
        let _ = std::mem::replace(&mut *is_previous_still_running.lock().unwrap(), false);
        cached_client
            .lock()
            .expect("The mutex should provide the client Option.")
            .as_mut()
            .map(|client| {
                client
                    .kill()
                    .expect("The client should be killable.");
            });
    }

    fn get_provider() -> MainframeProvider {
        let client_address = ClientAddress::new("localhost:3270", 3271);
        let temp_client = client_address.try_start_client_process().expect("The client process should be startable.");

        // wait a second
        std::thread::sleep(Duration::from_secs(1));

        let _ = std::mem::replace(&mut *cached_client.lock().unwrap(), Some(temp_client));
        let client_interface = client_address.try_connect_to_client_process().expect("The client interface should be able to connect to the client process.");
        MainframeProvider::new(client_interface)
    }

    #[test]
    fn initialize_mainframe_provider() {
        init();

        let _ = get_provider();

        cleanup();
    }

    #[test]
    fn get_screen_text() {
        init();

        let provider = get_provider();

        let screen_text = provider.get_screen_text();
        assert_eq!(24, screen_text.len());

        cleanup();
    }
}