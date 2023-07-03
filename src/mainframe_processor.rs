#![allow(dead_code)]

use std::{collections::HashMap, marker::PhantomData, cell::RefCell, rc::Rc};
use crate::client_interface::*;

enum KeyCode {
    LetterA,
    LetterB,
    LetterC,
    LetterD,
    LetterE,
    LetterF,
    LetterG,
    LetterH,
    LetterI,
    LetterJ,
    LetterK,
    LetterL,
    LetterM,
    LetterN,
    LetterO,
    LetterP,
    LetterQ,
    LetterR,
    LetterS,
    LetterT,
    LetterU,
    LetterV,
    LetterW,
    LetterX,
    LetterY,
    LetterZ,
    Number0,
    Number1,
    Number2,
    Number3,
    Number4,
    Number5,
    Number6,
    Number7,
    Number8,
    Number9,
    SymbolSpace,
    SymbolExclamationPoint,
    SymbolAt,
    SymbolNumber,
    SymbolDollar,
    SymbolPercentage,
    SymbolUpCaret,
    SymbolAmpersand,
    SymbolAsterisk,
    SymbolLeftParenthesis,
    SymbolRightParenthesis,
    SymbolHyphen,
    SymbolEqual,
    SymbolUnderscore,
    SymbolPlus,
    SymbolTilde,
    SymbolGrave,
    SymbolComma,
    SymbolPeriod,
    SymbolLessThan,
    SymbolGreaterThan,
    SymbolQuestion,
    SymbolForwardSlash,
    SymbolBackSlash,
    SymbolPipe,
    SymbolColon,
    SymbolSemicolon,
    SymbolSingleQuote,
    SymbolDoubleQuote,
    SymbolLeftSquareBracket,
    SymbolRightSquareBracket,
    SymbolLeftCurlyBrace,
    SymbolRightCurlyBrace,
    CommandEscape,
    CommandF1,
    CommandF2,
    CommandF3,
    CommandF4,
    CommandF5,
    CommandF6,
    CommandF7,
    CommandF8,
    CommandF9,
    CommandF10,
    CommandF11,
    CommandF12,
    CommandBackspace,
    CommandDelete,
    CommandTab,
    CommandUpArrow,
    CommandRightArrow,
    CommandDownArrow,
    CommandLeftArrow,
    CommandPageUp,
    CommandPageDown,
    CommandHome,
    CommandEnd,
    CommandInsert
}

struct KeyPress {
    key_code: KeyCode,
    is_ctrl_key_held: bool,
    is_shift_key_held: bool,
    is_alt_key_held: bool
}

trait ReadOnlyMainframeProvider {
    fn get_screen_text(&self) -> Vec<String>;
    fn get_text_at_location(&self, x: u8, y: u8, length: u8) -> String;
    fn get_fields_count(&self) -> u8;
}

trait MainframeProvider: ReadOnlyMainframeProvider {
    fn set_text_at_location(&self, x: u8, y: u8, text: &str) -> ();
    fn send_key_press(&self, key_press: &KeyPress) -> ();
}

trait Screen<T: MainframeProvider> {
    fn is_active(&self, provider: T) -> bool;
    fn try_navigate_to(&self, provider: &T) -> bool;
}

struct NavigateOperation<'a, T: MainframeProvider> {
    screen: &'a dyn Screen<T>
}

struct StoreOperation {
    x: u8,
    y: u8,
    length: u8,
    variable_name: String
}

enum SetOperationSource {
    RawText(String),
    StoredVariable(String)
}

struct SetOperation {
    x: u8,
    y: u8,
    source: SetOperationSource
}

struct KeyPressOperation {
    key_press: KeyPress
}

enum Operation<'a, T: MainframeProvider> {
    Navigate(NavigateOperation<'a, T>),
    Store(StoreOperation),
    Set(SetOperation),
    KeyPress(KeyPressOperation)
}

trait OperationCondition {
    fn is_true(&self) -> bool;
}

struct SingleOperationTreeNode<'a, T: MainframeProvider> {
    operation: Operation<'a, T>,
    next: Option<Box<OperationTreeNode<'a, T>>>
}

impl<'a, T: MainframeProvider> SingleOperationTreeNode<'a, T> {
    pub fn new(operation: Operation<'a, T>, next: Option<Box<OperationTreeNode<'a, T>>>) -> Self {
        SingleOperationTreeNode { operation, next }
    }
}

