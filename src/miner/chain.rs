//? used modules for the blockchain
#[allow(unused_imports)]
use std::{ collections::HashMap, hash::Hash, io, vec}; //for creating hash maps
use crate::{miner::mining::Block, wallet::tx::TrancOutput, wallet::transaction::Transaction};//including creates
use failure::format_err;//handling error
use log::info; 
//for displaying message
use sled;//for database
use bincode::serialize; //for serialization
use serde_json;
#[allow(unused_imports)]
use bitcoincash_addr::Address;//for testing purpose

//?Difficulty level of the chain POW
static DIFFICULTY:usize = 1;

//?Blockchain struct to store the last has of the chain and the database address
#[allow(dead_code)]
//Blockchain struct storing a vector of blocks
#[derive(Debug,Clone)]
pub struct Blockchain{
    inst_hash:String,//hash of the last block in the chain
    db: sled::Db,//database
}

//?Blockchain interator struct for interating the blockchain
pub struct BlockchainIterator<'a>{
    inst_hash:String,//store the hash of the last interating blockchain
    blockchain: &'a Blockchain,//stores the blockchain with lifetime operator
}

#[allow(dead_code)]
//? implementing the Blockchain struct
impl Blockchain {
    //* function to create a new blockchain starting from a default block */
    pub fn new() -> Result<Self,io::Error>{
        info!("Opening blockchain...");//message
        //opening database
        let db = sled::open("data/blocks")?;
        //getting the LAST hash
        let hash = db.get("LAST").expect("Must create a new block database first").unwrap_or_else(||{
            todo!("LAST_HASH_ERR");
        });
        info!("Found block database!");//message
        let lasthash = String::from_utf8(hash.to_vec()).expect("Can't get last hash!");
        Ok(Blockchain { inst_hash: lasthash.clone(), db })//created a new blockchain
    }

    //* function the blockchain startingwith a default block*/
    pub fn create_blockchain(address: String) -> Result<Self,io::Error>{
        info!("Creating blockchain");//message
        //opening database
        let db = sled::open("data/blocks")?;
        info!("Creating new block in database...");//message

        //settin up a coinbase transaction
        let coinbase = Transaction::new_coinbase(address, String::from("Default coinbase")).expect("Can't create coinbase while generating new blockchain");
        let default_block = Block::default(coinbase); //passing coinbase 
        
        //inserting block in database
        db.insert(default_block.get_hash(), bincode::serialize(&default_block).expect("Can't insert new blockchain to database"))?;
        db.insert("LAST", default_block.get_hash().as_bytes())?;//setting LAST hash
        //creating new blockcain struct
        let blockchain = Blockchain{inst_hash:default_block.get_hash(),db:db};
        //flushing the database
        let _result = blockchain.db.flush();
        Ok(blockchain)
    }

    //* function to add block into the blockchain */ 
    pub fn add_block(&mut self, transaction:Vec<Transaction>)->Result<(),io::Error>{
        let lasthash = self.db.get("LAST")?.unwrap(); //getting the last hash from the db

        //creating new block using the given transactions
        let new_block: Block= Block::new(transaction, String::from_utf8(lasthash.to_vec()).expect("VECTOR_ERROR"), DIFFICULTY).unwrap_or_else(|e|{
            todo!("BLOCK_CREATION_ERROR:{}",e);//error handling in case block can'tbe created
        });
        self.db.insert(new_block.get_hash(), serialize(&new_block).unwrap_or_else(|e|
            {
                todo!("BEFORE_DATABASE_SERIALIZATION_ERROR:{}",e);//handling error during data serialization
            }
        ))?;
        self.db.insert("LAST", new_block.get_hash().as_bytes())?;// setting the LAST key
        Ok(())
    }

    //* Function for creating the first iterator */
    pub fn iter(&self) -> BlockchainIterator{
        BlockchainIterator{
            inst_hash: self.inst_hash.clone(), //setting blockchain iterator
            blockchain: &self, //blochchain
        }
    }

