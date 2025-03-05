pub mod lib {

    use std::collections::HashMap;
    #[derive(PartialEq, Debug)]
    pub enum InstructionType {
        A,
        C,
        L, // labels
    }

    pub enum CInstructionField {
        Dest,
        Comp,
        Jump,
    }

    pub struct AsmParser(String);

    impl AsmParser {
        pub fn new(asm_file_contents: String) -> AsmParser {
            AsmParser(asm_file_contents)
        }

        pub fn get_instruction_type(&self, instruction_line: &str) -> InstructionType {
            const A: &str = "@";
            const PARANTHESIS_START: &str = "(";
            if instruction_line.starts_with(PARANTHESIS_START) {
                InstructionType::L
            } else if instruction_line.starts_with(A) {
                InstructionType::A
            } else {
                InstructionType::C
            }
        }

        pub fn clean_instructions(&self) -> String {
            let mut cleaned_instructions = String::from("");
            const COMMENTS: &str = "//";
            for current_line in self.0.lines() {
                let line = current_line.trim();
                if line.is_empty() || line.starts_with(COMMENTS) {
                    continue;
                } else if let Some(current_instruction) = line.find(COMMENTS) {
                    let instruction_before_comment = line[..current_instruction].trim();
                    if !instruction_before_comment.is_empty() {
                        cleaned_instructions.push_str(instruction_before_comment);
                    }
                } else {
                    cleaned_instructions.push_str(line);
                }

                cleaned_instructions.push('\n');
            }

            // remove last \n
            cleaned_instructions.pop();
            //dbg!(&cleaned_instructions);
            cleaned_instructions
        }

        pub fn get_symbol<'a>(&self, instruction_line: &'a str, pattern: &[char]) -> &'a str {
            instruction_line.trim_matches(pattern)
        }

        pub fn get_c_instruction_field<'a>(
            &self,
            instruction_line: &'a str,
            field_type: CInstructionField,
        ) -> Option<&'a str> {
            match field_type {
                CInstructionField::Dest => {
                    if instruction_line.contains("=") {
                        instruction_line.split("=").nth(0)
                    } else {
                        None
                    }
                }
                CInstructionField::Comp => {
                    let mut comp = instruction_line;
                    if comp.contains("=") {
                        comp = instruction_line
                            .split("=")
                            .nth(1)
                            .expect("Can't be None due to existence of '='");
                    }

                    if comp.contains(";") {
                        comp = comp
                            .split(";")
                            .nth(0)
                            .expect("Can't be None due to existence of ';'");
                    }

                    Some(comp)
                }
                CInstructionField::Jump => {
                    if instruction_line.contains(";") {
                        instruction_line.split(";").nth(1)
                    } else {
                        None
                    }
                }
            }
        }

        pub fn parse_instructions(
            &self,
            clean_instructions: String,
            symbol_table: &mut HashMap<String, i16>,
            c_instruction_comp_table: &HashMap<&str, &str>,
            c_instruction_dest_table: &HashMap<&str, &str>,
            c_instruction_jump_table: &HashMap<&str, &str>,
        ) -> String {
            let mut parsed_instructions = String::from("");
            // first pass, resolve labels and remove label lines from program
            let mut line_number: i16 = 0;
            let lines = clean_instructions
                .lines()
                .filter(|line| match self.get_instruction_type(line) {
                    InstructionType::L => {
                        symbol_table
                            .insert(self.get_symbol(line, &['(', ')']).to_string(), line_number);
                        false
                    }
                    InstructionType::A | InstructionType::C => {
                        line_number += 1;
                        true
                    }
                })
                .collect::<Vec<&str>>();

            let mut allocated_address: i16 = 16;
            // second pass
            for current_line in lines {
                if !parsed_instructions.trim().is_empty() {
                    parsed_instructions.push('\n');
                }

                if self.get_instruction_type(current_line) == InstructionType::A {
                    // check if parsable to a number, if not its a symbol (could be label variable or normal variable)
                    let maybe_symbol = self.get_symbol(current_line, &['@']);
                    if let Ok(parsable_number) = maybe_symbol.parse::<i16>() {
                        parsed_instructions.push_str(&format!("{:016b}", parsable_number));
                    } else if !symbol_table.contains_key(maybe_symbol) {
                        symbol_table.insert(maybe_symbol.to_string(), allocated_address);
                        parsed_instructions.push_str(&format!("{:016b}", allocated_address));
                        allocated_address += 1;
                    } else {
                        // most likely pre-defined symbols
                        parsed_instructions.push_str(&format!(
                            "{:016b}",
                            symbol_table
                                .get(maybe_symbol)
                                .expect("Did not intialize in main function")
                        ));
                    }
                } else {
                    let mut c_instruction_binary = String::from("111");
                    if let Some(comp_field) =
                        self.get_c_instruction_field(current_line, CInstructionField::Comp)
                    {
                        let comp_field_binary = c_instruction_comp_table
                            .get(comp_field)
                            .expect("Did not initialize in main function");
                        c_instruction_binary.push_str(comp_field_binary);
                    }

                    if let Some(dest_field) =
                        self.get_c_instruction_field(current_line, CInstructionField::Dest)
                    {
                        //dbg!(dest_field);
                        c_instruction_binary.push_str(
                            c_instruction_dest_table
                                .get(dest_field)
                                .expect("Did not initialize in main function"),
                        );
                    } else {
                        c_instruction_binary.push_str(
                            c_instruction_dest_table
                                .get("null")
                                .expect("Did not initialize in main function"),
                        );
                    }

                    if let Some(jump_field) =
                        self.get_c_instruction_field(current_line, CInstructionField::Jump)
                    {
                        c_instruction_binary.push_str(
                            c_instruction_jump_table
                                .get(jump_field)
                                .expect("Did not initialize in main function"),
                        );
                    } else {
                        c_instruction_binary.push_str(
                            c_instruction_jump_table
                                .get("null")
                                .expect("Did not initialize in main function"),
                        );
                    }

                    parsed_instructions.push_str(&c_instruction_binary);
                }
            }

            parsed_instructions
        }
    }
}
