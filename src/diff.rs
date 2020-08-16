use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum Node {
    N((usize, usize)),
    Root,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DiffLine<'a> {
    Keep(&'a [u8]),
    Delete(&'a [u8]),
    Insert(&'a [u8]),
}

// Heavily inspired by https://github.com/tamuhey/seqdiff/blob/master/src/lib.rs.
/// Returns the shortest edit script (or diff) between two slices of bytes slices.
pub fn get_diff<'a>(old: &[&'a [u8]], new: &[&'a [u8]]) -> Vec<DiffLine<'a>> {
    let n = old.len();
    let m = new.len();
    let bound = n + m;
    let get_y = |x, k| x + bound - k;
    let mut v = vec![0; 2 * bound + 1];
    let mut nodes_map = HashMap::new();

    // Forward pass
    'outer: for d in 0..=bound {
        for k in ((bound - d)..=bound + d).step_by(2) {
            let (mut x, parent) = if d == 0 {
                // Initial state
                (0, Node::Root)
            } else if k == (bound - d) || k != (bound + d) && v[k - 1] < v[k + 1] {
                // Move downward
                let px = v[k + 1];
                (px, Node::N((px, get_y(px, k + 1))))
            } else {
                // Move rightward
                let px = v[k - 1];
                (px + 1, Node::N((px, get_y(px, k - 1))))
            };
            let mut y = get_y(x, k);
            nodes_map.insert(Node::N((x, y)), parent);

            // Take as much diagonals as possible
            while x < n && y < m && old[x] == new[y] {
                nodes_map.insert(Node::N((x + 1, y + 1)), Node::N((x, y)));
                x += 1;
                y += 1;
            }

            v[k] = x;
            if x >= n && y >= m {
                // Done
                break 'outer;
            }
        }
    }

    // Backtrack
    let mut current = Node::N((n, m));
    let mut diff = Vec::new();
    loop {
        let previous = *nodes_map
            .get(&current)
            .expect("Internal error: failed to backtrack diff.");
        if previous == Node::Root {
            break;
        }
        let (prev_x, prev_y) = get_coordinates(previous);
        let (x, y) = get_coordinates(current);
        if x == prev_x && y == prev_y + 1 {
            diff.push(DiffLine::Insert(new[prev_y]));
        } else if x == prev_x + 1 && y == prev_y {
            diff.push(DiffLine::Delete(old[prev_x]));
        } else if x == prev_x + 1 && y == prev_y + 1 {
            diff.push(DiffLine::Keep(old[prev_x]));
        } else {
            panic!(
                "Internal error: malformed path in diff backtrack: from ({}, {}) to ({}, {}).",
                prev_x, prev_y, x, y
            )
        }
        current = previous;
    }
    diff.reverse();
    diff
}

/// Extracts the x and y coordinate of a node.
fn get_coordinates(node: Node) -> (usize, usize) {
    match node {
        Node::Root => (0, 0),
        Node::N((x, y)) => (x, y),
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_diff() {
        // Examples taken from https://blog.jcoglan.com/2017/02/12/the-myers-diff-algorithm-part-1/
        let old = vec![
            &[b'a'][..],
            &[b'b'],
            &[b'c'],
            &[b'a'],
            &[b'b'],
            &[b'b'],
            &[b'a'],
        ];
        let new = vec![&[b'c'][..], &[b'b'], &[b'a'], &[b'b'], &[b'a'], &[b'c']];
        let diff = get_diff(&old[..], &new[..]);
        let expected_diff = vec![
            DiffLine::Delete(old[0]),
            DiffLine::Delete(old[1]),
            DiffLine::Keep(old[2]),
            DiffLine::Insert(new[1]),
            DiffLine::Keep(old[3]),
            DiffLine::Keep(old[4]),
            DiffLine::Delete(old[5]),
            DiffLine::Keep(old[6]),
            DiffLine::Insert(new[5]),
        ];
        assert_eq!(diff, expected_diff);
    }
}
