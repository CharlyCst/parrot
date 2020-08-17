use crate::data::Snapshot;

pub enum Filter {
    Passthrough,
}

/// Represents a view of the snapshots after filters have been applied.
pub struct View {
    data: Vec<Snapshot>,
    filter: Filter,
    pub cursor: usize,
    pub window: (usize, usize),
    pub height: usize,
}

impl View {
    pub fn new(data: Vec<Snapshot>) -> View {
        let height = 10;
        let n = data.len();
        View {
            data,
            filter: Filter::Passthrough,
            cursor: 0,
            window: (0, std::cmp::min(height, n)),
            height,
        }
    }

    /// Returns a view of the data.
    pub fn get_view(&self) -> Vec<&Snapshot> {
        let mut view = Vec::new();
        for snap in self.data.iter() {
            view.push(snap)
        }

        view
    }
}
