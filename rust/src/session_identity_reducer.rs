//! One project/module transition policy for live previews and cold history.
//!
//! Identity is what the session was ASSIGNED, never what its log mentions. The
//! only evidence this reducer accepts is the session's own `[project]` prefix,
//! and a prefix is the assistant's own sentence — not independent proof of what
//! the owner asked for. So:
//! - one prefix alone is a MENTION and never seeds a name;
//! - the same project twice within the window is a declaration and seeds;
//! - one prefix that AGREES with the scope the CLI opened the session in (`cwd`)
//!   is corroborated by an independent fact and seeds at once.

use std::collections::VecDeque;

pub(super) struct IdentityReducer {
    pub current: Option<(String, String)>,
    recent: VecDeque<Option<String>>,
    lookback: usize,
    /// Project of the directory the CLI opened the session in, in the same form
    /// `declared_parts` returns. `None` = opened at the workspace root.
    scope: Option<String>,
}

impl IdentityReducer {
    pub fn new(current: Option<(String, String)>, lookback: usize, scope: Option<String>) -> Self {
        assert!(lookback > 0);
        Self {
            current,
            recent: VecDeque::new(),
            lookback,
            scope: scope.filter(|s| !s.trim().is_empty()),
        }
    }

    /// A plain assistant text counts in the existing observation window too.
    /// `false` means an isolated project mention was refused, not lost input.
    pub fn push(&mut self, declaration: Option<(String, String)>) -> bool {
        self.recent
            .push_back(declaration.as_ref().map(|(project, _)| project.clone()));
        if self.recent.len() > self.lookback {
            self.recent.pop_front();
        }
        let Some((project, module)) = declaration else {
            return true;
        };
        let confirmations = self
            .recent
            .iter()
            .filter(|p| p.as_deref() == Some(project.as_str()))
            .count();
        match &mut self.current {
            Some((old, lane)) if *old == project => {
                if !module.is_empty() {
                    *lane = module;
                }
            }
            Some(_) if confirmations < 2 => return false,
            // Nothing seeded yet: a lone prefix is a mention unless the scope
            // the session was opened in says the same thing.
            None if confirmations < 2 && self.scope.as_deref() != Some(project.as_str()) => {
                return false;
            }
            _ => self.current = Some((project, module)),
        }
        true
    }
}
