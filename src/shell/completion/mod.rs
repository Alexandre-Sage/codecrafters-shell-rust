use std::sync::Arc;

use crate::shell::completion;

pub mod builtins;
pub mod path_dirs;

pub(super) trait Completion {
    fn completion_items(&self, args: &str) -> Vec<String>;

    fn single_completion(&self, matches: Vec<String>, args: &str) -> Option<String> {
        if matches.len() == 1 {
            let completion_item = matches[0][args.len()..].to_string();
            return Some(format!("{completion_item} "));
        }

        let mut prefix = String::new();

        for (idx, char) in matches[0].chars().enumerate() {
            let all_match = matches
                .iter()
                .all(|item| item.chars().nth(idx) == Some(char));

            if !all_match {
                break;
            }

            prefix.push(char);
        }

        if prefix.len() <= args.len() {
            return None;
        }

        Some(prefix[args.len()..].to_string())
    }

    fn multiple_completion(&self, matches: Vec<String>) -> Option<String> {
        Some(matches.join("  "))
    }

    fn complete(&self, args: &str, multiple: bool) -> Option<String> {
        if args.is_empty() {
            return None;
        }

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

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCompletion {
        items: Vec<String>,
    }

    impl Completion for MockCompletion {
        fn completion_items(&self, args: &str) -> Vec<String> {
            self.items
                .iter()
                .filter(|item| item.starts_with(args))
                .cloned()
                .collect()
        }
    }

    #[test]
    fn test_no_matches_returns_none() {
        let comp = MockCompletion {
            items: vec!["echo".to_string(), "exit".to_string()],
        };
        assert_eq!(comp.complete("xyz", false), None);
    }

    #[test]
    fn test_single_match_completes_with_space() {
        let comp = MockCompletion {
            items: vec!["echo_test".to_string()],
        };
        // "ech" should complete to "o_test "
        assert_eq!(comp.complete("ech", false), Some("o_test ".to_string()));
    }

    #[test]
    fn test_partial_completion_common_prefix() {
        let comp = MockCompletion {
            items: vec!["echo_test".to_string(), "echo_debug".to_string()],
        };
        // "ech" should complete to "o_" (common prefix)
        assert_eq!(comp.complete("ech", false), Some("o_".to_string()));
    }

    #[test]
    fn test_no_common_prefix_beyond_input() {
        let comp = MockCompletion {
            items: vec!["echo".to_string(), "exit".to_string()],
        };
        // "e" has no common prefix beyond itself
        assert_eq!(comp.complete("e", false), None);
    }

    #[test]
    fn test_multiple_completion_shows_all_sorted() {
        let comp = MockCompletion {
            items: vec!["exit".to_string(), "echo".to_string(), "env".to_string()],
        };
        // Should return all matches sorted
        assert_eq!(
            comp.complete("e", true),
            Some("echo  env  exit".to_string())
        );
    }

    #[test]
    fn test_already_fully_typed_returns_space() {
        let comp = MockCompletion {
            items: vec!["echo".to_string()],
        };
        // Already fully typed "echo" - nothing to add
        assert_eq!(comp.complete("echo", false), Some(" ".to_string()));
    }

    #[test]
    fn test_partial_with_three_matches() {
        let comp = MockCompletion {
            items: vec![
                "application".to_string(),
                "apple".to_string(),
                "apply".to_string(),
            ],
        };
        assert_eq!(comp.complete("app", false), Some("l".to_string()));
        assert_eq!(comp.complete("appl", false), None);
        assert_eq!(comp.complete("appli", false), Some("cation ".to_string()));
    }

    #[test]
    fn test_empty_input_with_matches() {
        let comp = MockCompletion {
            items: vec!["echo".to_string(), "exit".to_string()],
        };
        // Empty input, multiple matches, no common prefix
        assert_eq!(comp.complete("", false), None);
    }

    #[test]
    fn test_single_char_partial_completion() {
        let comp = MockCompletion {
            items: vec!["test_one".to_string(), "test_two".to_string()],
        };
        // "t" should complete to "est_" (common prefix)
        assert_eq!(comp.complete("t", false), Some("est_".to_string()));
    }

    #[test]
    fn test_multiple_completion_preserves_sort_order() {
        let comp = MockCompletion {
            items: vec!["abcd".to_string(), "abbd".to_string(), "abdc".to_string()],
        };
        // Should be alphabetically sorted
        assert_eq!(
            comp.complete("a", true),
            Some("abbd  abcd  abdc".to_string())
        );
    }
}
