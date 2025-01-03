use std::{io, process::exit};
use bitcoincash_addr::Address;
use clap::{arg, Command};
use crate::{miner::chain::Blockchain, wallet::{transaction::Transaction, wallet::Wallets}};
#[derive(Debug)]
pub struct Cli{

}

impl Cli {
    pub fn new() -> Result<Cli,io::Error>{
        Ok(Cli{})
    }

    pub fn run(&mut self) -> Result<(),io::Error>{
        let matches = Command::new("bchain")
            //? primary details about our blockchain CLI
            .version("1.0") // version of our chain
            .author("Jester") // author of this project
            .about("A blockchain in rust") //about our project
            //?Sub-commands list for the blockchain cli

            //* All get operations of the command line */
            .subcommand(Command::new("print-chain").about("Pritns he blockchain"))
            .subcommand(Command::new("list-addresses")
                .about("Get a list of all your wallet addresses")
            )
            .subcommand(Command::new("get-balance")
                .about("Get your balance in the blockchain")
                .arg(arg!(<ADDRESS>"'The targeted adress'"))
            )

            //* All creation operations of the command line */
            .subcommand(Command::new("create")
                .about("Create a new Blockchain")
                .arg(arg!(<ADDRESS>"'The adress of genesis block'"))
            )
            .subcommand(Command::new("create-wallet")
                .about("Creates a wallet")
            )   

            //* All transaction operations of our command line */
            .subcommand(Command::new("send")
                .about("Reward Someone!!")
                .arg(arg!(<FROM>"'Source wallet address'"))
                .arg(arg!(<TO>"'Destination wallet address'"))
                .arg(arg!(<AMOUNT>"Amount to be transfered"))
            )
            .get_matches();

        //? All the matches related to the command line 

        //* All the get matchings */

        //function to get the balance of an user
        if let Some(ref matches) = matches.subcommand_matches("get-balance"){
            if let Some(address) = matches.get_one::<String>("ADDRESS"){
                let pub_key_hash = Address::decode(address).unwrap().body;
                let bc = Blockchain::new()?;
                let utoxs = bc.find_utxo(pub_key_hash.clone());
                let mut balance:i32 = 0;
                for out in utoxs{
                    balance+=out.value;
                }
                println!(
                    "
                    Account: {}
                    Balance: {}
                    ",
                    &address,&balance
                )
            }
        }

        //function to get a list of all addresses of wallets present in database
        if let Some(ref _matches)=matches.subcommand_matches("list-addresses"){
            let wallets = Wallets::new()?;
            let addresses = wallets.get_all_addresses();
            println!("Addresses:");
            for address in addresses{
                println!("{}",address);
            }
        }

        //function to print our blockchain
        if let Some(ref _matches)=matches.subcommand_matches("print-chain"){
            self.print_chain();
        }

        //* All the creation matches of our command line */

        //Function to create a new blockchain with a coinbase
        if let Some(ref matches) = matches.subcommand_matches("create"){
            if let Some(address) = matches.get_one::<String>("ADDRESS"){
                let address: String = String::from(address);
                let _response = Blockchain::create_blockchain(address.clone());
                println!("Created Blockchain");
            }
        }

        //Function to create a new wallet in the blockchain
        if let Some(ref _matches) = matches.subcommand_matches("create-wallet"){
            let mut wallets = Wallets::new()?;
            let address = wallets.create_wallet();
            wallets.save_all()?;
            println!("Success: {}",address);
        }

        //* All the transaction matches of our command line */

        //Function to send currency from and to particular address, a partcular amount
        if let Some(ref matches) = matches.subcommand_matches("send"){
            let from: &String = if let Some(address) = matches.get_one::<String>("FROM"){
                address
            }else{
                println!("From not supply! usage");
                exit(1)
            };

            let to = if let Some(address) = matches.get_one::<String>("TO"){
                address
            }else{
                println!("From not supply! usage");
                exit(1)
            };

            let amount: i32 = if let Some(amount) = matches.get_one::<String>("AMOUNT"){
                amount.parse().expect("Parsing error!")
            }else{
                println!("From not supply! usage");
                exit(1)
            };

            let mut bc = Blockchain::new()?;
            let tx = Transaction::new_utxo(from, to, amount, &bc).unwrap_or_else(|err|{
                todo!("Can't create transaction: {}",err);
            });
            bc.add_block(vec![tx])?;
            println!("Success");
        }   

        Ok(())
    }

    //? Functions

    //Print function to print our blockchain using blockchain iterator
    fn print_chain(&self){
        let b = Blockchain::new().unwrap();
        for block in b.iter(){
            println!("{:#?}",block);
        }
    }
}

//testing
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn test_decode(){
        let address = "3HWd4D3Li8bJbonVuNDZnxcRZygozMTriz";
        let pub_key_hash = Address::decode(address).unwrap().body;
        println!("{:?}",pub_key_hash);
    }
    #[test]
    fn test_find_utxos() -> Result<(),io::Error>{
        let address = "3HWd4D3Li8bJbonVuNDZnxcRZygozMTriz";
        let pub_key_hash = Address::decode(address).unwrap().body;
        let chain = Blockchain::new()?;
        let utoxs = chain.find_utxo(pub_key_hash.clone());
        println!("{:?}",utoxs);
        Ok(())
    }
}