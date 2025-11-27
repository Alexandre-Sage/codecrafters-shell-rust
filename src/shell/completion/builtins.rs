use std::{collections::HashSet, sync::Arc};

use crate::{
    commands::CommandToken,
    shell::{
        completion::{
            self,
            path_dirs::{self, PathDirsCompletion},
            Completion, CompletionComponent,
        },
        path::PathDirsProvider,
    },
};

pub struct BuiltinsCompletion {
    builtins: Vec<String>,
    next: Arc<dyn CompletionComponent>,
}

impl BuiltinsCompletion {
    pub fn new(path_dirs: Arc<PathDirsProvider>) -> Self {
        let builtins = CommandToken::into_completion();
        let next = Arc::new(PathDirsCompletion::new(path_dirs));
        Self { builtins, next }
    }
}

impl Completion for BuiltinsCompletion {
    fn completion_items(&self, args: &str) -> Vec<String> {
        self.builtins
            .iter()
            .cloned()
            .filter(|builtin| builtin.starts_with(args))
            .collect()
    }
}

impl CompletionComponent for BuiltinsCompletion {
    fn next(&self) -> Option<Arc<dyn CompletionComponent>> {
        Some(Arc::clone(&self.next))
    }

    fn handler(&self, args: &str, multiple: bool) -> Option<String> {
        self.complete(args, multiple)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> BuiltinsCompletion {
        BuiltinsCompletion::new(Arc::new(PathDirsProvider::from_env()))
    }

    #[test]
    fn complete_partial_command_with_single_match() {
        let completion = setup();

        assert_eq!(completion.complete("ec", false), Some("ho".to_string()));
        assert_eq!(completion.complete("typ", false), Some("e".to_string()));
        assert_eq!(completion.complete("pw", false), Some("d".to_string()));
        assert_eq!(completion.complete("exi", false), Some("t".to_string()));
    }

    #[test]
    fn complete_ambiguous_prefix_returns_none() {
        let completion = setup();

        // "e" matches both "echo" and "exit"
        assert_eq!(completion.complete("e", false), None);
    }

    #[test]
    fn complete_full_command_returns_empty_string() {
        let completion = setup();

        assert_eq!(completion.complete("echo", false), Some("".to_string()));
        assert_eq!(completion.complete("exit", false), Some("".to_string()));
        assert_eq!(completion.complete("pwd", false), Some("".to_string()));
    }

    #[test]
    fn complete_no_match_returns_none() {
        let completion = setup();

        assert_eq!(completion.complete("xyz", false), None);
        assert_eq!(completion.complete("ls", false), None);
        assert_eq!(completion.complete("unknown", false), None);
    }

    #[test]
    fn complete_empty_input_returns_none() {
        let completion = setup();

        // Empty string matches all commands - ambiguous
        assert_eq!(completion.complete("", false), None);
    }
}
