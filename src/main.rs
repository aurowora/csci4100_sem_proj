use crate::avl::Leaderboard;

mod avl;


pub fn main() {
    let mut leader = Leaderboard::new();

    // from the prof's example
    leader.insert("A", 150);
    leader.insert("B", 200);
    leader.insert("C", 120);
    leader.insert("D", 180);
    leader.insert("E", 250);
    
    eprintln!("Leaderboard (Pre-order traversal):");
    for (player, score) in leader.pre_order() {
        eprintln!("Score {}: {}", score, player.join(", "))
    }
    
    // top 3
    eprintln!("\nTop 3 Players:");
    for (player, score) in leader.top_n_players(3) {
        eprintln!("#{} Player {} - Score: {}", leader.rank_of(&player, score).unwrap(), player, score);
    }
    
    // find the rank of player B
    eprintln!("\nRank of Player B: {}", leader.rank_of("B", 200).unwrap());
}