    //* function to load spent transactions */
    fn load_spent_transactions(&self)->HashMap<String,Vec<i32>>{
        let db = sled::open("data/spent_records").unwrap();
        let mut spent_transactions = HashMap::new();
        for item in db.into_iter(){
            if let Ok((key,value)) = item{
                let tx_id = String::from_utf8(key.to_vec()).unwrap_or_default();
                let outputs: Vec<i32> = serde_json::from_slice(&value).unwrap_or_default();
                spent_transactions.insert(tx_id,outputs);
            }
        }
        spent_transactions
    }

    //* function to save pent transactions */
    fn save_spent_transactions(&self, tx_id: &str, outputs:Vec<i32>) -> Result<(),io::Error>{
        let db = sled::open("data/spent_records")?;
        let value = serde_json::to_vec(&outputs)?;
        db.insert(tx_id, value)?;
        db.flush()?;
        println!("Transaction saved");
        Ok(())
    }

    //* function to send the list of unsent transactions */
    fn find_unspent_transactions(&self, address: Vec<u8>)->Vec<Transaction>{
        let mut spent_tx: HashMap<String,Vec<i32>> = self.load_spent_transactions();//hash map to store spent
        let mut unspent_tx: Vec<Transaction> = Vec::new();//hash to store unspent transactions

        for block in self.iter(){ //iterating Clockchain
            for tx in block.get_transaction(){ //iterating through transactions of each block
                for index in 0..tx.vout.len(){//interating through vout
                    if let Some(ids) = spent_tx.get(&tx.tranc_id){ //checking if transaction is in spent id
                        if ids.contains(&(index as i32)){
                            continue;//checking if transaction is already spent
                        }
                    }

                    if tx.vout[index].can_be_unlock_with(address.clone()){//checking public auth
                        unspent_tx.push(tx.to_owned()); //getting unsent transactions output
                    }
                }
                if !tx.is_coinbase(){ // checking if the transaction is coinbase or not
                    for i in &tx.vin{ // getting transactions in vin
                        if i.can_unlock_output_with(address.clone()){ // cheking auth with public address
                            match spent_tx.get_mut(&i.from) {
                                Some(v) => {
                                    v.push(i.vout);
                                    self.save_spent_transactions(&i.from,v.to_vec()).unwrap();
                                }
                                None => {
                                    spent_tx.insert(i.from.clone(),vec![i.vout]);
                                    self.save_spent_transactions(&i.from,vec![i.vout]).unwrap();
                                }                                
                            }
                        }
                    }
                }
            }
        }
        unspent_tx
    }

    //* funcion to find and return all unsent transaction outputs */
    pub fn find_utxo(&self, address: Vec<u8>) -> Vec<TrancOutput>{
        let mut utxos = Vec::<TrancOutput>::new(); //vec to store UTXOs
        let unspend_txs = self.find_unspent_transactions(address.clone()); //getting unspend transactions
        for tx in unspend_txs{ // iterating unspent transactions
            for out in &tx.vout{ 
                if out.can_be_unlock_with(address.clone()){
                    utxos.push(out.clone());
                }
            } 
        }
        utxos
    }

    //* function to return list of transactions containing unspent outputs */
    pub fn find_spendable_outputs(&self, address:Vec<u8> , amount: i32) -> (i32,HashMap<String,Vec<i32>>){
        let mut unspent_outputs: HashMap<String,Vec<i32>> = HashMap::new(); //getting unspent transactions
        let mut accumulated: i32 = 0; //accumanted amount from utxos
        let unspent_txs: Vec<Transaction> = self.find_unspent_transactions(address.clone());
        
        for tx in unspent_txs{
            for index in 0..tx.vout.len(){
                if tx.vout[index].can_be_unlock_with(address.clone()) && accumulated< amount{
                    match unspent_outputs.get_mut(&tx.tranc_id){
                        Some(v)=>{
                            v.push(index as i32);
                        },
                        None => {
                            unspent_outputs.insert(tx.tranc_id.clone(), vec![index as i32]);
                        }
                    }
                    accumulated+=tx.vout[index].value;
                    if accumulated>=amount{
                        return (accumulated, unspent_outputs);
                    }
                }
            }
        }
        (accumulated,unspent_outputs)
    }

    //* function to find a transaction with a particular id */
    pub fn find_transaction(&self, id: &str) -> Result<Transaction,failure::Error>{
        for block in self.iter(){
            for transaction in block.get_transaction(){
                if transaction.tranc_id == id{
                    return Ok(transaction.clone())
                }
            }
        }
        Err(format_err!("Transaction not found"))
    }

