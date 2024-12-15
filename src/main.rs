use std::{default, fmt::format};
use hex::{decode, encode};

fn main() {}

// Implement this to plug into `Contract::build` to generate the contracts
pub enum Cmds {
    ApproveErc20,
    TransferErc20,
    CustomCall,
}

#[derive(Default)]
struct Contract {
    calldata: String,
    calldata_offsets: Vec<usize>,
    source: String,
}
impl Contract {
    // Add more as you please/need
    pub const P1_OP: &'static str = "60";
    pub const P20_OP: &'static str = "73";
    pub const GAS_OP: &'static str = "5A";
    pub const CALL_OP: &'static str = "F1";
    pub const POP_OP: &'static str = "50";
    pub const CD_LOAD_OP: &'static str = "35";
    pub const MSTORE_OP: &'static str = "52";
    pub const SHR_OP: &'static str = "1C";

    pub const APPROVE_SIG: &'static str = "095ea7b3";

    pub fn default() -> Self {
        Self {
            calldata: String::new(),
            calldata_offsets: vec![],
            source: String::new(),
        }
    }

    pub fn build(cmds: Vec<Cmds>) -> Self {
        let mut contract = Self::default();

        // implement cmd handling 
        // ...

        contract
    }

    // Adds onto the end of our existing calldata with new PACKED inputs
    // Meaning they aren't left side padded to save gas and be more compact
    pub fn extend_calldata(&mut self, new_calldata: Vec<&str>) {
        println!("\n[Extending Calldata: {}]", new_calldata.len());
        println!("- [00] Old: {}", &self.calldata);
            
        for (i, item) in new_calldata.iter().enumerate() {
        
                // Add it onto the end of our existing calldata
            self.calldata.extend([*item]);
                    
                    // Record the offset of where we just added so we can ref it later
            match self.calldata_offsets.len() == 0 {
                true => {
                        // We don't init with anything since we wont have calldata (duh)
                        // So we add it here to initialise + our own calldata
                    self.calldata_offsets.push(0);
                    // Since we're dealing with strings it'll be double the 
                    // len -- we want the amount of bytes instead so 1/2 it
                    self.calldata_offsets.push(self.calldata.len() / 2);
                }
                false => self.calldata_offsets.push(self.calldata.len() / 2),
            }

            // println!("self.calldata_offsets {:?}", &self.calldata_offsets);
            println!("- [{:02x}] New: {}", i + 1, &self.calldata);
        }
    }

    // Convert our packed calldata into left side padded uint
    // This is what functions normally take:
    // 0x000000000000000000000000000000000000000000000000000000AABBCCDD
    // 
    // instead of right padded
    // 0xAABBCCDD000000000000000000000000000000000000000000000000000000
    //
    // But signatures for protocols always 4 bytes long at the start 
    // of the calldata
    pub fn pad_cd_to_mem(&self, cd_from: usize, cd_to: usize) -> String {
        println!("\n---\n");
        println!("[Unpack Calldata]");

        let mut seq = String::new();
        let mut mem_offset: usize = 4; // start from 4 bc we'll skip the sig
				
        // Load and store sig far left
        // 0x AABBCCDD 000000000000000000000000000000000000000000000000000000
        let sig_offset = self.calldata_offsets[cd_from];
        seq.extend([format!(
            "{}{:02x}{}{}00{}",
            Self::P1_OP,
            sig_offset,
            Self::CD_LOAD_OP,
            Self::P1_OP,
            Self::MSTORE_OP,
        )]);
				
        // Skip the first offset because that'll be our signature
        // and the following offsets are the variables we use for said
        // function call
        //
        // You can extend this to be more optimised, of course, but it
        // doesn't serve too much purpose aside from cost of tx
        for (o, offset) in self.calldata_offsets[cd_from..cd_to]
            .iter()
            .enumerate()
            .skip(1)
        {
            // edit: realised i did this in the most complex way possible lmao
            let next_offset = self.calldata_offsets[o + 1];
            let mut word = self.calldata.split_at(*offset * 2).1;
            word = word.split_at((next_offset - offset) * 2).0;
            let padded = format!("{:0>64}", word);
					
            // how much we'll shift our rigt padded word to be left padded
            // and therefore function calldata compatable
            let shr_amt = 64 - word.len();

            let calldata_load = format!("{:02x}{}", offset, Self::CD_LOAD_OP);
            let shr = format!("{:02x}{}", shr_amt, Self::SHR_OP);
            let mstore = format!("{:02x}{}", mem_offset, Self::MSTORE_OP);
            let sub_seq = format!("{}{}{}", calldata_load, shr, mstore);

            println!("[SHR 0x{:02x} Word {}]", shr_amt, word);
            println!("- [Calldata To MSTORE: {}] {}", o, sub_seq);
            println!("- [x] Padded Word {}\n", padded);
						
		    // move onto the next word 
            mem_offset += 32;
						
            seq.extend([sub_seq]);
        }

        println!("[Calldata To MSTORE: Sequence] {}", seq);
        println!("\n---");
        seq
    }
	
		
    // Modularised approve token component. Whenever you want an approve 
    // call this helper function to generate the CALL for an ERC20 approve
    // 
    // You could definitely make a single generalised function and have 
    // your monitoring system spit our the variables instead of hardcoding
    // like i did here
    pub fn approve_token(&mut self, token: &str, to: &str, amount: &str) {
    
        // Get our start offset so we can calculate our last one after
        // adding to the calldata and calldata_offsets
        let latest_offset = if let Some(x) = self.calldata_offsets.last() {
            *x
        } else {
            0
        };
			
        // Each nibble = 1 character (pretty convenient right?!)
        let calldata_len: usize = (Self::APPROVE_SIG.len() + to.len() + amount.len()) / 2;
        self.extend_calldata(vec![Self::APPROVE_SIG, to, amount]);
        
        // Calldata To Memory before CALL (which it references)
        let cd_to_mem = self.pad_cd_to_mem(latest_offset, latest_offset + 3);
				
        // CALL arguments
        // selector, word, word
        let mem_size: String = format!("{:02x}", 4 + 32 + 32);
        let ret: String = format!("{}00{}00", Self::P1_OP, Self::P1_OP);
        let arg: String = format!(
            "{}{}{}00",
            Self::P1_OP,
            mem_size,
            Self::P1_OP, // our memory 
        );
        let token: String = format!("{}{}", Self::P20_OP, token); // assuming isnt pass in as '0x...'
        let gas_call_pop: String = format!("{}{}{}", Self::GAS_OP, Self::CALL_OP, Self::POP_OP);

        let seq = format!(
            "{}{}{}{}{}",
            // we inject the "calldata to memory" anywhere to make manual
            // reversing cancer to do, but here makes it clearer rn
            cd_to_mem,
            ret,
            arg,
            token,
            gas_call_pop
        );

        let source_b = self.source.clone();
        self.source.extend([seq]);
        println!(
            "\n[Extending Source] Token Approval\n- [ ] Old {}\n- [ ] New {}\n",
            source_b, &self.source
        );
    }
}

/*
cargo test test -- --nocapture
*/
#[cfg(test)]
mod test {
    use super::*;

    /*
    cargo test test::test_approve_token -- --nocapture
     */
    #[test]
    fn test_approve_token() {
        let mut contract = Contract::default();
        contract.approve_token(
            "C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // weth
            "9FC3da866e7DF3a1c57adE1a97c9f00a70f010c8", // some randos addy
            "3635C9ADC5DEA00000", // 1,000 tokens
        );

        println!("\n[Calldata]\n{}", contract.calldata);
    }
}
