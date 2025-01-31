//! In Module 1, we discussed Block ciphers like AES. Block ciphers have a fixed length input.
//! Real wold data that we wish to encrypt _may_ be exactly the right length, but is probably not.
//! When your data is too short, you can simply pad it up to the correct length.
//! When your data is too long, you have some options.
//!
//! In this exercise, we will explore a few of the common ways that large pieces of data can be
//! broken up and combined in order to encrypt it with a fixed-length block cipher.
//!
//! WARNING: ECB MODE IS NOT SECURE.
//! Seriously, ECB is NOT secure. Don't use it irl. We are implementing it here to understand _why_
//! it is not secure and make the point that the most straight-forward approach isn't always the
//! best, and can sometimes be trivially broken.

use aes::{
	cipher::{generic_array::GenericArray, BlockCipher, BlockDecrypt, BlockEncrypt, KeyInit},
	Aes128,
};

use rand::Rng;

///We're using AES 128 which has 16-byte (128 bit) blocks.
const BLOCK_SIZE: usize = 16;

fn main() {
	todo!("Maybe this should be a library crate. TBD");
}

/// Simple AES encryption
/// Helper function to make the core AES block cipher easier to understand.
fn aes_encrypt(data: [u8; BLOCK_SIZE], key: &[u8; BLOCK_SIZE]) -> [u8; BLOCK_SIZE] {
	// Convert the inputs to the necessary data type
	let mut block = GenericArray::from(data);
	let key = GenericArray::from(*key);

	let cipher = Aes128::new(&key);

	cipher.encrypt_block(&mut block);

	block.into()
}

/// Simple AES encryption
/// Helper function to make the core AES block cipher easier to understand.
fn aes_decrypt(data: [u8; BLOCK_SIZE], key: &[u8; BLOCK_SIZE]) -> [u8; BLOCK_SIZE] {
	// Convert the inputs to the necessary data type
	let mut block = GenericArray::from(data);
	let key = GenericArray::from(*key);

	let cipher = Aes128::new(&key);

	cipher.decrypt_block(&mut block);

	block.into()
}

/// Before we can begin encrypting our raw data, we need it to be a multiple of the
/// block length which is 16 bytes (128 bits) in AES128.
///
/// The padding algorithm here is actually not trivial. The trouble is that if we just
/// naively throw a bunch of zeros on the end, there is no way to know, later, whether
/// those zeros are padding, or part of the message, or some of each.
///
/// The scheme works like this. If the data is not a multiple of the block length,  we
/// compute how many pad bytes we need, and then write that number into the last several bytes.
/// Later we look at the last byte, and remove that number of bytes.
///
/// But if the data _is_ a multiple of the block length, then we have a problem. We don't want
/// to later look at the last byte and remove part of the data. Instead, in this case, we add
/// another entire block containing the block length in each byte. In our case,
/// [16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16]
fn pad(mut data: Vec<u8>) -> Vec<u8> {
	// When twe have a multiple the second term is 0
	let number_pad_bytes = BLOCK_SIZE - data.len() % BLOCK_SIZE;

	for _ in 0..number_pad_bytes {
		data.push(number_pad_bytes as u8);
	}

	data
}

/// Groups the data into BLOCK_SIZE blocks. Assumes the data is already
/// a multiple of the block size. If this is not the case, call `pad` first.
fn group(data: Vec<u8>) -> Vec<[u8; BLOCK_SIZE]> {
	let mut blocks = Vec::new();
	let mut i = 0;
	while i < data.len() {
		let mut block: [u8; BLOCK_SIZE] = Default::default();
		block.copy_from_slice(&data[i..i + BLOCK_SIZE]);
		blocks.push(block);

		i += BLOCK_SIZE;
	}

	blocks
}

/// Does the opposite of the group function
fn un_group(blocks: Vec<[u8; BLOCK_SIZE]>) -> Vec<u8> {
	blocks.concat()
}

/// Does the opposite of the pad function.
fn un_pad(mut data: Vec<u8>) -> Vec<u8> {
    let number_of_bytes_to_remove = data.pop().unwrap();
    for _ in 0..number_of_bytes_to_remove-1{
        data.pop();
    }
    data
}

/// The first mode we will implement is the Electronic Code Book, or ECB mode.
/// Warning: THIS MODE IS NOT SECURE!!!!
///
/// This is probably the first thing you think of when considering how to encrypt
/// large data. In this mode we simply encrypt each block of data under the same key.
/// One good thing about this mode is that it is parallelizable. But to see why it is
/// insecure look at: https://www.ubiqsecurity.com/wp-content/uploads/2022/02/ECB2.png
fn ecb_encrypt(plain_text: Vec<u8>, key: [u8; 16]) -> Vec<u8> {
	let blocks = group(pad(plain_text));

    let ciphers:Vec<[u8; BLOCK_SIZE]> = blocks.iter().map(|block| aes_encrypt(*block, &key))
        .collect();

    un_group(ciphers)
}

/// Opposite of ecb_encrypt.
fn ecb_decrypt(cipher_text: Vec<u8>, key: [u8; BLOCK_SIZE]) -> Vec<u8> {
    let ciphers:Vec<[u8; BLOCK_SIZE]> = group(cipher_text);

    let blocks: Vec<[u8; 16]> = ciphers.iter().map(|cipher| aes_decrypt(*cipher, &key)).collect();

    un_pad(un_group(blocks))
}

