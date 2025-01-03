//? Modules used in transaction
use bincode::{self}; //for binary serialization
use crypto::ed25519; //for digital signature funtionality(Edwards Elliptic Curve)
use crypto::{digest::Digest, sha2::Sha256}; //hasing
use failure::format_err; //for handling errors
use serde::{Deserialize, Serialize}; // for serialization and deserialization
use std::{collections::HashMap,io}; // for generating hashmaps
use crate::miner::chain::Blockchain; //importing the blockchain module 
use crate::wallet::tx::{TrancInput,TrancOutput}; //imporint the Transaction Input-Output structs
use super::wallet::{hash_pub_key, Wallets}; //using wallet functions

//? Blockchain transaction struct
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Transaction {
    pub tranc_id: String, //unique id featuring a particular transaction
    pub vin: Vec<TrancInput>, // refering the previous UTXO output from transaction output as transaction input
    pub vout: Vec<TrancOutput>, // using the previous UTXO as input and sending a new UTXO as output to the recipitant
}

//? implementing transaction functions
impl Transaction {

    //* For creating a new transaction utxo */
    /// from: sender address
    /// to: receiver address
    /// amount: to be sent from sender to receiver
    /// blockchain: the involved blockchain
    pub fn new_utxo(from: &str, to: &str, amount: i32, blockchain: &Blockchain) -> Result<Transaction,failure::Error>{
        let mut vin = Vec::new();//for storing previous UTXO outputs to refer as input
        let wallets = Wallets::new()?; // creating a wallets object 
        
        //getting a wallet for a particular address(from) from the wallets hashmap 
        let wallet = match wallets.get_wallet(from){
            Some(w) => w, //returning it there is 
            None => return Err(format_err!("SENDER_WALLET_NOT_FOUND!")),//Handling error from sender side
        };

        //checking if reciever address is correct or not
        if let None = wallets.get_wallet(to){
            return Err(format_err!("RECEIVER_WALLET_NOT_FOUND!"));//handlin error from receiver side
        }

        let mut pub_key_hash: Vec<u8> = wallet.public_key.clone(); //cloning senders public key
        hash_pub_key(&mut pub_key_hash); //Creating hash using Sha256 and Rimpemd160

        //for findnig spendable UTXO outputs from previous transactions
        let acc_v = blockchain.find_spendable_outputs(pub_key_hash, amount); //storing spendable UTXOs 
        if acc_v.0<amount {
            eprintln!("NOT_ENOUGH_BALANCE");//for handling error in case 
            return Err(format_err!("NOT_ENOUGH_BALANCE: Current Balance {}",acc_v.0));//handling error for not enough amount in senders account
        }

        //iterating through the transaction output UTXOs to include them in vin 
        for tx in acc_v.1{
            for out in tx.1{
                let input = TrancInput{
                    from:tx.0.clone(),//previous UTXO senders name
                    vout:out,//amount taken from the UTXO
                    signature:Vec::new(),//signature initialisation 
                    pub_key: wallet.public_key.clone(), //public key of the sender
                };
                vin.push(input);
            }
        }
        //setting up vout
        let mut vout = vec![TrancOutput::new(
            amount,//amount to be transfered to the receiver
            to.to_string()//public address of the receiver
        )?];
        if acc_v.0>amount{
            vout.push(
                TrancOutput::new(
                    acc_v.0-amount,//showing the remaining amount in the account 
                    from.to_string()//senders public address
                )?
            )
        }

        //creating the transaction
        let mut transaction = Transaction{
            tranc_id: String::new(),//initialising transaction id 
            vin,//UTXO inputs from previous outputs
            vout,//UTXO output from affordable inputs
        };
        transaction.tranc_id = transaction.hash()?; //setting up the transaction ID
        let _response = blockchain.sign_transaction(&mut transaction, &wallet.secret_key); //signing the transaction for auth 
        Ok(transaction)//successful tansaction
    }

    //* function for creating a new coinbase transaction */
    //to: miner address
    //data: Message for the miner
    pub fn new_coinbase(to: String, mut data: String) -> Result<Self, io::Error> {
        if data == String::from("") {//default minor data condition
            data += &format!("Reward to {}", to);//message
        }

        //creating coinbase transaction
        let mut transaction: Transaction = Transaction {
            tranc_id: String::new(),//transaction id
            vin: vec![TrancInput {
                from: String::new(),//no sender
                vout: -1,//no UTXO(blockchain net quantity increases with each mining)
                signature: Vec::new(),//no signature needed 
                pub_key: Vec::from(data.as_bytes()),//just a default address
            }],
            vout: vec![TrancOutput::new(100,to).unwrap()],//100coins rewarded to the miner
        };
        transaction.tranc_id = transaction.hash()?;//setting the coinbase transaction id 
        Ok(transaction)//successfull transaction
    }

