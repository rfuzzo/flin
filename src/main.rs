use flin::Game;

fn main() {
    println!("Care for a round of Flin?");

    let mut g = Game::new();
    g.play();
    if let Some(winner) = g.winner {
        println!("The winner is: {}", winner);
    } else {
        println!("Draw!");
    }
}
