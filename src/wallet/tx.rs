//? Modules used 
use bitcoincash_addr::Address;
use log::debug; //for showig debug result
use serde::{Deserialize, Serialize}; // for serialization and deserialization
use super::wallet::hash_pub_key;//impoorting the hash_pub_key function from the wallet 

//? Transaction input refering to previous UTXO outputs to be used as an input source
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TrancInput {
    pub from: String,//public key address of the sender
    pub vout: i32,
    pub signature: Vec<u8>,//signature of the sender, showing authentication
    pub pub_key: Vec<u8>,//public key of the sender for recipitant verification
}

//? Represents the transaction output creating a new UTXO for the receiver
#[derive(Debug, Deserialize, Serialize,Clone)]
pub struct TrancOutput {
    pub value: i32,//value transfered
    pub pub_key_hash: Vec<u8>,//the hashed public key of the recipitent

}

//? implementint the TrancInput struct
impl TrancInput {
    //* Checks whether same address initiated the transaction */
    pub fn can_unlock_output_with(&self, unlocking_data: Vec<u8>) -> bool {
        let mut pubkeyhash = self.pub_key.clone();//cloning the unlocking data 
        hash_pub_key(&mut pubkeyhash); //creating Base58 pubkey 
        pubkeyhash == unlocking_data //comparing and sharing the data
    }
}

//? implementing the TrancOutput struct
impl TrancOutput {
    pub fn can_be_unlock_with(&self, unlocking_data: Vec<u8>) -> bool {
        self.pub_key_hash == unlocking_data
    }

    pub fn lock(&mut self, address: &str) -> Result<(),failure::Error>{
        let pub_key_hash = Address::decode(address).unwrap().body;//decoding the pub key hash to vec<u8> from Base58
        debug!("Lock: {}",address); //Locking the transaction address
        self.pub_key_hash = pub_key_hash; //setting the recipitant address
        Ok(())
    }

    //* To create a new transaction output */
    pub fn new(value: i32, address: String) -> Result<Self,failure::Error>{
        let mut trancoutput = TrancOutput{
            value, //setting transaction value
            pub_key_hash: Vec::new(), //initislizing
        };
        trancoutput.lock(&address)?; //setting the pub key hash
        Ok(trancoutput)
    }
}
#[cfg(test)]
mod tests{
    // use core::hash;

    use super::*;
    #[test]
    fn test_can_unlock(){
        let address = "3HWd4D3Li8bJbonVuNDZnxcRZygozMTriz";
        let pub_key_hash = Address::decode(address).unwrap().body;
        let mut clone = pub_key_hash.clone();    
        hash_pub_key(&mut clone);    
    }
}