    //* function to check whether a block is a coin base */
    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.tranc_id.is_empty() && self.vin[0].vout == -1 //condition for coinbase
    }

    //* function to sign the signature */
    //prinvate_key = private key of the sender
    pub fn sign(&mut self, private_key: &[u8], prev_txs: HashMap<String,Transaction>) -> Result<(),failure::Error>{
        if self.is_coinbase(){ //checking is the given transaction is a coinbase trnsaction 
            return Ok(());//so signature required for coinbase transaction
        }

        //checking if all transactions are validated by ids or not
        for vin in &self.vin{
            if prev_txs.get(&vin.from).unwrap().tranc_id.is_empty(){//checking condition
                return Err(format_err!("PREVIOUS_TRANSACTION_FAULT_ID_ERROR")); //handling error
            }
        }
        let mut tranc_copy = self.trim_copy(); //getting a full clone of the transaction

        for in_id in 0..tranc_copy.vin.len(){
            let prev_tx = prev_txs.get(&tranc_copy.vin[in_id].from).unwrap();//getting transaction
            tranc_copy.vin[in_id].signature.clear(); //clearing the previous signature
            tranc_copy.vin[in_id].pub_key = prev_tx.vout[tranc_copy.vin[in_id].vout as usize]
                .pub_key_hash.clone();
            tranc_copy.tranc_id = tranc_copy.hash()?;
            tranc_copy.vin[in_id].pub_key = Vec::new();//removing public key
            let signature = ed25519::signature(tranc_copy.tranc_id.as_bytes(), private_key); //signing each UTXO input from previous output
            self.vin[in_id].signature = signature.to_vec(); //assigning signature to the transaction vin
        }
        Ok(())
    }

    //* Function to verify the signature of a transaction */
    ///similar to sign function
    ///only difference in verification and signing step 
    pub fn verify(&mut self, prev_txs: HashMap<String,Transaction>) -> Result<bool,failure::Error>{
        if self.is_coinbase(){
            return Ok(true);
        }

        for vin in &self.vin{
            if prev_txs.get(&vin.from).unwrap().tranc_id.is_empty(){
                return Err(format_err!("ERROR: Previous transaction is not correct"));
            }
        }

        let mut tranc_copy = self.trim_copy();//making copy

        for in_id in 0..self.vin.len(){
            let prev_tx = prev_txs.get(&self.vin[in_id].from).unwrap();
            tranc_copy.vin[in_id].signature.clear();
            tranc_copy.vin[in_id].pub_key = prev_tx.vout[tranc_copy.vin[in_id].vout as usize]
                .pub_key_hash.clone();
            tranc_copy.tranc_id = tranc_copy.hash()?;
            tranc_copy.vin[in_id].pub_key = Vec::new();

            //using elliptic curve for signature verification
            if !ed25519::verify(
                &tranc_copy.tranc_id.as_bytes(), //transaction id
                &self.vin[in_id].signature, //transaction public address
                &self.vin[in_id].signature, //previously generated signatire
            ){
                return Ok(false);
            }
        }
        Ok(true)
    }

    //* function to hash a transaction id can return it */
    fn hash(&mut self) -> Result<String,io::Error>{
        self.tranc_id = String::new(); //initialising
        let data = bincode::serialize(&self).unwrap_or_else(|e|{
            todo!("SIG_SERIALIZATION_ERR:{}",e);
        });
        let mut hasher = Sha256::new();//Sha256 hashing algo
        hasher.input(&data);//digesting data
        Ok(hasher.result_str())
    }

    //* function to copy a transaction and return it */
    //Done so as not to effect the original data
    fn trim_copy(&self) -> Transaction{
        let mut vin = Vec::new(); //for storing UTXO inputs
        let mut vout = Vec::new();//for storing UTXO output

        //cloning the vin 
        for v in &self.vin{
            vin.push(TrancInput{
                from: v.from.clone(),
                vout: v.vout.clone(),
                signature: v.signature.clone(),
                pub_key: v.pub_key.clone(),
            });
        }

        //cloning the vout
        for v in &self.vout{
            vout.push(TrancOutput{
                value: v.value.clone(),
                pub_key_hash: v.pub_key_hash.clone(),
            });
        }

        Transaction { tranc_id: self.tranc_id.clone() , vin, vout} //returning the cloned transaction
    }
}

