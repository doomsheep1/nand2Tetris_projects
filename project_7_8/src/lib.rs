use std::{collections::HashMap, error::Error};

#[derive(Eq, Hash, PartialEq)]
pub enum VMCommandType {
    Carithmetic,
    Cpush,
    Cpop,
    Clabel,
    Cgoto,
    Cif,
    Cfunction,
    Creturn,
    Ccall,
}

pub struct VmCodeParser;

impl Default for VmCodeParser {
    fn default() -> Self {
        Self::new()
    }
}

impl VmCodeParser {
    pub fn new() -> VmCodeParser {
        VmCodeParser
    }

    pub fn clean_vm_code(&self, vm_code: String) -> String {
        let mut cleaned_vm_code = String::from("");
        const COMMENTS: &str = "//";
        for current_line in vm_code.lines() {
            let line = current_line.trim();
            if line.is_empty() || line.starts_with(COMMENTS) {
                continue;
            } else if let Some(current_vm_code_line) = line.find(COMMENTS) {
                let vm_code_before_comment = line[..current_vm_code_line].trim();
                if !vm_code_before_comment.is_empty() {
                    cleaned_vm_code.push_str(vm_code_before_comment);
                }
            } else {
                cleaned_vm_code.push_str(line);
            }

            cleaned_vm_code.push('\n');
        }

        // remove last \n
        cleaned_vm_code.pop();
        //dbg!(&cleaned_instructions);
        cleaned_vm_code
    }

    fn command_type(
        &self,
        current_command: &str,
        command_table: &HashMap<VMCommandType, Vec<&str>>,
    ) -> Option<VMCommandType> {
        if current_command.starts_with("push") {
            let push_command_vec: &Vec<&str> = command_table
                .get(&VMCommandType::Cpush)
                .expect("Did not initialize in function");
            if push_command_vec.iter().any(|command| {
                self.arg1(current_command, &VMCommandType::Cpush)
                    .is_some_and(|value| value == *command)
            }) {
                Some(VMCommandType::Cpush)
            } else {
                None
            }
        } else if current_command.starts_with("pop") {
            let pop_command_vec: &Vec<&str> = command_table
                .get(&VMCommandType::Cpop)
                .expect("Did not initialize in function");
            if pop_command_vec.iter().any(|command| {
                self.arg1(current_command, &VMCommandType::Cpop)
                    .is_some_and(|value| value == *command)
            }) {
                Some(VMCommandType::Cpop)
            } else {
                None
            }
        } else if current_command.starts_with("label") {
            Some(VMCommandType::Clabel)
        } else if current_command.starts_with("goto") {
            Some(VMCommandType::Cgoto)
        } else if current_command.starts_with("if-goto") {
            Some(VMCommandType::Cif)
        } else if current_command.starts_with("call") {
            Some(VMCommandType::Ccall)
        } else if current_command.starts_with("function") {
            Some(VMCommandType::Cfunction)
        } else if current_command == "return" {
            Some(VMCommandType::Creturn)
        } else {
            let arithmetic_command_vec: &Vec<&str> = command_table
                .get(&VMCommandType::Carithmetic)
                .expect("Did not initialize in function");
            if arithmetic_command_vec
                .iter()
                .any(|command| current_command == *command)
            {
                Some(VMCommandType::Carithmetic)
            } else {
                None
            }
        }
    }

    fn arg1<'a>(&self, current_command: &'a str, command_type: &VMCommandType) -> Option<&'a str> {
        match command_type {
            VMCommandType::Carithmetic | VMCommandType::Creturn => None,
            VMCommandType::Cpush
            | VMCommandType::Cpop
            | VMCommandType::Clabel
            | VMCommandType::Cgoto
            | VMCommandType::Cif
            | VMCommandType::Cfunction
            | VMCommandType::Ccall => current_command.split(" ").nth(1),
        }
    }

    fn arg2<'a>(&self, current_command: &'a str, command_type: &VMCommandType) -> Option<&'a str> {
        match command_type {
            VMCommandType::Carithmetic
            | VMCommandType::Creturn
            | VMCommandType::Clabel
            | VMCommandType::Cgoto
            | VMCommandType::Cif => None,
            VMCommandType::Cpush
            | VMCommandType::Cpop
            | VMCommandType::Ccall
            | VMCommandType::Cfunction => current_command
                .split(" ")
                .nth(2)
                .filter(|&maybe_index| maybe_index.parse::<i16>().is_ok()),
        }
    }
}

