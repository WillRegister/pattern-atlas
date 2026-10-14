use std::collections::VecDeque;

/// A bounded, workspace-wide timeline. Save operations do not clear it.
pub struct History<T> {
    current: T,
    undo: VecDeque<T>,
    redo: Vec<T>,
    limit: usize,
    last_group: Option<String>,
    last_edit: f64,
}
impl<T: Clone + PartialEq> History<T> {
    pub fn new(initial: T, limit: usize) -> Self {
        Self {
            current: initial,
            undo: VecDeque::new(),
            redo: vec![],
            limit: limit.max(1),
            last_group: None,
            last_edit: 0.,
        }
    }
    pub fn record(&mut self, next: T, group: Option<String>, time: f64) {
        if next == self.current {
            self.current = next;
            return;
        }
        let merge = group.is_some()
            && group == self.last_group
            && time >= self.last_edit
            && time - self.last_edit < 0.75;
        if !merge {
            self.undo.push_back(self.current.clone());
            if self.undo.len() > self.limit {
                self.undo.pop_front();
            }
        }
        self.current = next;
        self.redo.clear();
        self.last_group = group;
        self.last_edit = time;
    }
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
    pub fn undo(&mut self) -> Option<T> {
        let previous = self.undo.pop_back()?;
        self.redo
            .push(std::mem::replace(&mut self.current, previous));
        self.last_group = None;
        Some(self.current.clone())
    }
    pub fn redo(&mut self) -> Option<T> {
        let next = self.redo.pop()?;
        self.undo
            .push_back(std::mem::replace(&mut self.current, next));
        self.last_group = None;
        Some(self.current.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::History;
    #[test]
    fn typing_groups_but_deletion_is_a_separate_step() {
        let mut h = History::new("alias".to_owned(), 100);
        h.record("alias A".into(), Some("alias-field".into()), 1.);
        h.record("alias AB".into(), Some("alias-field".into()), 1.2);
        h.record("deleted".into(), None, 1.3);
        assert_eq!(h.undo().as_deref(), Some("alias AB"));
        assert_eq!(h.undo().as_deref(), Some("alias"));
        assert_eq!(h.redo().as_deref(), Some("alias AB"));
        assert_eq!(h.redo().as_deref(), Some("deleted"));
    }
    #[test]
    fn a_new_edit_discards_redo_and_history_is_bounded() {
        let mut h = History::new(0, 2);
        for i in 1..=3 {
            h.record(i, None, i as f64);
        }
        assert_eq!(h.undo(), Some(2));
        assert_eq!(h.undo(), Some(1));
        assert_eq!(h.undo(), None);
        h.record(9, None, 10.);
        assert!(!h.can_redo());
        assert_eq!(h.undo(), Some(1));
    }
    #[test]
    fn pauses_and_different_fields_split_typing() {
        let mut h = History::new(0, 100);
        h.record(1, Some("name".into()), 1.);
        h.record(2, Some("name".into()), 2.);
        h.record(3, Some("context".into()), 2.1);
        assert_eq!(h.undo(), Some(2));
        assert_eq!(h.undo(), Some(1));
        assert_eq!(h.undo(), Some(0));
    }
}
