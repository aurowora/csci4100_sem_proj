/* Public API */
use std::cmp::max;
use std::fmt::{Display, Formatter};
use std::mem;

pub struct Leaderboard {
    root: Option<AVLNode>
}

impl Leaderboard {
    /// Create a new AVL tree
    pub fn new() -> Self {
        Leaderboard { root: None }
    }

    /// Insert a new score into the AVL tree.
    pub fn insert(&mut self, player_id: impl AsRef<str>, score: u64) {
        if let Some(ref mut inner) = self.root {
            inner.insert(player_id, score);
        } else {
            self.root = Some(AVLNode {
                player_id: vec![player_id.as_ref().to_owned()],
                score,
                right: None,
                left: None,
                height: 1,
                children: 0
            });
        }
    }


    /// Delete all score of a specific player.
    pub fn delete_player(&mut self, player_id: impl AsRef<str>) {
        if let Some(ref mut inner) = self.root {
            if inner.delete_player(player_id) {
                self.root = None;
            }
        }
    }

    /// Delete a specific score belonging to a player
    pub fn delete_player_score(&mut self, player_id: impl AsRef<str>, score: u64) {
        if let Some(ref mut inner) = self.root {
            if inner.delete_player_score(player_id, score) {
                self.root = None;
            }
        }
    }

    pub fn top_n_players(&self, n: usize) -> Vec<(String, u64)> {
        if let Some(ref inner) = self.root {
            inner.top_n_players(n)
        } else {
            Vec::new()
        }
    }

    pub fn rank_of(&self, player: impl AsRef<str>, score: u64) -> Option<usize> {
        if let Some(ref inner) = self.root {
            inner.rank_of(player, score, 0)
        } else {
            None
        }
    }
    
    pub fn pre_order(&self) -> LeaderboardIter {
        LeaderboardIter {
            nodes: match &self.root {
                Some(r) => vec![r],
                None => Vec::new()
            }
        }
    }
}

impl Display for Leaderboard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match &self.root {
            Some(r) => r.to_string(),
            None => "None".to_owned()
        })
    }
}

/// An iterator over the leaderboard entries. This does pre-order traversal.
pub struct LeaderboardIter<'a> {
    nodes: Vec<&'a AVLNode>
}

impl<'a> Iterator for LeaderboardIter<'a> {
    type Item = (&'a Vec<String>, u64);
    
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(n) = self.nodes.pop() {
            if let Some(ref right) = n.right {
                self.nodes.push(right);
            }
            if let Some(ref left) = n.left {
                self.nodes.push(left);
            }

            Some((&n.player_id, n.score))
        } else {
            None
        }
    }
}

/* Private API */

struct AVLNode {
    // It is possible for multiple players to achieve the exact same score.
    // For simplicity, we will store these in the same node.
    player_id: Vec<String>,
    score: u64,
    right: Option<Box<AVLNode>>,
    left: Option<Box<AVLNode>>,
    height: isize,
    children: usize
}

impl AVLNode {
    fn new(player: impl AsRef<str>, score: u64) -> Self {
        Self {
            player_id: vec![player.as_ref().to_owned()],
            score,
            right: None,
            left: None,
            height: 1,
            children: 0,
        }
    }

    #[inline]
    fn height_left(&self) -> isize {
        self.left.as_ref().map(|l| l.height).unwrap_or(0)
    }

    #[inline]
    fn height_right(&self) -> isize {
        self.right.as_ref().map(|r| r.height).unwrap_or(0)
    }

    #[inline]
    fn update_attrs(&mut self) {
        // tree height
        self.height = max(self.height_left(), self.height_right()) + 1;

        // number of children
        // +1 to each because node is present
        let left_children = self.left.as_ref().map(|l| l.children + 1).unwrap_or(0);
        let right_children = self.right.as_ref().map(|l| l.children + 1).unwrap_or(0);
        self.children = left_children + right_children;
    }

    #[inline]
    fn unbalanced(&self) -> bool {
        isize::abs_diff(self.height_left(), self.height_right()) > 1
    }

