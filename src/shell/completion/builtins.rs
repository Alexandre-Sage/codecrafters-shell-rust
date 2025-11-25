use std::sync::Arc;

use crate::{
    commands::CommandToken,
    shell::{
        completion::{
            path_dirs::{self, PathDirsCompletion},
            CompletionComponent,
        },
        path::PathDirs,
    },
};

pub struct BuiltinsCompletion {
    builtins: Vec<String>,
    next: Arc<dyn CompletionComponent>,
}

impl BuiltinsCompletion {
    pub fn new(path_dirs: Arc<PathDirs>) -> Self {
        let builtins = CommandToken::into_completion();
        let next = Arc::new(PathDirsCompletion::new(path_dirs));
        Self { builtins, next }
    }

    pub fn complete(&self, args: &str) -> Option<String> {
        let matches: Vec<_> = self
            .builtins
            .iter()
            .filter(|builtin| builtin.starts_with(args))
            .collect();

        if matches.is_empty() {
            return None;
        }

        if matches.len() > 1 {
            return None;
        }

        let matched = matches[0];
        let completion_item = matched[args.len()..].to_string();

        Some(completion_item)
    }
}

impl CompletionComponent for BuiltinsCompletion {
    fn next(&self) -> Option<Arc<dyn CompletionComponent>> {
        Some(Arc::clone(&self.next))
    }

    fn handler(&self, args: &str) -> Option<String> {
        self.complete(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> BuiltinsCompletion {
        BuiltinsCompletion::new(Arc::new(PathDirs::from_env()))
    }

    #[test]
    fn complete_partial_command_with_single_match() {
        let completion = setup();

        assert_eq!(completion.complete("ec"), Some("ho".to_string()));
        assert_eq!(completion.complete("typ"), Some("e".to_string()));
        assert_eq!(completion.complete("pw"), Some("d".to_string()));
        assert_eq!(completion.complete("exi"), Some("t".to_string()));
    }

    #[test]
    fn complete_ambiguous_prefix_returns_none() {
        let completion = setup();

        // "e" matches both "echo" and "exit"
        assert_eq!(completion.complete("e"), None);
    }

    #[test]
    fn complete_full_command_returns_empty_string() {
        let completion = setup();

        assert_eq!(completion.complete("echo"), Some("".to_string()));
        assert_eq!(completion.complete("exit"), Some("".to_string()));
        assert_eq!(completion.complete("pwd"), Some("".to_string()));
    }

    #[test]
    fn complete_no_match_returns_none() {
        let completion = setup();

        assert_eq!(completion.complete("xyz"), None);
        assert_eq!(completion.complete("ls"), None);
        assert_eq!(completion.complete("unknown"), None);
    }

    #[test]
    fn complete_empty_input_returns_none() {
        let completion = setup();

        // Empty string matches all commands - ambiguous
        assert_eq!(completion.complete(""), None);
    }
}
