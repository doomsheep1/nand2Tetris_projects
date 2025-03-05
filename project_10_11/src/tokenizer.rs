use crate::utils::{clean_jack_code, get_fixed_lexical_elements};
use std::{fs, path::PathBuf};

#[derive(Eq, Hash, PartialEq, Debug)]
pub enum TokenType {
    Keyword,
    Symbol,
    Identifier,
    IntConstant,
    StringConst,
    None,
}

pub struct JackTokenizer {
    file_content: String,
    current_token: String,
    current_token_type: TokenType,
}

impl JackTokenizer {
    pub fn new(jack_file: &PathBuf) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(jack_file)?;
        let cleaned_jack_code = clean_jack_code(contents);
        Ok(Self {
            file_content: cleaned_jack_code,
            current_token: String::from(""),
            current_token_type: TokenType::None,
        })
    }

    pub fn has_more_tokens(&self) -> bool {
        !self.file_content.is_empty()
    }

    pub fn get_token(&self) -> &str {
        &self.current_token
    }

    pub fn get_token_type(&self) -> &TokenType {
        &self.current_token_type
    }

    pub fn get_file_content(&self) -> &str {
        &self.file_content
    }

    fn set_token_details(&mut self, current_token: &str, trim_content: &str) {
        self.current_token.clear();
        self.current_token.push_str(current_token);
        if current_token.len() == 1 {
            self.file_content = self
                .file_content
                .strip_prefix(trim_content)
                .expect("Impossible to be none as it was checked by caller")
                .into();
        } else {
            self.file_content = self.file_content.trim_start_matches(trim_content).into();
        }
    }

    pub fn advance(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let fixed_lexical_elements = get_fixed_lexical_elements();
        if self.has_more_tokens() {
            let keywords = fixed_lexical_elements
                .get(&TokenType::Keyword)
                .expect("Should have been initialized");
            let symbols = fixed_lexical_elements
                .get(&TokenType::Symbol)
                .expect("Should have been initialized");
            let mut working_token = String::from("");
            self.file_content = self.file_content.trim_start().into();
            if keywords.iter().any(|keyword| {
                if self.file_content.starts_with(&format!("{keyword} "))
                    || self.file_content.starts_with(&format!("{keyword};"))
                    || self.file_content.starts_with(&format!("{keyword})"))
                {
                    self.set_token_details(keyword, keyword);
                    true
                } else {
                    false
                }
            }) {
                self.current_token_type = TokenType::Keyword;
            } else if symbols.iter().any(|symbol| {
                if self.file_content.starts_with(symbol) {
                    self.set_token_details(symbol, symbol);
                    true
                } else {
                    false
                }
            }) {
                self.current_token_type = TokenType::Symbol;
            } else if self
                .file_content
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_digit())
            {
                let file_char_iterator = self.file_content.chars();
                for number in file_char_iterator {
                    if number.is_ascii_digit() {
                        working_token.push(number);
                    } else {
                        break;
                    }
                }

                self.set_token_details(&working_token, &working_token);
                self.current_token_type = TokenType::IntConstant;
            } else if self.file_content.starts_with('"') {
                if let Some(end_dbl_quote) = &self.file_content[1..].find('"') {
                    let token = &self.file_content[1..end_dbl_quote + 1].to_string();
                    self.set_token_details(token, &format!("\"{token}\""));
                    self.current_token_type = TokenType::StringConst;
                } else {
                    Err("Parse failed, invalid String constant token".to_string())?
                }
            } else if self
                .file_content
                .chars()
                .next()
                .is_some_and(|character| character.is_alphabetic() || character == '_')
            {
                let file_char_iterator = self.file_content.chars();
                for character in file_char_iterator {
                    if character.is_alphabetic() || character == '_' || character.is_ascii_digit() {
                        working_token.push(character);
                    } else {
                        break;
                    }
                }
                self.set_token_details(&working_token, &working_token);
                self.current_token_type = TokenType::Identifier;
            } else {
                Err("Parse failed, token did not match any of the defined token types".to_string())?
            }
        } else {
            Err("No more tokens available".to_string())?;
        }

        Ok(())
    }
}

mod tests {
    use super::{JackTokenizer, TokenType};

    /*
       This test case contains all lexical elements Keyword,Identifier,Symbol,IntConstant and StringConstant
    */
    #[test]
    fn validate_tokenization() {
        let mut dummy_tokenizer = JackTokenizer {
            file_content: String::from(""),
            current_token: String::from(""),
            current_token_type: TokenType::None,
        };
        dummy_tokenizer.file_content = "function void main(){var SquareGame game;let game2 = 2;do game.run();do game.dispose();let game3 = \"hello\";return;}".to_string();
        let mut correct_tokens = Vec::new();
        correct_tokens.push(("function", TokenType::Keyword));
        correct_tokens.push(("void", TokenType::Keyword));
        correct_tokens.push(("main", TokenType::Identifier));
        correct_tokens.push(("(", TokenType::Symbol));
        correct_tokens.push((")", TokenType::Symbol));
        correct_tokens.push(("{", TokenType::Symbol));
        correct_tokens.push(("var", TokenType::Keyword));
        correct_tokens.push(("SquareGame", TokenType::Identifier));
        correct_tokens.push(("game", TokenType::Identifier));
        correct_tokens.push((";", TokenType::Symbol));
        correct_tokens.push(("let", TokenType::Keyword));
        correct_tokens.push(("game2", TokenType::Identifier));
        correct_tokens.push(("=", TokenType::Symbol));
        correct_tokens.push(("2", TokenType::IntConstant));
        correct_tokens.push((";", TokenType::Symbol));
        correct_tokens.push(("do", TokenType::Keyword));
        correct_tokens.push(("game", TokenType::Identifier));
        correct_tokens.push((".", TokenType::Symbol));
        correct_tokens.push(("run", TokenType::Identifier));
        correct_tokens.push(("(", TokenType::Symbol));
        correct_tokens.push((")", TokenType::Symbol));
        correct_tokens.push((";", TokenType::Symbol));
        correct_tokens.push(("do", TokenType::Keyword));
        correct_tokens.push(("game", TokenType::Identifier));
        correct_tokens.push((".", TokenType::Symbol));
        correct_tokens.push(("dispose", TokenType::Identifier));
        correct_tokens.push(("(", TokenType::Symbol));
        correct_tokens.push((")", TokenType::Symbol));
        correct_tokens.push((";", TokenType::Symbol));
        correct_tokens.push(("let", TokenType::Keyword));
        correct_tokens.push(("game3", TokenType::Identifier));
        correct_tokens.push(("=", TokenType::Symbol));
        correct_tokens.push(("hello", TokenType::StringConst));
        correct_tokens.push((";", TokenType::Symbol));
        correct_tokens.push(("return", TokenType::Keyword));
        correct_tokens.push((";", TokenType::Symbol));
        correct_tokens.push(("}", TokenType::Symbol));
        let mut count = 0;
        while !dummy_tokenizer.file_content.is_empty() {
            if dummy_tokenizer.advance().is_ok() {
                let (correct_token, correct_token_type) = &correct_tokens[count];
                let token = dummy_tokenizer.get_token();
                let token_type = dummy_tokenizer.get_token_type();
                assert_eq!(correct_token.to_string(), token.to_string());
                assert_eq!(correct_token_type, token_type);
                count += 1;
            }
        }
    }
}
