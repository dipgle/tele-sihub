//! One project/module transition policy for live previews and cold history.

use std::collections::VecDeque;

pub(super) struct IdentityReducer {
    pub current: Option<(String, String)>,
    recent: VecDeque<Option<String>>,
    lookback: usize,
}

impl IdentityReducer {
    pub fn new(current: Option<(String, String)>, lookback: usize) -> Self {
        assert!(lookback > 0);
        Self {
            current,
            recent: VecDeque::new(),
            lookback,
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
            _ => self.current = Some((project, module)),
        }
        true
    }
}
