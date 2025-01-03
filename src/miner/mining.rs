//? modules used in mining the chain
use crate::wallet::transaction::Transaction;
use bincode; //to serialize and deserialize to binary code
use crypto::digest::Digest;
use crypto::sha2::Sha256; // Sha256 alorithm for hashing
use log::info; //to  print info related text in the blockchain
use serde::{Deserialize, Serialize}; // for serialization and deserialization of structs
use std::io::{self}; //self property of implementation
use std::time::SystemTime; // for getting timestamp //wallet imported from transaction

//? Block of blockchain storing list of transactions and proof of work
///Each Block containing
///  -> timestamp: related to UNIX_EPOX
///  -> transactions: for storing the list of transactions for the particular block in blockchain
///  -> prev_block_hash: containing previous block hash to interate in blockchain
///  -> hash: the proof of work for the chain
///  -> height: is basically the difficulty of the proof of work here
///  -> nonce: is the random unique number for each block related to the proof-of-work
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    timestamp: u128,
    transactions: Vec<Transaction>,
    prev_block_hash: String,
    hash: String,
    _height: usize,
    nonce: i32,
}

//? global difficulty for the proof of work
static DIFFICULTY: usize = 1;

#[allow(dead_code)]
//? implementations of the 'Block' struct
impl Block {
    //* new function to create a new block in the blockchain */
    pub fn new(
        data: Vec<Transaction>,
        prev_block_hash: String,
        height: usize,
    ) -> Result<Self, io::Error> {
        let timestamp: u128 = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
            .as_millis(); //getting timestamp in miliseconds

        let mut block: Block = Block {
            timestamp,//time of block creation
            transactions: data,//transaction vector dataset
            prev_block_hash,//previus block hash used to iterate the block chain
            hash: String::new(),//proof of work 
            _height: height,//height of the block
            nonce: 0,//random number 
        };//creating a new instance of the block struct

        block.generate_proof_of_work();//generating proof of work for our block 
        Ok(block)
    }

    //* to create a default block in the blockchain for the miner coinbase */
    pub fn default(coinbase: Transaction) -> Block {
        //no previous block hash
        //coinbase for miner passed as data
        Block::new(vec![coinbase], String::new(), DIFFICULTY).unwrap()
    }

    //* Function to prepare list of data for generating the proof of work */
    fn prepare_hash_data(&self) -> Result<Vec<u8>, io::Error> {
        //the whole content will collectively act as a source for generating POW
        let content: (String, Vec<Transaction>, u128, usize, i32) = (
            self.prev_block_hash.clone(),//previous block hash
            self.transactions.clone(),//transactions
            self.timestamp,//timestamp 
            DIFFICULTY,//difficulty of the POW
            self.nonce,//nonce
        );

        //to serialize the list to binary format(a vector of)
        let bytes: Vec<u8> = bincode::serialize(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;//for error mapping
        Ok(bytes)
    }

    //* To validate whther for the particular nonce the level of difficulty is reached or not */
    fn validate(&self) -> Result<bool, io::Error> {
        let data: Vec<u8> = self.prepare_hash_data()?;//getting hash data for validation
        let mut hasher = Sha256::new();//using Sha256 algorithm to generate the hash
        hasher.input(&data[..]);//digesting the hash data
        Ok(hasher.result_str().starts_with(&"0".repeat(DIFFICULTY)))//checking the number of starting zeroes
    }

    //* Generating the POW for a particular block to validate the chain */
    fn generate_proof_of_work(&mut self) {
        info!("Doing the mining work on the block");//info message 
        while !self.validate().unwrap_or_else(|e| {
            todo!("NONCE_VALIDATION_ERROR: {}", e);//handling validation error
        }) {
            self.nonce += 1;//adjusting the nonce
        }

        //preparing the POW for the particular block
        let data = self.prepare_hash_data().unwrap_or_else(|e| {
            todo!("FETCHING_ERROR: {}", e);//handling error during fetching data
        });

        let mut hasher = Sha256::new();// using Sha256 algo to prepare the POW
        hasher.input(&data);//digesting the hash data
        self.hash = hasher.result_str();//setting the POW for the particular block
    }

    //? Additional implementations of the Block Struct

    //* To get the previous hash of the block */
    pub fn get_previus_hash(&self) -> String {
        self.prev_block_hash.clone()
    }

    //* To get the POW of the current block */
    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }

    //* To get transaction details of the block */
    pub fn get_transaction(&self) -> Vec<Transaction> {
        self.transactions.clone()
    }
}