pub struct VmCodeWriter {
    code_parser: VmCodeParser,
    cleaned_vm_commands: String,
}

impl VmCodeWriter {
    pub fn new(code_parser: VmCodeParser, cleaned_vm_commands: String) -> VmCodeWriter {
        VmCodeWriter {
            code_parser,
            cleaned_vm_commands,
        }
    }

    pub fn translate(
        &self,
        command_table: &HashMap<VMCommandType, Vec<&str>>,
        file_name: &str,
        function_call_stack: &mut Vec<String>,
    ) -> Result<String, Box<dyn Error>> {
        let mut translated_vm_code = String::from("");
        let mut line_number: i16 = 0;
        for current_command in self.cleaned_vm_commands.lines() {
            if let Some(command_type) = self
                .code_parser
                .command_type(current_command, command_table)
            {
                let segment_list = command_table.get(&command_type);
                match command_type {
                    VMCommandType::Carithmetic => {
                        // arg functions kinda useless as it just returns itself
                        if let Some(translated_command) = self.write_arithmetic(
                            current_command,
                            segment_list.expect("Did not intialize in symbol table"),
                            &line_number,
                        ) {
                            translated_vm_code.push_str(&translated_command);
                            translated_vm_code.push('\n');
                        } else {
                            Err(format!(
                                "Command translation failed for current command: {current_command}"
                            ))?
                        }
                    }
                    VMCommandType::Cpush | VMCommandType::Cpop => {
                        let segment = self.code_parser.arg1(current_command, &command_type);
                        let index = self.code_parser.arg2(current_command, &command_type);
                        if let (Some(segment_value), Some(index_value)) = (segment, index) {
                            if command_type == VMCommandType::Cpush {
                                if let Some(translated_command) = self.write_push(
                                    segment_value,
                                    index_value,
                                    segment_list.expect("Did not initialize in symbol table"),
                                    file_name,
                                ) {
                                    translated_vm_code.push_str(&translated_command);
                                    translated_vm_code.push('\n');
                                } else {
                                    Err(format!("Command translation failed for current command: {current_command}"))?
                                }
                            } else if command_type == VMCommandType::Cpop {
                                if let Some(translated_command) = self.write_pop(
                                    segment_value,
                                    index_value,
                                    segment_list.expect("Did not initialize in symbol table"),
                                    file_name,
                                ) {
                                    translated_vm_code.push_str(&translated_command);
                                    translated_vm_code.push('\n');
                                } else {
                                    Err(format!("Command translation failed for current command: {current_command}"))?
                                }
                            }
                        } else {
                            Err(format!(
                                "Command arguments are invalid, please check: {:?} {:?}",
                                segment, index
                            ))?
                        }
                    }
                    VMCommandType::Clabel => {
                        let function_context;
                        if let Some(previous_function) = function_call_stack.last() {
                            function_context = previous_function.to_string();
                        } else {
                            // this scenario most likely happens when calling sys.init
                            function_context = String::new();
                        }

                        if let Some(translated_command) =
                            self.write_label(current_command, &function_context)
                        {
                            translated_vm_code.push_str(&translated_command);
                            translated_vm_code.push('\n');
                        } else {
                            Err(format!(
                                "Command translation failed for current command: {current_command}"
                            ))?
                        }
                    }
                    VMCommandType::Cgoto => {
                        let function_context;
                        if let Some(previous_function) = function_call_stack.last() {
                            function_context = previous_function.to_string();
                        } else {
                            // this scenario most likely happens when calling sys.init
                            function_context = String::new();
                        }

                        if let Some(translated_command) =
                            self.write_goto(current_command, &function_context)
                        {
                            translated_vm_code.push_str(&translated_command);
                            translated_vm_code.push('\n');
                        } else {
                            Err(format!(
                                "Command translation failed for current command: {current_command}"
                            ))?
                        }
                    }
                    VMCommandType::Cif => {
                        let function_context;
                        if let Some(previous_function) = function_call_stack.last() {
                            function_context = previous_function.to_string();
                        } else {
                            // this scenario most likely happens when calling sys.init
                            function_context = String::new();
                        }

                        if let Some(translated_command) =
                            self.write_if(current_command, &function_context)
                        {
                            translated_vm_code.push_str(&translated_command);
                            translated_vm_code.push('\n');
                        } else {
                            Err(format!(
                                "Command translation failed for current command: {current_command}"
                            ))?
                        }
                    }
                    VMCommandType::Ccall => {
                        let function_name = self
                            .code_parser
                            .arg1(current_command, &VMCommandType::Ccall);
                        let args = self
                            .code_parser
                            .arg2(current_command, &VMCommandType::Ccall);
                        if let (Some(function_name), Some(args)) = (function_name, args) {
                            let return_address;
                            if let Some(previous_function) = function_call_stack.last() {
                                dbg!(&function_call_stack);
                                let call_count = function_call_stack
                                    .iter()
                                    .filter(|&s| s == previous_function)
                                    .count();
                                return_address = format!("{previous_function}$ret.{call_count}");
                            } else {
                                // this scenario most likely happens when calling sys.init
                                return_address = format!("{file_name}.$ret");
                            }

                            function_call_stack.push(function_name.to_string());

                            let args: i16 = args
                                .parse()
                                .expect("Parsing to i16 should have been validated");
                            if let Some(translated_command) =
                                self.write_call(function_name, args, &return_address)
                            {
                                translated_vm_code.push_str(&translated_command);
                                translated_vm_code.push('\n');
                            } else {
                                Err(format!("Command translation failed for current command: {current_command}"))?
                            }
                        } else {
                            Err(format!(
                                "Command arguments are invalid, please check: {:?} {:?}",
                                function_name, args
                            ))?
                        }
                    }
                    VMCommandType::Cfunction => {
                        let function_name = self
                            .code_parser
                            .arg1(current_command, &VMCommandType::Cfunction);
                        let local_vars = self
                            .code_parser
                            .arg2(current_command, &VMCommandType::Cfunction);
                        if let (Some(function_name), Some(local_vars)) = (function_name, local_vars)
                        {
                            let local_vars: i16 = local_vars
                                .parse()
                                .expect("Parsing to i16 should have been validated");
                            if let Some(translated_command) =
                                self.write_function(function_name, local_vars)
                            {
                                translated_vm_code.push_str(&translated_command);
                                translated_vm_code.push('\n');
                            } else {
                                Err(format!("Command translation failed for current command: {current_command}"))?
                            }
                        } else {
                            Err(format!(
                                "Command arguments are invalid, please check: {:?} {:?}",
                                function_name, local_vars
                            ))?
                        }
                    }
                    VMCommandType::Creturn => {
                        // pop function stack
                        if let Some(translated_command) = self.write_return() {
                            translated_vm_code.push_str(&translated_command);
                            translated_vm_code.push('\n');
                        } else {
                            Err(format!(
                                "Command translation failed for current command: {current_command}"
                            ))?
                        }
                    }
                }

                line_number += 1;
            } else {
                Err(format!(
                    "Command is invalid, please check: {current_command}"
                ))?
            }
        }

        Ok(translated_vm_code)
    }