    fn insert(&mut self, player: impl AsRef<str>, score: u64) {
        // Insert the node, recursively.
        if self.score == score {
            self.player_id.push(player.as_ref().to_owned());
            // nothing else changes.
            return;
        } else if score < self.score {
            if let Some(ref mut left_node) = self.left {
                left_node.insert(player, score)
            } else {
                self.left = Some(Box::new(Self::new(player, score)))
            }
            self.update_attrs()
        } else { // score > self.score
            if let Some(ref mut right_node) = self.right {
                right_node.insert(player, score)
            } else {
                self.right = Some(Box::new(Self::new(player, score)))
            }
            self.update_attrs()
        }

        self.rebalance_if_needed()
    }

    #[inline]
    fn rebalance_if_needed(&mut self) {
        if !self.unbalanced() { return; }

        // right heavy
        if self.height_right() - self.height_left() > 1 {
            let rr_heavy = match self.right {
                Some(ref r) => r.height_right() > r.height_left(),
                None => unreachable!()
            };

            if rr_heavy {
                self.rotate_left()
            } else { // rl
                self.right.as_mut().unwrap().rotate_right();
                self.rotate_left()
            }
        } else { // left heavy
            let ll_heavy = match self.left {
                Some(ref r) => r.height_left() > r.height_right(),
                None => unreachable!()
            };

            if ll_heavy {
                self.rotate_right()
            } else { // lr
                self.left.as_mut().unwrap().rotate_left();
                self.rotate_right()
            }
        }

    }

    /*
        a
         \
          b
           \
            c

     To fix this, b becomes the new root with a as the left child and c as the right child
     */
    #[inline]
    fn rotate_left(&mut self) {
        // take a's existing right child
        let mut b = self.right.take().unwrap();
        // take b's original left subtree and give it to A (current node)
        self.right = b.left.take();
        // swap ourselves and b in memory
        // (&mut self refers to b now! b refers to a now!)
        mem::swap(self, b.as_mut());
        b.update_attrs();
        self.left = Some(b);
        self.update_attrs();
    }

    /*
              c
             /                  b
           b           ->     /   \
          /                  a     c
        a

    B becomes the new root, c takes ownership of b's right child as its left child,
    B takes ownership of c as its right child
    */
    #[inline]
    fn rotate_right(&mut self) {
        // get b
        let mut b = self.left.take().unwrap();
        // take b's right child, give it to C
        self.left = b.right.take();
        // note: b refers to c in memory now, self refers to b.
        mem::swap(self, b.as_mut());
        b.update_attrs();
        self.right = Some(b);
        self.update_attrs();
    }

    // return TRUE to request deletion from the caller
    #[must_use = "If this function returns true, it is the caller's responsibility to delete the node delete() was called on"]
    pub fn delete_player(&mut self, player_id: impl AsRef<str>) -> bool {
        let ply_id = player_id.as_ref();

        // we need to do a post order traversal here because it is nice for
        // our left and right subtrees to be "settled" before we do anything
        // to the current node.

        if let Some(ref mut l) = self.left {
            if l.delete_player(ply_id) {
                self.left = None;
            }
        }
        if let Some(ref mut r) = self.right {
            if r.delete_player(ply_id) {
                self.right = None;
            }
        }

        // delete the player identified by the caller if present
        self.player_id.retain(|player_id| player_id != ply_id);

        // and delete this node if it doesn't contain any player id
        if self.player_id.len() < 1 {
            return self.delete();
        }

        // maybe dirty
        self.update_attrs();
        self.rebalance_if_needed();

        false
    }

    // return TRUE to request deletion from the caller
    #[must_use = "If this function returns true, it is the caller's responsibility to delete the node delete() was called on"]
    pub fn delete_player_score(&mut self, player_id: impl AsRef<str>, score: u64) -> bool {
        if self.score == score {
            // Delete the player identified by the caller
            let ply_id = player_id.as_ref();
            self.player_id.retain(|player_id| player_id != ply_id);

            if self.player_id.len() < 1 {
                return self.delete();
            }
        } else if score < self.score {
            if let Some(ref mut l) = self.left {
                if l.delete_player_score(player_id, score) {
                    self.left = None;
                }
            }
        } else { // right
            if let Some(ref mut r) = self.right {
                if r.delete_player_score(player_id, score) {
                    self.right = None;
                }
            }
        }

        self.update_attrs();
        self.rebalance_if_needed();

        false
    }

