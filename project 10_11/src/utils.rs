use std::collections::HashMap;

use crate::tokenizer::TokenType;

pub fn clean_jack_code(contents: String) -> String {
    let mut found_multi_line_comment = false;
    let mut cleaned_jack_code = String::from("");
    contents.lines().for_each(|current_line| {
        let line = current_line.trim();
        if line.starts_with("/*") {
            found_multi_line_comment = true;
        } else if !line.is_empty() && !line.starts_with("//") && !found_multi_line_comment {
            if let Some(current_jack_line) = line.find("//") {
                let jack_before_comment = line[..current_jack_line].trim();
                if !jack_before_comment.is_empty() {
                    cleaned_jack_code.push_str(jack_before_comment);
                }
            } else {
                cleaned_jack_code.push_str(line);
            }
        }

        if found_multi_line_comment && line.contains("*/") {
            found_multi_line_comment = false;
        }
    });

    cleaned_jack_code
}

pub fn get_fixed_lexical_elements() -> HashMap<TokenType, Vec<&'static str>> {
    let mut lexical_elements: HashMap<TokenType, Vec<&str>> = HashMap::new();
    lexical_elements.insert(
        TokenType::Keyword,
        vec![
            "class",
            "constructor",
            "function",
            "method",
            "field",
            "static",
            "var",
            "int",
            "char",
            "boolean",
            "void",
            "true",
            "false",
            "null",
            "this",
            "let",
            "do",
            "if",
            "else",
            "while",
            "return",
        ],
    );
    lexical_elements.insert(
        TokenType::Symbol,
        vec![
            "{", "}", "(", ")", "[", "]", ".", ",", ";", "+", "-", "*", "/", "&", "|", "<", ">",
            "=", "~",
        ],
    );
    lexical_elements
}

#[cfg(test)]
mod tests {
    use super::clean_jack_code;

    #[test]
    fn validate_cleaned_jack_code() {
        let uncleaned_file_content = "// This file is part of www.nand2tetris.org\n
// and the book \"The Elements of Computing Systems\"\n
// by Nisan and Schocken, MIT Press.\n
// File name: projects/10/Square/SquareGame.jack\n
\n
// (same as projects/9/Square/SquareGame.jack)\n
/**\n
 * Implements the Square game.\n
 * This simple game allows the user to move a black square around\n
 * the screen, and change the square's size during the movement.\n
 * When the game starts, a square of 30 by 30 pixels is shown at the\n
 * top-left corner of the screen. The user controls the square as follows.\n
 * The 4 arrow keys are used to move the square up, down, left, and right.\n
 * The 'z' and 'x' keys are used, respectively, to decrement and increment\n
 * the square's size. The 'q' key is used to quit the game.\n
 */\n
class SquareGame {\n
   field Square square; // the square of this game\n
   field int direction; // the square's current direction:\n
                        // 0=none, 1=up, 2=down, 3=left, 4=right\n
\n
   /** Constructs a new Square Game. */\n
   constructor SquareGame new() {\n
      // Creates a 30 by 30 pixels square and positions it at the top-left\n
      // of the screen.\n
      let square = Square.new(0, 0, 30);\n
      let direction = 0;  // initial state is no movement\n
      return this;\n
   }\n
\n
   /** Disposes this game. */\n
   method void dispose() {\n
      do square.dispose();\n
      do Memory.deAlloc(this);\n
      return;\n
   }\n
\n
   /** Moves the square in the current direction. */\n
   method void moveSquare() {\n
      if (direction = 1) { do square.moveUp(); }\n
      if (direction = 2) { do square.moveDown(); }\n
      if (direction = 3) { do square.moveLeft(); }\n
      if (direction = 4) { do square.moveRight(); }\n
      do Sys.wait(5);  // delays the next movement\n
      return;\n
   }\n
\n
   /** Runs the game: handles the user's inputs and moves the square accordingly */\n
   method void run() {\n
      var char key;  // the key currently pressed by the user\n
      var boolean exit;\n
      let exit = false;\n
\n
      while (~exit) {\n
         // waits for a key to be pressed\n
         while (key = 0) {\n
            let key = Keyboard.keyPressed();\n
            do moveSquare();\n
         }\n
         if (key = 81)  { let exit = true; }     // q key\n
         if (key = 90)  { do square.decSize(); } // z key\n
         if (key = 88)  { do square.incSize(); } // x key\n
         if (key = 131) { let direction = 1; }   // up arrow\n
         if (key = 133) { let direction = 2; }   // down arrow\n
         if (key = 130) { let direction = 3; }   // left arrow\n
         if (key = 132) { let direction = 4; }   // right arrow\n
\n
         // waits for the key to be released\n
         while (~(key = 0)) {\n
            let key = Keyboard.keyPressed();\n
            do moveSquare();\n
         }\n
     } // while\n
     return;\n
   }\n
}\n
\n
\n
\n
"
        .to_string();
        let correct_cleaned_jack_code = "class SquareGame {field Square square;field int direction;constructor SquareGame new() {let square = Square.new(0, 0, 30);let direction = 0;return this;}method void dispose() {do square.dispose();do Memory.deAlloc(this);return;}method void moveSquare() {if (direction = 1) { do square.moveUp(); }if (direction = 2) { do square.moveDown(); }if (direction = 3) { do square.moveLeft(); }if (direction = 4) { do square.moveRight(); }do Sys.wait(5);return;}method void run() {var char key;var boolean exit;let exit = false;while (~exit) {while (key = 0) {let key = Keyboard.keyPressed();do moveSquare();}if (key = 81)  { let exit = true; }if (key = 90)  { do square.decSize(); }if (key = 88)  { do square.incSize(); }if (key = 131) { let direction = 1; }if (key = 133) { let direction = 2; }if (key = 130) { let direction = 3; }if (key = 132) { let direction = 4; }while (~(key = 0)) {let key = Keyboard.keyPressed();do moveSquare();}}return;}}".to_string();
        let cleaned_jack_code = clean_jack_code(uncleaned_file_content);
        assert_eq!(correct_cleaned_jack_code, cleaned_jack_code);
    }
}