    pub fn write_init(&self) -> String {
        let mut translated_command = String::from("");
        // init stack pointer
        translated_command.push_str("@256\nD=A\n@SP\nM=D\n");

        translated_command
    }

    fn write_label(&self, label_command: &str, function_context: &str) -> Option<String> {
        let mut translated_command = String::from("");
        if let Some(label_name) = self.code_parser.arg1(label_command, &VMCommandType::Clabel) {
            if function_context.is_empty() {
                translated_command.push_str(&format!("({label_name})"));
            } else {
                translated_command.push_str(&format!("({function_context}${label_name})"));
            }

            Some(translated_command)
        } else {
            None
        }
    }

    fn write_goto(&self, goto_command: &str, function_context: &str) -> Option<String> {
        let mut translated_command = String::from("");
        if let Some(label_name) = self.code_parser.arg1(goto_command, &VMCommandType::Cgoto) {
            if function_context.is_empty() {
                translated_command.push_str(&format!("@{label_name}\n0;JMP"));
            } else {
                translated_command.push_str(&format!("@{function_context}${label_name}\n0;JMP"));
            }

            Some(translated_command)
        } else {
            None
        }
    }

    fn write_if(&self, if_command: &str, function_context: &str) -> Option<String> {
        let mut translated_command = String::from("");
        if let Some(label_name) = self.code_parser.arg1(if_command, &VMCommandType::Cif) {
            if function_context.is_empty() {
                translated_command.push_str(&format!("@SP\nAM=M-1\nD=M\n@{label_name}\nD;JNE"));
            } else {
                translated_command.push_str(&format!(
                    "@SP\nAM=M-1\nD=M\n@{function_context}${label_name}\nD;JNE"
                ));
            }

            Some(translated_command)
        } else {
            None
        }
    }