    // return TRUE to request deletion from the caller
    #[must_use = "If this function returns true, it is the caller's responsibility to delete the node delete() was called on"]
    pub fn delete(&mut self) -> bool {
        if self.left.is_none() && self.right.is_none() {
            // case 1: just delete me
            return true;
        } else if self.left.is_some() ^ self.right.is_some() {
            // case 2: swap this node and the child
            // and allow this node to be dropped.
            let mut only_child = if self.left.is_some() {
                self.left.take().unwrap()
            } else { // right
                self.right.take().unwrap()
            };

            mem::swap(self, &mut only_child);
        } else {
            // case 3: we have two children. We need to replace the current node
            // with our predecessor. We'll take the left child, swap it with this node
            // and continue deletion recursively until we've pushed it to a leaf

            // calling mem::swap(self, &mut l) doesn't work here because
            // we don't want to mess with the pointers to the children at this point
            let l = self.left.as_mut().unwrap();
            mem::swap(&mut self.score, &mut l.score);
            mem::swap(&mut self.player_id, &mut l.player_id);

            if l.delete() {
                self.left = None;
            }
        }

        self.update_attrs();
        self.rebalance_if_needed();

        false
    }

    pub fn top_n_players(&self, n: usize) -> Vec<(String, u64)> {
        let mut results = Vec::with_capacity(n);

        if let Some(ref right) = self.right {
            results.extend(right.top_n_players(n));
        }

        // anything left for us?
        let mut rem = n - results.len();
        if rem < 1 {
            return results; // no :(
        }

        // get up to `rem` scores from the current node
        results.extend(self.player_id.iter().enumerate()
            .take_while(|(i, v)| *i < rem)
            .map(|(_, v)| (v.clone(), self.score)));
        
        // if we still need more elements, try the left child
        rem = n - results.len();
        if let Some(ref left) = self.left {
            results.extend(left.top_n_players(rem));
        }

        results
    }

    pub fn rank_of(&self, player: impl AsRef<str>, score: u64, num_better: usize) -> Option<usize> {
        // nodes can potentially store more than one player inside, but because
        // we assign the same rank number to ties, we can ignore this fact in
        // this method.
        
        let ply_id = player.as_ref();
        let right_tree_size = self.right.as_ref().map(|r| r.children + 1 /* include this node too */).unwrap_or(0);
        
        if self.score == score {
            if self.player_id.iter().filter(|v| **v == ply_id).count() < 1  {
                // ???
                return None;
            }
            
            Some(1 + right_tree_size + num_better)
        } else if score < self.score {
            if let Some(ref left) = self.left {
                left.rank_of(ply_id, score, 1 + right_tree_size + num_better )
            } else {
                None
            }
        } else {
            if let Some(ref right) = self.right {
                right.rank_of(ply_id, score, num_better)
            } else {
                None
            }
        }
    }
    
    fn format_string(&self, mut buf: &mut String, level: usize) {
        // player list at node
        let mut players = self.player_id.iter()
            .fold(String::new(), |mut acc, ply| {
                if acc.len() > 0 {
                    acc.push(',');
                    acc.push(' ');
                }
                acc.push_str(ply);
                acc
            });
        
        // score, height, children
        players.push('(');
        players.push_str(&self.score.to_string());
        players.push_str(", ");
        players.push_str(&self.height.to_string());
        players.push_str(", ");
        players.push_str(&self.children.to_string());
        players.push(')');
        
        
        let padding = "\t".repeat(level);
        
        buf.push_str(&format!("{padding}{players}\n{padding}right:\n"));
        match &self.right {
            Some(rn) => rn.format_string(&mut buf, level + 1),
            None => buf.push_str(&format!("{padding}\t(no right node)\n"))
        }
        buf.push_str(&format!("{padding}left:\n"));
        match &self.left {
            Some(ln) => ln.format_string(&mut buf, level + 1),
            None => buf.push_str(&format!("{padding}\t(no left node)\n"))
        }
        buf.push('\n');
    }

}

impl Display for AVLNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::new();
        self.format_string(&mut buf, 0);
        write!(f, "{}", buf)
    }
}