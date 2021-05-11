extern crate esa_cli;

use std::env;

use dotenv::dotenv;

use crate::esa_cli::esa::{self, Esa};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let esa = Esa::new();
    let team = esa
        .team(
            esa::TeamId::new(env::var("ESA_TEAM_ID").unwrap()),
            env::var("ESA_ACCESS_TOKEN").unwrap(),
        )
        .await;
    match team {
        Ok(team) => println!("team: {}", team.name),
        Err(reason) => println!("error: {}", reason),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