    fn write_function(&self, function_name: &str, local_vars: i16) -> Option<String> {
        let mut translated_command = String::from("");
        translated_command.push_str(&format!("({function_name})\n"));
        // intialize local memory segment on global stack for current called function
        // this means base address of called function's local memory segment is on the stack's memory segment....confusing
        translated_command.push_str("@0\nD=A\n");
        for _index in 0..local_vars {
            translated_command.push_str("@SP\nA=M\nM=D\n@SP\nM=M+1\n");
        }

        translated_command.pop(); // remove last \n
        Some(translated_command)
    }

    fn write_call(&self, function_name: &str, args: i16, return_address: &str) -> Option<String> {
        let mut translated_command = String::from("");
        // save return_address
        translated_command.push_str(&format!(
            "@{return_address}\nD=A\n@SP\nA=M\nM=D\n@SP\nM=M+1\n"
        ));
        // save segment pointers
        let assign_sp = "D=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n";
        translated_command.push_str(&format!("@LCL\n{assign_sp}"));
        translated_command.push_str(&format!("@ARG\n{assign_sp}"));
        translated_command.push_str(&format!("@THIS\n{assign_sp}"));
        translated_command.push_str(&format!("@THAT\n{assign_sp}"));
        // reposition arg pointer for called function
        let backtrack_count = 5 + args;
        translated_command.push_str(&format!("@{backtrack_count}\nD=A\n@SP\nD=M-D\n@ARG\nM=D\n"));
        // reposition LCL pointer for called function
        translated_command.push_str("@SP\nD=M\n@LCL\nM=D\n");
        // jump to execute function at its label
        translated_command.push_str(&format!("@{function_name}\n0;JMP\n"));

        // jump back to continue overall program flow using return address
        translated_command.push_str(&format!("({return_address})"));
        Some(translated_command)
    }

    fn write_return(&self) -> Option<String> {
        let mut translated_command = String::from("");
        // get end frame, end frame is not the end of the global stack, but the starting stack address
        // of the current called function which is the current called function's LCL pointer...
        translated_command.push_str("@LCL\nD=M\n@R13\nM=D\n");
        // get return address
        translated_command.push_str("@5\nA=D-A\nD=M\n@R14\nM=D\n");
        // copy top stack value which is the function's return value to function's arg pointer which is also under caller's stack
        translated_command.push_str("@SP\nA=M-1\nD=M\n@ARG\nA=M\nM=D\n");
        // set stack pointer to just after the arg pointer
        translated_command.push_str("@ARG\nD=M+1\n@SP\nM=D\n");
        // restore caller's memory segments
        let restore_start = "@R13\nAM=M-1\nD=M\n";
        translated_command.push_str(&format!("{restore_start}@THAT\nM=D\n"));
        translated_command.push_str(&format!("{restore_start}@THIS\nM=D\n"));
        translated_command.push_str(&format!("{restore_start}@ARG\nM=D\n"));
        translated_command.push_str(&format!("{restore_start}@LCL\nM=D\n"));
        // goto return address
        translated_command.push_str("@R14\nA=M\n0;JMP");

        Some(translated_command)
    }

