use crate::data::Snapshot;

pub enum Filter {
    Passthrough,
}

/// Represents a view of the snapshots after filters have been applied.
pub struct View {
    data: Vec<Snapshot>,
    /// Number of items visible after filtering
    pub nb_items: usize,
    /// Height of the view window
    pub height: usize,
    /// Position of the cursor relative to the view window
    pub cursor: usize,
    /// Current window over the data view
    pub window: (usize, usize),
}

impl View {
    pub fn new(data: Vec<Snapshot>) -> View {
        let height = 5;
        let n = data.len();
        View {
            data,
            window: (0, std::cmp::min(height, n) - 1),
            height,
            cursor: 0,
            nb_items: n,
        }
    }

    /// Returns the total number of items (not the count of visible items).
    pub fn get_total_item_count(&self) -> usize {
        self.data.len()
    }

    /// Returns a view of the data.
    pub fn get_view(&self) -> Vec<&Snapshot> {
        let mut view = Vec::new();
        for snap in self.data.iter() {
            view.push(snap)
        }

        view
    }

    /// Moves the cursor up.
    pub fn up(&mut self) {
        let (min, _) = self.window;
        if self.cursor > 0 {
            self.cursor -= 1;
        } else if min > 0 {
            let n = self.data.len();
            let min = min - 1;
            let max = std::cmp::min(min + self.height, n) - 1;
            self.window = (min, max);
        }
    }

    /// Moves the cursor down.
    pub fn down(&mut self) {
        let n = self.data.len();
        let (_, max) = self.window;
        if self.cursor < self.height - 1 {
            self.cursor += 1;
        } else if max < n - 1 {
            let max = max + 1;
            let min = std::cmp::max(0, max - self.height + 1);
            self.window = (min, max);
        }
    }
}