struct ConditionalOperationTreeNode<'a, T: MainframeProvider> {
    condition: fn(&HashMap<String, String>, &dyn ReadOnlyMainframeProvider) -> bool,
    consequent: Box<OperationTreeNode<'a, T>>,
    alternative: Option<Box<OperationTreeNode<'a, T>>>
}

impl<'a, T: MainframeProvider> ConditionalOperationTreeNode<'a, T> {
    pub fn new(condition: fn(&HashMap<String, String>, &dyn ReadOnlyMainframeProvider) -> bool, consequent: Box<OperationTreeNode<'a, T>>, alternative: Option<Box<OperationTreeNode<'a, T>>>) -> Self {
        ConditionalOperationTreeNode { condition, consequent, alternative }
    }
}

enum OperationTreeNode<'a, T: MainframeProvider> {
    Single(SingleOperationTreeNode<'a, T>),
    Conditional(ConditionalOperationTreeNode<'a, T>)
}

struct OperationContext<T: MainframeProvider> {
    value_per_variable_name: Rc<RefCell<HashMap<String, String>>>,
    phantom_mainframe_provider: PhantomData<T>,
}

impl<T: MainframeProvider> OperationContext<T> {
    pub fn new(value_per_variable_name: HashMap<String, String>) -> Self {
        OperationContext {
            value_per_variable_name: Rc::new(RefCell::new(value_per_variable_name)),
            phantom_mainframe_provider: PhantomData,
        }
    }
    pub fn process_operation<'a>(&self, provider: &T, operation: &Operation<'a, T>) {
        match operation {
            Operation::Navigate(operation) => {
                let is_successful = operation.screen.try_navigate_to(provider);
                // TODO react to failure
            },
            Operation::Store(operation) => {
                let value = provider.get_text_at_location(operation.x, operation.y, operation.length);
                self.value_per_variable_name.borrow_mut().insert(operation.variable_name.clone(), value);
            },
            Operation::Set(operation) => {
                match &operation.source {
                    SetOperationSource::RawText(text) => {
                        provider.set_text_at_location(operation.x, operation.y, text.as_str());
                    },
                    SetOperationSource::StoredVariable(variable_name) => {
                        let borrowed_value_per_variable_name = self.value_per_variable_name.borrow();
                        let value = borrowed_value_per_variable_name.get(variable_name);
                        match value {
                            Some(text) => {
                                provider.set_text_at_location(operation.x, operation.y, text)
                            },
                            None => {
                                // TODO react to None
                            }
                        }
                    }
                }
            },
            Operation::KeyPress(operation) => {
                provider.send_key_press(&operation.key_press);
            }
        }
    }
    pub fn process_operations<'a>(&'a self, provider: &'a T, operations: &OperationTreeNode<'a, T>) {
        let mut current_operation = Some(operations);
        while let Some(operation) = current_operation {
            match operation {
                OperationTreeNode::Single(operation) => {
                    self.process_operation(provider, &operation.operation);
                    current_operation = operation.next.as_ref().map(|operation| {
                        operation.as_ref()
                    });
                },
                OperationTreeNode::Conditional(operation) => {
                    let borrowed_value_per_variable_name = self.value_per_variable_name.borrow();
                    if (operation.condition)(&borrowed_value_per_variable_name, provider as &dyn ReadOnlyMainframeProvider) {
                        current_operation = Some(operation.consequent.as_ref());
                    }
                    else if let Some(operation) = &operation.alternative {
                        current_operation = Some(operation.as_ref());
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::client_interface::ClientInterface;

    use super::*;

    struct TestMainframeProvider {
        client_interface: RefCell<ClientInterface>
    }

    impl ReadOnlyMainframeProvider for TestMainframeProvider {
        fn get_screen_text(&self) -> Vec<String> {
            let get_text_range_result = self.client_interface
                .borrow_mut()
                .execute(GetTextRangeCommand::new(0, 0, 80, 24));
            let lines = get_text_range_result.expect("The lines should be returned from the client interface");
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

            // get the first field cursor position so that we can determine when we've cycled back

            // move to the next field

            // get the current field cursor position

            // check if we have cycled back to the first field

            todo!()
        }
    }

    impl MainframeProvider for TestMainframeProvider {
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

        fn send_key_press(&self, key_press: &KeyPress) -> () {
            todo!()
        }
    }

    #[test]
    fn initialize_context() {
        let operation_context = OperationContext::<TestMainframeProvider>::new(HashMap::from([
            (String::from("test"), String::from("something"))
        ]));
    }
}