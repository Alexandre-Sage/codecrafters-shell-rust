use std::sync::Arc;

pub mod builtins;
pub mod path_dirs;

pub trait CompletionComponent {
    fn execute(&self, input: &str) -> Option<String> {
        match self.handler(input) {
            Some(res) => Some(res),
            None => {
                if let Some(next) = self.next() {
                    return next.execute(input);
                }
                None
            }
        }
    }

    fn handler(&self, args: &str) -> Option<String>;

    fn next(&self) -> Option<Arc<dyn CompletionComponent>>;
}