    fn write_push(
        &self,
        segment_value: &str,
        index_value: &str,
        segment_list: &[&str],
        file_name: &str,
    ) -> Option<String> {
        let mut translated_command = String::from("");
        let increment_sp = "@SP\nA=M\nM=D\n@SP\nM=M+1";
        if segment_list.contains(&segment_value) {
            let segment_value_upper_case: &str = &segment_value.to_uppercase();
            match segment_value_upper_case {
                "CONSTANT" => {
                    translated_command.push_str(&format!("@{index_value}\nD=A\n"));
                }
                "STATIC" => {
                    translated_command.push_str(&format!("@{file_name}.{index_value}\nD=M\n"));
                }
                "POINTER" => {
                    let pointer_end = "D=M\n";
                    if index_value == "0" {
                        translated_command.push_str("@THIS\n");
                        translated_command.push_str(pointer_end);
                    } else if index_value == "1" {
                        translated_command.push_str("@THAT\n");
                        translated_command.push_str(pointer_end);
                    }
                }
                "TEMP" => {
                    translated_command.push_str(&format!("@{index_value}\nD=A\n@5\nA=D+A\nD=M\n"));
                }
                "LOCAL" => {
                    translated_command
                        .push_str(&format!("@LCL\nD=M\n@{index_value}\nA=D+A\nD=M\n"));
                }
                "ARGUMENT" => {
                    translated_command
                        .push_str(&format!("@ARG\nD=M\n@{index_value}\nA=D+A\nD=M\n"));
                }
                _ => {
                    translated_command.push_str(&format!(
                        "@{segment_value_upper_case}\nD=M\n@{index_value}\nA=D+A\nD=M\n"
                    ));
                }
            }
        }

        if !translated_command.is_empty() {
            translated_command.push_str(increment_sp);
            Some(translated_command)
        } else {
            None
        }
    }

    fn write_pop(
        &self,
        segment_value: &str,
        index_value: &str,
        segment_list: &[&str],
        file_name: &str,
    ) -> Option<String> {
        let mut translated_command = String::from("");

        if segment_list.contains(&segment_value) {
            let deref_sp = "@SP\nAM=M-1\nD=M\n";
            let segment_value_upper_case: &str = &segment_value.to_uppercase();
            match segment_value_upper_case {
                "STATIC" => {
                    translated_command
                        .push_str(&format!("{deref_sp}@{file_name}.{index_value}\nM=D"));
                }
                "POINTER" => {
                    let pointer_end = "M=D";
                    if index_value == "0" {
                        translated_command.push_str(&format!("{deref_sp}@THIS\n"));
                        translated_command.push_str(pointer_end);
                    } else if index_value == "1" {
                        translated_command.push_str(&format!("{deref_sp}@THAT\n"));
                        translated_command.push_str(pointer_end);
                    }
                }
                "TEMP" => {
                    translated_command.push_str(&format!(
                        "@5\nD=A\n@{index_value}\nD=D+A\n@R13\nM=D\n{deref_sp}@R13\nA=M\nM=D"
                    ));
                }
                "LOCAL" => {
                    translated_command.push_str(&format!(
                        "@{index_value}\nD=A\n@LCL\nD=D+M\n@R13\nM=D\n{deref_sp}@R13\nA=M\nM=D"
                    ));
                }
                "ARGUMENT" => {
                    translated_command.push_str(&format!(
                        "@{index_value}\nD=A\n@ARG\nD=D+M\n@R13\nM=D\n{deref_sp}@R13\nA=M\nM=D"
                    ));
                }
                _ => {
                    translated_command.push_str(&format!("@{index_value}\nD=A\n@{segment_value_upper_case}\nD=D+M\n@R13\nM=D\n{deref_sp}@R13\nA=M\nM=D"));
                }
            }
        }

        if !translated_command.is_empty() {
            Some(translated_command)
        } else {
            None
        }
    }

