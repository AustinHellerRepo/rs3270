
trait Screen<T: MutableMainframeProvider> {
    fn is_active(&self, provider: T) -> bool;
    fn try_navigate_to(&self, provider: &T) -> bool;
}

struct NavigateOperation<'a, T: MutableMainframeProvider> {
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

enum Operation<'a, T: MutableMainframeProvider> {
    Navigate(NavigateOperation<'a, T>),
    Store(StoreOperation),
    Set(SetOperation),
}

trait OperationCondition {
    fn is_true(&self) -> bool;
}

struct SingleOperationTreeNode<'a, T: MutableMainframeProvider> {
    operation: Operation<'a, T>,
    next: Option<Box<OperationTreeNode<'a, T>>>
}

impl<'a, T: MutableMainframeProvider> SingleOperationTreeNode<'a, T> {
    pub fn new(operation: Operation<'a, T>, next: Option<Box<OperationTreeNode<'a, T>>>) -> Self {
        SingleOperationTreeNode { operation, next }
    }
}

struct ConditionalOperationTreeNode<'a, T: MutableMainframeProvider> {
    condition: fn(&HashMap<String, String>, &dyn ImmutableMainframeProvider) -> bool,
    consequent: Box<OperationTreeNode<'a, T>>,
    alternative: Option<Box<OperationTreeNode<'a, T>>>
}

impl<'a, T: MutableMainframeProvider> ConditionalOperationTreeNode<'a, T> {
    pub fn new(condition: fn(&HashMap<String, String>, &dyn ImmutableMainframeProvider) -> bool, consequent: Box<OperationTreeNode<'a, T>>, alternative: Option<Box<OperationTreeNode<'a, T>>>) -> Self {
        ConditionalOperationTreeNode { condition, consequent, alternative }
    }
}

enum OperationTreeNode<'a, T: MutableMainframeProvider> {
    Single(SingleOperationTreeNode<'a, T>),
    Conditional(ConditionalOperationTreeNode<'a, T>)
}

struct OperationContext<T: MutableMainframeProvider> {
    value_per_variable_name: Rc<RefCell<HashMap<String, String>>>,
    phantom_mainframe_provider: PhantomData<T>,
}

impl<T: MutableMainframeProvider> OperationContext<T> {
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
            //Operation::KeyPress(operation) => {
            //    provider.send_key_press(&operation.key_press);
            //}
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
                    if (operation.condition)(&borrowed_value_per_variable_name, provider as &dyn ImmutableMainframeProvider) {
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


    #[test]
    fn initialize_context() {
        let operation_context = OperationContext::<MainframeProvider>::new(HashMap::from([
            (String::from("test"), String::from("something"))
        ]));
    }
}