use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::{LineWriter, Write};
use std::path::Path;

use hack_assembler::lib::AsmParser;
// test
// nand2tetris project 6 hack assembler source code
// usage:
// pass in the path of a c:\*.asm file as an argument e.g. ./hack_assembler cmyHackAssemblyFile.asm
// it will output a myHackAssembly.hack file
// use this file for project 6 requirements

fn check_valid_asm_file(args: &[String]) -> Result<&Path, Box<dyn Error>> {
    if args.len() != 2 {
        Err("Please enter a file path as an argument to the program.".to_string())?
    }
    let file_path = Path::new(&args[1]);
    let extension = file_path.extension();
    
    if !extension.is_some_and(|ext| ext == "asm") {
        Err("Please enter a file path that is of *.asm type to the program.".to_string())?
    }

    Ok(file_path)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let file_path = check_valid_asm_file(&args)?;
    let contents = fs::read_to_string(file_path)?;
    let asm_parser = AsmParser::new(contents);

    // initialize symbol table
    let mut symbol_table: HashMap<String, i16> = HashMap::new();
    symbol_table.insert(String::from("SCREEN"), 16384);
    symbol_table.insert(String::from("KBD"), 24576);
    symbol_table.insert(String::from("SP"), 0);
    symbol_table.insert(String::from("LCL"), 1);
    symbol_table.insert(String::from("ARG"), 2);
    symbol_table.insert(String::from("THIS"), 3);
    symbol_table.insert(String::from("THAT"), 4);

    const R: &str = "R";

    for n in 0..16 {
        let mut current_key = String::from(R);
        current_key.push_str(&n.to_string());
        symbol_table.insert(current_key, n);
    }

    let mut c_instruction_comp_table: HashMap<&str, &str> = HashMap::new();
    c_instruction_comp_table.insert("0", "0101010");
    c_instruction_comp_table.insert("1", "0111111");
    c_instruction_comp_table.insert("-1", "0111010");
    c_instruction_comp_table.insert("D", "0001100");
    c_instruction_comp_table.insert("A", "0110000");
    c_instruction_comp_table.insert("M", "1110000");
    c_instruction_comp_table.insert("!D", "0001101");
    c_instruction_comp_table.insert("!A", "0110001");
    c_instruction_comp_table.insert("!M", "1110001");
    c_instruction_comp_table.insert("-D", "0001111");
    c_instruction_comp_table.insert("-A", "0110011");
    c_instruction_comp_table.insert("-M", "1110011");
    c_instruction_comp_table.insert("D+1", "0011111");
    c_instruction_comp_table.insert("1+D", "0011111");
    c_instruction_comp_table.insert("A+1", "0110111");
    c_instruction_comp_table.insert("1+A", "0110111");
    c_instruction_comp_table.insert("M+1", "1110111");
    c_instruction_comp_table.insert("1+M", "1110111");
    c_instruction_comp_table.insert("D-1", "0001110");
    c_instruction_comp_table.insert("A-1", "0110010");
    c_instruction_comp_table.insert("M-1", "1110010");
    c_instruction_comp_table.insert("D+A", "0000010");
    c_instruction_comp_table.insert("A+D", "0000010");
    c_instruction_comp_table.insert("D+M", "1000010");
    c_instruction_comp_table.insert("M+D", "1000010");
    c_instruction_comp_table.insert("D-A", "0010011");
    c_instruction_comp_table.insert("D-M", "1010011");
    c_instruction_comp_table.insert("A-D", "0000111");
    c_instruction_comp_table.insert("M-D", "1000111");
    c_instruction_comp_table.insert("D&A", "0000000");
    c_instruction_comp_table.insert("A&D", "0000000");
    c_instruction_comp_table.insert("D&M", "1000000");
    c_instruction_comp_table.insert("M&D", "1000000");
    c_instruction_comp_table.insert("D|A", "0010101");
    c_instruction_comp_table.insert("A|D", "0010101");
    c_instruction_comp_table.insert("D|M", "1010101");
    c_instruction_comp_table.insert("M|D", "1010101");

    let mut c_instruction_dest_table: HashMap<&str, &str> = HashMap::new();
    c_instruction_dest_table.insert("null", "000");
    c_instruction_dest_table.insert("M", "001");
    c_instruction_dest_table.insert("D", "010");
    c_instruction_dest_table.insert("DM", "011");
    c_instruction_dest_table.insert("MD", "011");
    c_instruction_dest_table.insert("A", "100");
    c_instruction_dest_table.insert("AM", "101");
    c_instruction_dest_table.insert("MA", "101");
    c_instruction_dest_table.insert("AD", "110");
    c_instruction_dest_table.insert("DA", "110");
    c_instruction_dest_table.insert("ADM", "111");
    c_instruction_dest_table.insert("AMD", "111");
    c_instruction_dest_table.insert("DAM", "111");
    c_instruction_dest_table.insert("DMA", "111");
    c_instruction_dest_table.insert("MAD", "111");
    c_instruction_dest_table.insert("MDA", "111");

    let mut c_instruction_jump_table: HashMap<&str, &str> = HashMap::new();
    c_instruction_jump_table.insert("null", "000");
    c_instruction_jump_table.insert("JGT", "001");
    c_instruction_jump_table.insert("JEQ", "010");
    c_instruction_jump_table.insert("JGE", "011");
    c_instruction_jump_table.insert("JLT", "100");
    c_instruction_jump_table.insert("JNE", "101");
    c_instruction_jump_table.insert("JLE", "110");
    c_instruction_jump_table.insert("JMP", "111");

    //dbg!(&symbol_table);

    let parsed_instructions = asm_parser.parse_instructions(
        asm_parser.clean_instructions(),
        &mut symbol_table,
        &c_instruction_comp_table,
        &c_instruction_dest_table,
        &c_instruction_jump_table,
    );

    let output_hack_file = File::create(file_path.with_extension("hack"))?;
    let mut output_hack_file = LineWriter::new(output_hack_file);
    output_hack_file.write_all(parsed_instructions.as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod test_main {
    use super::*;

    #[test]
    fn asm_file_validation_little_args() {
        let too_little_arguments = vec!["test".to_string()];
        let result = check_valid_asm_file(&too_little_arguments);
        assert!(result.is_err());
        
    }

    #[test]
    fn asm_file_validation_many_args() {
        let too_many_arguments = vec!["test".to_string(),"test1".to_string(),"test2".to_string()];
        let result = check_valid_asm_file(&too_many_arguments);
        assert!(result.is_err());
    }

    #[test]
    fn asm_file_validation_bad_path() {
        let bad_path_argument = vec!["test".to_string(),"bad_path.exe".to_string()];
        let result = check_valid_asm_file(&bad_path_argument);
        assert!(result.is_err());
    }
}