    fn write_arithmetic(
        &self,
        current_command: &str,
        segment_list: &[&str],
        line_number: &i16,
    ) -> Option<String> {
        let mut translated_command = String::from("");

        if segment_list.contains(&current_command) {
            let deref_sp = "@SP\nAM=M-1\nD=M\n";
            let push_bool = "@SP\nA=M-1\nM=D";
            match current_command {
                "add" => {
                    translated_command.push_str(&format!("{deref_sp}A=A-1\nM=D+M"));
                }
                "sub" => {
                    translated_command.push_str(&format!("{deref_sp}A=A-1\nM=M-D"));
                }
                "neg" => {
                    translated_command.push_str("@SP\nA=M-1\nM=-M");
                }
                "eq" => {
                    translated_command.push_str(&format!("{deref_sp}A=A-1\nD=M-D\n@equal.{line_number}\nD;JEQ\nD=0\n@done.{line_number}\n0;JMP\n(equal.{line_number})\nD=-1\n(done.{line_number})\n{push_bool}"));
                }
                "gt" => {
                    translated_command.push_str(&format!("{deref_sp}A=A-1\nD=M-D\n@greater.{line_number}\nD;JGT\nD=0\n@done.{line_number}\n0;JMP\n(greater.{line_number})\nD=-1\n(done.{line_number})\n{push_bool}"));
                }
                "lt" => {
                    translated_command.push_str(&format!("{deref_sp}A=A-1\nD=M-D\n@lesser.{line_number}\nD;JLT\nD=0\n@done.{line_number}\n0;JMP\n(lesser.{line_number})\nD=-1\n(done.{line_number})\n{push_bool}"));
                }
                "and" => {
                    translated_command.push_str(&format!("{deref_sp}A=A-1\nM=D&M"));
                }
                "or" => {
                    translated_command.push_str(&format!("{deref_sp}A=A-1\nM=D|M"));
                }
                "not" => {
                    translated_command.push_str("@SP\nA=M-1\nM=!M");
                }
                _ => {}
            }
        }

        if !translated_command.is_empty() {
            Some(translated_command)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] // test for removing comments and blank lines
    fn parse_clean_instructions() {
        let input_1 = String::from("push constant 5 // to be removed\n ");
        let input_2 = String::from("gt  ");
        let input_3 = String::from("\npop local 2 // sad\n push static 5");
        let input_4 =
            String::from("\npop local 2 // sad\n push static 5\nadd // adding\n    \nsub//23");
        let test_parser = VmCodeParser::new();

        let cleaned_vm_code1 = test_parser.clean_vm_code(input_1);
        let cleaned_vm_code2 = test_parser.clean_vm_code(input_2);
        let cleaned_vm_code3 = test_parser.clean_vm_code(input_3);
        let cleaned_vm_code4 = test_parser.clean_vm_code(input_4);

        assert_eq!("push constant 5".to_string(), cleaned_vm_code1);
        assert_eq!("gt".to_string(), cleaned_vm_code2);
        assert_eq!("pop local 2\npush static 5".to_string(), cleaned_vm_code3);
        assert_eq!(
            "pop local 2\npush static 5\nadd\nsub".to_string(),
            cleaned_vm_code4
        );
    }

    #[test]
    fn parse_command_type() {
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
        let test_parser = VmCodeParser::new();
        let push_command = test_parser.command_type("push constant 0", &command_symbol_table);
        let pop_command = test_parser.command_type("pop this 5", &command_symbol_table);
        let arith_command = test_parser.command_type("and", &command_symbol_table);
        let label_command = test_parser.command_type("label END", &command_symbol_table);
        let goto_command = test_parser.command_type("goto MOON", &command_symbol_table);
        let if_goto_command = test_parser.command_type("if-goto test", &command_symbol_table);
        let call_command = test_parser.command_type("call Crazy.frog 9", &command_symbol_table);
        let function_command =
            test_parser.command_type("function Crazy.frog 3", &command_symbol_table);
        let return_command = test_parser.command_type("return", &command_symbol_table);
        let invalid_command = test_parser.command_type("yo mama", &command_symbol_table);

        assert!(push_command.is_some_and(|value| value == VMCommandType::Cpush));
        assert!(pop_command.is_some_and(|value| value == VMCommandType::Cpop));
        assert!(arith_command.is_some_and(|value| value == VMCommandType::Carithmetic));
        assert!(label_command.is_some_and(|value| value == VMCommandType::Clabel));
        assert!(goto_command.is_some_and(|value| value == VMCommandType::Cgoto));
        assert!(if_goto_command.is_some_and(|value| value == VMCommandType::Cif));
        assert!(call_command.is_some_and(|value| value == VMCommandType::Ccall));
        assert!(function_command.is_some_and(|value| value == VMCommandType::Cfunction));
        assert!(return_command.is_some_and(|value| value == VMCommandType::Creturn));
        assert!(invalid_command.is_none());
    }

    #[test]
    fn parse_args() {
        let test_parser = VmCodeParser::new();
        let push_arg1 = test_parser.arg1("push constant 0", &VMCommandType::Cpush);
        let push_arg2 = test_parser.arg2("push constant 0", &VMCommandType::Cpush);
        let pop_arg1 = test_parser.arg1("pop static 2", &VMCommandType::Cpop);
        let pop_arg2 = test_parser.arg2("pop static 2", &VMCommandType::Cpop);
        let arith_arg1 = test_parser.arg1("add", &VMCommandType::Carithmetic);
        let arith_arg2 = test_parser.arg2("sub", &VMCommandType::Carithmetic);
        let label_arg1 = test_parser.arg1("label OKOKOK", &VMCommandType::Clabel);
        let label_arg2 = test_parser.arg2("label OKOKOK", &VMCommandType::Clabel);
        let goto_arg1 = test_parser.arg1("goto OKOKOK", &VMCommandType::Cgoto);
        let goto_arg2 = test_parser.arg2("goto OKOKOK", &VMCommandType::Cgoto);
        let if_goto_arg1 = test_parser.arg1("if-goto OKOKOK", &VMCommandType::Cif);
        let if_goto_arg2 = test_parser.arg2("if-goto OKOKOK", &VMCommandType::Cif);
        let call_arg1 = test_parser.arg1("call yopapa 4", &VMCommandType::Ccall);
        let call_arg2 = test_parser.arg2("call yopapa 4", &VMCommandType::Ccall);
        let function_arg1 = test_parser.arg1("function yopapa 2", &VMCommandType::Cfunction);
        let function_arg2 = test_parser.arg2("function yopapa 2", &VMCommandType::Cfunction);
        let return_arg1 = test_parser.arg1("return", &VMCommandType::Creturn);
        let return_arg2 = test_parser.arg2("return 4", &VMCommandType::Creturn);
        assert!(push_arg1.is_some_and(|value| value == "constant"));
        assert!(push_arg2.is_some_and(|value| value == "0"));
        assert!(pop_arg1.is_some_and(|value| value == "static"));
        assert!(pop_arg2.is_some_and(|value| value == "2"));
        assert!(arith_arg1.is_none());
        assert!(arith_arg2.is_none());
        assert!(label_arg1.is_some_and(|value| value == "OKOKOK"));
        assert!(label_arg2.is_none());
        assert!(goto_arg1.is_some_and(|value| value == "OKOKOK"));
        assert!(goto_arg2.is_none());
        assert!(if_goto_arg1.is_some_and(|value| value == "OKOKOK"));
        assert!(if_goto_arg2.is_none());
        assert!(call_arg1.is_some_and(|value| value == "yopapa"));
        assert!(call_arg2.is_some_and(|value| value == "4"));
        assert!(function_arg1.is_some_and(|value| value == "yopapa"));
        assert!(function_arg2.is_some_and(|value| value == "2"));
        assert!(return_arg1.is_none());
        assert!(return_arg2.is_none());
    }
}
