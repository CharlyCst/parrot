use std::rc::Rc;

use crate::data::Snapshot;
use super::parser::Filter;

/// Represents a view of the snapshots after filters have been applied.
pub struct View {
    data: Vec<Rc<Snapshot>>,
    view: Vec<Rc<Snapshot>>,
    /// Height of the view window
    pub height: usize,
    /// Position of the cursor relative to the view window
    pub cursor: usize,
    /// Current window over the data view
    pub window: (usize, usize),
}

impl View {
    pub fn new(data: Vec<Rc<Snapshot>>) -> View {
        let height = 5;
        let n = data.len();
        let mut view = Vec::with_capacity(data.len());
        for snap in &data {
            view.push(Rc::clone(snap));
        }
        View {
            data,
            view,
            window: (0, std::cmp::min(height, n)),
            height,
            cursor: 0,
        }
    }

    /// Returns the total number of items (not the count of visible items).
    pub fn get_total_item_count(&self) -> usize {
        self.data.len()
    }

    /// Returns a view of the data.
    pub fn get_view(&self) -> &Vec<Rc<Snapshot>> {
        &self.view
    }

    /// Returns the selected item.
    pub fn get_selected(&self) -> Option<&Snapshot> {
        if self.view.len() == 0 {
            None
        } else {
            Some(&self.view[self.window.0 + self.cursor])
        }
    }

    /// Moves the cursor up.
    pub fn up(&mut self) {
        let (min, _) = self.window;
        if self.cursor > 0 {
            self.cursor -= 1;
        } else if min > 0 {
            let n = self.view.len();
            let min = min - 1;
            let max = std::cmp::min(min + self.height, n);
            self.window = (min, max);
        }
    }

    /// Moves the cursor down.
    pub fn down(&mut self) {
        let n = self.view.len();
        let (_, max) = self.window;
        if self.cursor + 1 < self.height && self.cursor + 1 < self.view.len() {
            self.cursor += 1;
        } else if max + 1 < n {
            let max = max + 1;
            let min = std::cmp::max(0, max as i64 - self.height as i64) as usize;
            self.window = (min, max);
        }
    }

    /// Applies a filter
    pub fn apply_filter(&mut self, filter: Filter) {
        match filter {
            Filter::Tag(ref tag) => self.apply_tag_filter(tag),
            Filter::Name(ref name) => self.apply_name_filter(name),
        }
        self.update_window();
    }

    /// Remove any filter currently applied.
    pub fn clear_filters(&mut self) {
        let mut view = Vec::with_capacity(self.data.len());
        for snap in &self.data {
            view.push(Rc::clone(snap));
        }
        self.view = view;
        self.update_window();
    }

    /// Update the position of the window to create a sane state.
    fn update_window(&mut self) {
        let (min, max) = self.window;
        let n = self.view.len();
        if max > n {
            let max = n;
            let min = if max < self.height {
                0
            } else {
                max - self.height
            };
            self.window = (min, max);
            // If there are less items than the position of the cursor
            if self.cursor > max {
                self.cursor = if max == 0 { 0 } else { max - 1 };
            }
        } else if max - min < self.height && max < n {
            let max = std::cmp::min(min + self.height, n);
            self.window = (min, max);
        }
    }

    /// Applies a tag filter.
    fn apply_tag_filter(&mut self, tag: &String) {
        let old_view = std::mem::replace(&mut self.view, Vec::new());
        for snap in old_view {
            if snap.tags.contains(tag) {
                self.view.push(snap);
            }
        }
    }

    /// Applies a name filter.
    fn apply_name_filter(&mut self, name: &String) {
        let old_view = std::mem::replace(&mut self.view, Vec::new());
        for snap in old_view {
            if snap.name.contains(name) {
                self.view.push(snap);
            }
        }
    }
}
