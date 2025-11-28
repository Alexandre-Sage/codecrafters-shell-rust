use std::sync::Arc;

pub mod builtins;
pub mod path_dirs;

pub(super) trait Completion {
    fn completion_items(&self, args: &str) -> Vec<String>;

    fn single_completion(&self, matches: Vec<String>, args: &str) -> Option<String> {
        // if matches.len() > 1 {
        //     return None;
        // }

        let completion_item = matches[0][args.len()..].to_string();

        Some(completion_item)
    }

    fn multiple_completion(&self, matches: Vec<String>) -> Option<String> {
        Some(matches.join("  "))
    }

    fn complete(&self, args: &str, multiple: bool) -> Option<String> {
        let mut matches = self.completion_items(args);

        if matches.is_empty() {
            return None;
        }

        matches.sort();
        if multiple {
            return self.multiple_completion(matches);
        }

        return self.single_completion(matches, args);
    }
}

pub trait CompletionComponent {
    fn execute(&self, input: &str, multiple: bool) -> Option<String> {
        match self.handler(input, multiple) {
            Some(res) => Some(res),
            None => {
                if let Some(next) = self.next() {
                    return next.execute(input, multiple);
                }
                None
            }
        }
    }

    fn handler(&self, args: &str, multiple: bool) -> Option<String>;

    fn next(&self) -> Option<Arc<dyn CompletionComponent>>;
}
