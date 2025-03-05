use crate::compile::CompilationEngine;
use std::{
    env,
    error::Error,
    path::{Path, PathBuf},
};

mod compile;
mod symbol;
mod tokenizer;
mod utils;
mod writer;

fn get_valid_jack_files<P: AsRef<Path>>(file_path: P) -> Vec<PathBuf> {
    let mut paths_vec: Vec<PathBuf> = Vec::new();
    let file_path = file_path.as_ref();

    if file_path.is_file() && file_path.extension().is_some_and(|ext| ext == "jack") {
        paths_vec.push(file_path.to_path_buf());
    } else if file_path.is_dir() {
        if let Ok(file_dir) = file_path.read_dir() {
            for file_entry in file_dir.flatten() {
                let valid_file_path = file_entry.path();
                if valid_file_path.is_file()
                    && valid_file_path.extension().is_some_and(|ext| ext == "jack")
                {
                    paths_vec.push(valid_file_path);
                } else if valid_file_path.is_dir() {
                    paths_vec.extend(get_valid_jack_files(valid_file_path));
                }
            }
        }
    }

    paths_vec
}

fn check_valid_jack_files(args: &[String]) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    // validate there was an argument passed
    if args.len() != 2 {
        Err("Please enter a file path as an argument to the program.".to_string())?
    }

    // validate to see whether there are vm files
    let jack_files_vec = get_valid_jack_files(Path::new(&args[1]));
    if jack_files_vec.is_empty() {
        Err("Please ensure the file path entered has files of extension type *.jack".to_string())?
    }

    Ok(jack_files_vec)
}

// meant to be the jack_analyzer
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let jack_files_vec = check_valid_jack_files(&args)?;
    for jack_file in jack_files_vec {
        if let Ok(mut compilation_engine) = CompilationEngine::new(&jack_file) {
            compilation_engine.compile_class()?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_jack_files_invalid_args() {
        let too_little_arguments = vec!["test".to_string()];
        let too_many_arguments = vec!["test".to_string(), "test2".to_string(), "test3".to_string()];
        let bad_file_path = vec!["test".to_string(), "something.jak".to_string()];
        let missing_single_jack_file = vec!["test".to_string(), "hello.jack".to_string()];
        let missing_multi_jack_files = vec!["test".to_string(), "./hello".to_string()]; //directory hello
        let result_too_little = check_valid_jack_files(&too_little_arguments);
        let result_too_many = check_valid_jack_files(&too_many_arguments);
        let result_bad_path = check_valid_jack_files(&bad_file_path);
        let result_missing_single_jack_file = check_valid_jack_files(&missing_single_jack_file);
        let result_missing_multi_jack_files = check_valid_jack_files(&missing_multi_jack_files);
        assert!(result_too_little.is_err());
        assert!(result_too_many.is_err());
        assert!(result_bad_path.is_err());
        assert!(result_missing_single_jack_file.is_err());
        assert!(result_missing_multi_jack_files.is_err());
    }
}