    //* function to find and give previous transactions */
    fn get_previus_txs(&self, tx:&Transaction) -> Result<HashMap<String, Transaction>,io::Error>{
        let mut prev_txs = HashMap::new();
        for vin in &tx.vin{
            let prev_tx = self.find_transaction(&vin.from).unwrap();
            prev_txs.insert(prev_tx.tranc_id.clone(), prev_tx);
        }
        Ok(prev_txs)
    }

    //* function to sign a transaction for auth */
    pub fn sign_transaction(&self, tx: &mut Transaction, private_key: &[u8]) -> Result<(),io::Error>{
        let prev_txs = self.get_previus_txs(tx)?;
        tx.sign(private_key, prev_txs).unwrap();
        Ok(())
    }

    //* function to verify a signed transaction with the public key */
    pub fn verify_transaction(&self, tx: &mut Transaction) -> Result<bool,failure::Error>{
        let prev_txs = self.get_previus_txs(tx)?;
        tx.verify(prev_txs)
    }
}

//?interator implementation
impl<'a> Iterator for BlockchainIterator<'a>{
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item>{
        if let Ok(encoded_block) = self.blockchain.db.get(&self.inst_hash){
            match encoded_block{
                Some(data)=>{
                    if let Ok(block) = bincode::deserialize::<Block>(&data){
                        if block.get_previus_hash()==""{
                            self.inst_hash.clear();
                            return Some(block);
                        }
                        self.inst_hash = block.get_previus_hash();
                        return Some(block);
                    }
                }
                None => ()
            }
        }
        None 
    }
}

// testing code

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn test_utxo()->Result<(),io::Error>{
        let address = "3HWd4D3Li8bJbonVuNDZnxcRZygozMTriz";
        let pub_key_hash = Address::decode(address).unwrap().body;
        // let mut utxos = Vec::<TrancOutput>::new(); 
        let chain = Blockchain::new()?;
        let _unspend_txs = chain.find_unspent_transactions(pub_key_hash.clone());
        println!("Fetching done");
        Ok(())
    }
    #[test]
    fn test_unspent_transactions()->Result<(),io::Error>{
        let address = "3HWd4D3Li8bJbonVuNDZnxcRZygozMTriz";
        let pub_key_hash = Address::decode(address).unwrap().body;
        let chain = Blockchain::new()?;
        let mut spent_tx: HashMap<String,Vec<i32>> = HashMap::new();//hash map to store spent
        let mut unspent_tx: Vec<Transaction> = Vec::new();//hash to store unspent transactions

        for block in chain.iter(){ //iterating Clockchain
            for tx in block.get_transaction(){ //iterating through transactions of each block
                for index in 0..tx.vout.len(){//interating through vout
                    if let Some(ids) = spent_tx.get(&tx.tranc_id){ //checking if transaction is in spent id
                        if ids.contains(&(index as i32)){
                            continue;//checking if transaction is already spent
                        }
                    }

                    if tx.vout[index].can_be_unlock_with(pub_key_hash.clone()){//checking public auth
                        unspent_tx.push(tx.to_owned()); //getting unsent transactions output
                    }
                }
                if !tx.is_coinbase(){ // checking if the transaction is coinbase or not
                    for i in &tx.vin{ // getting transactions in vin
                        if i.can_unlock_output_with(pub_key_hash.clone()){ // cheking auth with public address
                            match spent_tx.get_mut(&i.from) {
                                Some(v) => {
                                    v.push(i.vout);
                                }
                                None => {
                                    spent_tx.insert(i.from.clone(),vec![i.vout]);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
    #[test]
    fn test_uto()->Result<(),io::Error>{
        let db = sled::open("data/spent_records")?;
        println!("database loaded");
        println!("{}",db.iter().count());
        for item in db.iter(){
            println!("inside loop");
            if let Ok((key,value)) = item{
                let tx_id = String::from_utf8(key.to_vec()).unwrap_or_default();
                let outputs:Vec<i32> = serde_json::from_slice::<Vec<i32>>(&value).unwrap_or_default();
                println!("id: {}",tx_id);
                for i in outputs{
                    println!("value: {}",i);
                }
            }
        }
        Ok(())
    }
}
