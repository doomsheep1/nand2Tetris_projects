use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{LineWriter, Write};
use std::path::PathBuf;
use std::{env, path::Path};
use vm_translator::{VMCommandType, VmCodeParser, VmCodeWriter};

// nand2tetris project 7 and 8 vm_translator source code
// usage:
// pass in the path of a *.vm file as an argument e.g. ./vm_translator myVMFile.vm or
// pass in a directory containing 1 or more *.vm files as an argument e.g. ./vm_translator myVMDirectory
// it will output a myVmFile.asm file or myVMDirectory.asm
// use this for project 7 and 8 requirements

fn get_valid_vm_files<P: AsRef<Path>>(file_path: P) -> Vec<PathBuf> {
    let mut paths_vec: Vec<PathBuf> = Vec::new();
    let file_path = file_path.as_ref();

    if file_path.is_file() && file_path.extension().is_some_and(|ext| ext == "vm") {
        paths_vec.push(file_path.to_path_buf());
    } else if file_path.is_dir() {
        if let Ok(file_dir) = file_path.read_dir() {
            for file_entry in file_dir.flatten() {
                let valid_file_path = file_entry.path();
                if valid_file_path.is_file()
                    && valid_file_path.extension().is_some_and(|ext| ext == "vm")
                {
                    paths_vec.push(valid_file_path);
                } else if valid_file_path.is_dir() {
                    paths_vec.extend(get_valid_vm_files(valid_file_path));
                }
            }
        }
    }

    paths_vec

    //Err("Please enter a file path that is of *.vm or a directory containing 1 or more *.vm files to the program.".to_string())?
}

fn check_valid_vm_files(args: &[String]) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    // validate there was an argument passed
    if args.len() != 2 {
        Err("Please enter a file path as an argument to the program.".to_string())?
    }

    // validate to see whether there are vm files
    let mut vm_files_vec = get_valid_vm_files(Path::new(&args[1]));
    if vm_files_vec.is_empty() {
        Err("Please ensure the file path entered has files of extension type *.vm".to_string())?
    } else if let Some(sys_vm_index) = vm_files_vec.iter().position(|x| x.ends_with("Sys.vm")) {
        // reorder vec so that sys is always first if applicable
        let sys_vm_file = vm_files_vec.remove(sys_vm_index);
        vm_files_vec.insert(0, sys_vm_file);
    }

    Ok(vm_files_vec)
}

fn get_command_symbol_table() -> HashMap<VMCommandType, Vec<&'static str>> {
    let mut command_symbol_table: HashMap<VMCommandType, Vec<&str>> = HashMap::new();
    command_symbol_table.insert(
        VMCommandType::Carithmetic,
        vec!["add", "sub", "neg", "eq", "gt", "lt", "and", "or", "not"],
    );
    command_symbol_table.insert(
        VMCommandType::Cpush,
        vec![
            "constant", "local", "argument", "this", "that", "static", "temp", "pointer",
        ],
    );
    command_symbol_table.insert(
        VMCommandType::Cpop,
        vec![
            "local", "argument", "this", "that", "static", "temp", "pointer",
        ],
    );
    command_symbol_table
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let vm_files_vec = check_valid_vm_files(&args)?;
    let command_symbol_table = get_command_symbol_table();
    let asm_file_path = Path::new(&args[1]);
    let output_asm_file = File::create(asm_file_path.with_extension("asm"))?;
    let mut output_asm_file = LineWriter::new(output_asm_file);

    // to track function call sequence
    let mut function_call_stack: Vec<String> = Vec::new();
    let mut bootstrap_code_exists = false;

    for vm_file in vm_files_vec {
        let vm_file_name_no_extension = vm_file
            .as_path()
            .file_stem()
            .expect("Should be valid")
            .to_str()
            .expect("Should be valid");
        let contents = fs::read_to_string(&vm_file)?;
        let vm_code_parser = VmCodeParser::new();
        let cleaned_contents = vm_code_parser.clean_vm_code(contents);
        let vm_code_writer = VmCodeWriter::new(vm_code_parser, cleaned_contents);
        if vm_file_name_no_extension == "Sys" {
            // bootstrap code required
            let init_code = String::from("call Sys.init 0");
            let init_code_parser = VmCodeParser::new();
            let cleaned_init_code = init_code_parser.clean_vm_code(init_code); // this is useless...but to stay consistent
            let init_code_writer = VmCodeWriter::new(init_code_parser, cleaned_init_code);
            let init_vm_code = init_code_writer.write_init();
            output_asm_file.write_all(init_vm_code.as_bytes())?;
            let translated_vm_code = init_code_writer.translate(
                &command_symbol_table,
                asm_file_path
                    .file_stem()
                    .expect("Should be valid")
                    .to_str()
                    .expect("Should be valid"),
                &mut function_call_stack,
            )?;
            output_asm_file.write_all(translated_vm_code.as_bytes())?;
            bootstrap_code_exists = true;
        }
        let translated_vm_code: String = vm_code_writer.translate(
            &command_symbol_table,
            vm_file_name_no_extension,
            &mut function_call_stack,
        )?;
        output_asm_file.write_all(translated_vm_code.as_bytes())?;
    }

    if !bootstrap_code_exists {
        // set end of file
        output_asm_file.write_all("(end_asm_file)\n@end_asm_file\n0;JMP".as_bytes())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vm_file_validation_little_args() {
        let too_little_arguments = vec!["test".to_string()];
        let result = check_valid_vm_files(&too_little_arguments);
        assert!(result.is_err());
    }

    #[test]
    fn vm_file_validation_many_args() {
        let too_many_arguments = vec!["test".to_string(), "test1".to_string(), "test2".to_string()];
        let result = check_valid_vm_files(&too_many_arguments);
        assert!(result.is_err());
    }

    #[test]
    fn vm_file_validation_bad_path() {
        let bad_path_argument = vec!["test".to_string(), "bad_path.exe".to_string()];
        let result = check_valid_vm_files(&bad_path_argument);
        assert!(result.is_err());
    }
}
