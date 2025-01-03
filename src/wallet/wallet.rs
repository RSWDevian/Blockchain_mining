//? Modules used in Wallet
use std::{collections::HashMap, io}; // for generating hashmap and io errors
use crypto::digest::Digest; // for hashing
use crypto::ripemd160::Ripemd160;// to rehash the SHA256 output for getting a 160 bit(20bytes) hashed public key
use crypto::sha2::Sha256; // Sha256 algorithm for hashing 256bits (32bytes) output
use log::info; //for message displaying
use serde::{Serialize,Deserialize}; // for serialization and deserialization 
use crypto::ed25519;// Edwards Elliptic Curve for encryption 
use rand::RngCore; // for generating random number
use rand::rngs::OsRng; // for generating random number
use bitcoincash_addr::{Address,HashType,Scheme}; //for address creating, hash data storage and scheme declaration

//? Wallet 
#[derive(Serialize,Deserialize,Debug,Clone,PartialEq)]
pub struct Wallet{
    pub secret_key: Vec<u8>, // secret key/private key of the user 
    pub public_key: Vec<u8>, // public key of the user
}

//? Implementations of the Wallet struct
impl Wallet{
    //* new function for creating a new wallet for a user and generating key-pair relation */
    fn new() -> Self {
        let mut key: [u8; 32] = [0; 32];// Declare a mutable 32-byte array for a random seed for ed25519
        OsRng.fill_bytes(&mut key); // generating random number for cryptographic security
        let (secret,public) = ed25519::keypair(&key); //Generate a keypair using ED25519 elliptic curve algorithm
        let secret_key = secret.to_vec(); //converting secret key to vector and assigning it 
        let public_key = public.to_vec(); //converting public key to vector and assigning it
        Wallet { secret_key, public_key } // Creating a new Wallet instance and returning it 
    }

    //* Generates a cryptographic address for the wallet for identification */
    fn get_address(&self) -> String{
        let mut pub_hash: Vec<u8> = self.public_key.clone(); // cloning & storing the public hash 
        hash_pub_key(&mut pub_hash); //hashing the pub key for generating a unique identifier
        println!("{:?}",pub_hash);
        let address = Address{
            body: pub_hash, //body og the address
            scheme: Scheme::Base58, //Encoding Scheme: Base58
            hash_type: HashType::Script, // script hash 
            ..Default::default()
        };
        address.encode().unwrap() //converting public key to Base58 human readable form
    }
}


//? public function for hashing the public key using Sha256 to 32 bytes
//? then encrypting with Rimpemd160 algorithm to get a 20 bytes public key
pub fn hash_pub_key(pub_key: &mut Vec<u8>){
    let mut hasher1: Sha256 = Sha256::new(); //creating Sha256 object
    hasher1.input(pub_key); //digesting the public key
    let mut sha256_hash = [0u8; 32];
    hasher1.result(&mut sha256_hash); //result of the digested clone public key
    let mut hasher2: Ripemd160 = Ripemd160::new(); //creating Rimpemd160 object
    hasher2.input(&sha256_hash); //digesting the hashed public key
    let mut ripemd160_hash = [0u8; 20];
    pub_key.resize(20, 0); // setting the length
    hasher2.result(&mut ripemd160_hash); //setting the 20 bytes public key  

    pub_key.clear(); 
    pub_key.extend_from_slice(&ripemd160_hash);  
}

//? Struct for storing multiple wallets in a hashmap and then uploading them to the sled db
#[derive(Serialize,Deserialize,Debug,Clone,PartialEq)]
pub struct Wallets{
    wallets: HashMap<String,Wallet> //hashmap of wallets
}

//? implementations of the Wallets struct
impl Wallets {
    //* new function to new wallets struct for creating and storing wallets */
    pub fn new() -> Result<Wallets,io::Error>{
        let mut wallets = Wallets{
            wallets: HashMap::<String,Wallet>::new(), //creating an instance of wallets struct
        };

        let db = sled::open("data/wallets")?;//opening the sled database

        for item in db.into_iter(){
            let i = item?; //cheking the iterator and storing it
            let address = String::from_utf8(i.0.to_vec()).unwrap();//getting the address of the wallet 
            let wallet = bincode::deserialize::<Wallet>(&i.1.to_vec()).unwrap();//getting the wallet
            wallets.wallets.insert(address, wallet);//storing it in the hashmap 
        }
        drop(db);//to dispose off the called database object
        Ok(wallets)//returning the wallets function
    }

    //* function to create a new wallet for a new user */
    pub fn create_wallet(&mut self)->String{
        let wallet = Wallet::new(); //creating a new instance of the wallet 
        let address = wallet.get_address(); // getting the Base58 hashed and encrypted public of the wallet
        self.wallets.insert(address.clone(), wallet);//inserting the wallet into the hashmap
        info!("Creating wallet: {}",address); //returning the public address of the user
        address //returning the address 
    }

    //* function to get all the address or all public addresses for a user */
    pub fn get_all_addresses(&self) -> Vec<String>{
        let mut addresses: Vec<String> = Vec::new(); //declaring a string vector for storing the public addresses
        for (address,_) in &self.wallets{
            addresses.push(address.clone());//interating and pushing the addresses into the vector
        }
        addresses //returning the address list
    }

    //* To get wallet corresponding to the particular address */
    pub fn get_wallet(&self,address: &str) -> Option<Wallet>{
        self.wallets.get(address).cloned()
    }

    //* to save the newly created wallet and all other data into the sled db and flush it = */
    pub fn save_all(&self)->Result<(),io::Error>{
        let db = sled::open("data/wallets")?; //opening the sled database
        for (address,wallet) in &self.wallets{
            let data: Vec<u8> = bincode::serialize(wallet).unwrap(); //serializing the data from string to vec<u8>
            db.insert(address, data)?;// inserting as key-value pair
        }
        db.flush()?; //flusing the database
        drop(db); //dropping the database object
        Ok(())
    }
}