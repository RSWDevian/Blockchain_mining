mod miner{
    pub mod chain;
    mod mining;
}
mod wallet{
    pub mod transaction;
    pub mod tx;
    pub mod wallet;
}
mod command_line{
    pub mod cli;
}
//? use of CLI in out module
use std::io;
use command_line::cli::Cli;

fn main()->Result<(),io::Error>{
    let mut cli = Cli::new()?;
    cli.run()?;
    Ok(())
}