/// The next mode, which you can implement on your own is cipherblock chaining.
/// This mode actually is secure, and it often used in real world applications.
///
/// In this mode, the ciphertext from the first block is XORed with the
/// plaintext of the next block before it is encrypted.
///
/// For more information, and a very clear diagram,
/// see https://de.wikipedia.org/wiki/Cipher_Block_Chaining_Mode
///
/// You will need to generate a random initialization vector (IV) to encrypt the
/// very first block because it doesn't have a previous block. Typically this IV
/// is inserted as the first block of ciphertext.
fn cbc_encrypt(plain_text: Vec<u8>, key: [u8; BLOCK_SIZE]) -> Vec<u8> {
	// Remember to generate a random initialization vector for the first block.
	let blocks = group(pad(plain_text));

    let mut nonce:[u8; BLOCK_SIZE] = rand::random();
    let mut ciphers:Vec<[u8; BLOCK_SIZE]> = vec![nonce]; // inserts the IV in the first block

    for i in 1..=blocks.len() {
        ciphers[i] = aes_encrypt(xor_arrays(blocks[i], nonce), &key);
        nonce = ciphers[i];
    }

    un_group(ciphers)
}

fn xor_arrays(array1: [u8; BLOCK_SIZE], array2: [u8; BLOCK_SIZE]) -> [u8; BLOCK_SIZE] {
    let mut result: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];
    
    for i in 0..BLOCK_SIZE {
        result[i] = array1[i] ^ array2[i];
    }
    
    result
}

fn cbc_decrypt(cipher_text: Vec<u8>, key: [u8; BLOCK_SIZE]) -> Vec<u8> {
    let mut ciphers:Vec<[u8; BLOCK_SIZE]> = group(cipher_text);

    // retreive nonce and remove it
    let mut nonce:[u8; BLOCK_SIZE] = ciphers[0];
    ciphers.remove(0);

    let mut blocks: Vec<[u8; 16]> = Vec::new();

    for i in 0..ciphers.len() {
        let block = aes_decrypt(ciphers[i], &key);
        blocks[i] = xor_arrays(block, nonce);
        nonce = blocks[i]
    }

    // remove the
    blocks.remove(0);
    un_pad(un_group(blocks))
}

/// Another mode which you can implement on your own is counter mode.
/// This mode is secure as well, and is used in real world applications.
/// It allows parallelized encryption and decryption, as well as random read access when decrypting.
///
/// In this mode, there is an index for each block being encrypted (the "counter"), as well as a random nonce.
/// For a 128-bit cipher, the nonce is 64 bits long.
///
/// For the ith block, the 128-bit value V of `nonce | counter` is constructed, where | denotes
/// concatenation. Then, V is encrypted with the key using ECB mode. Finally, the encrypted V is
/// XOR'd with the plaintext to produce the ciphertext.
///
/// A very clear diagram is present here:
/// https://en.wikipedia.org/wiki/Block_cipher_mode_of_operation#Counter_(CTR)
///
/// Once again, you will need to generate a random nonce which is 64 bits long. This should be
/// inserted as the first block of the ciphertext.
fn ctr_encrypt(plain_text: Vec<u8>, key: [u8; BLOCK_SIZE]) -> Vec<u8> {
	// Remember to generate a random nonce

	let blocks = group(pad(plain_text));

    let nonce:[u8; BLOCK_SIZE] = rand::random();
    let mut counter: [u8; 8] = [0; 8];

    let mut ciphers:Vec<[u8; BLOCK_SIZE]> = vec![nonce]; // adding 128 bit nonce in the front
    for i in 1..=blocks.len() {
        let encypted_v = aes_encrypt( // encrypt V
            concat_arrays(nonce, counter),
                  &key
              );

        ciphers[i] = xor_arrays( // xor block with encrypted V
            blocks[i], 
            encypted_v
        );
        increment_counter(&mut counter);
    }

    un_group(ciphers)
}

fn increment_counter(counter: &mut [u8; 8]) {
    for i in (0..8).rev() {
        if counter[i] == u8::MAX {
            counter[i] = 0;
        } else {
            counter[i] += 1;
            break;
        }
    }
}

fn concat_arrays(arr1: [u8; BLOCK_SIZE], arr2: [u8; BLOCK_SIZE/2]) -> [u8; 16] {
    let mut result = [0u8; BLOCK_SIZE]; // Create an array to hold the result
    
    // make sure to use only first 64 bits in nonce
    for i in 0..BLOCK_SIZE/2 {
        result[i] = arr1[i];
    }
    
    // Copy elements from counter
    for i in 0..BLOCK_SIZE/2 {
        result[i + 8] = arr2[i];
    }
    
    result
}

fn ctr_decrypt(cipher_text: Vec<u8>, key: [u8; BLOCK_SIZE]) -> Vec<u8> {
    let mut ciphers:Vec<[u8; BLOCK_SIZE]> = group(cipher_text);

    // retreive nonce
    let nonce:[u8; BLOCK_SIZE] = ciphers[0];
    ciphers.remove(0);

    let mut counter: [u8; 8] = [0; 8];
    let mut blocks: Vec<[u8; 16]> = Vec::new();

    for i in 0..ciphers.len() {
        // decrypt v
        let v: [u8; 16] = aes_decrypt(concat_arrays(nonce, counter), &key);
        blocks[i] = xor_arrays(ciphers[i], v);
        increment_counter(&mut counter)
    }

    un_pad(un_group(blocks))
}