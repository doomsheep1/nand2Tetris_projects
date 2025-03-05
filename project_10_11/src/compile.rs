use std::{fs::File, io::LineWriter, path::PathBuf};

use crate::{
    symbol::{Kind, SymbolTable},
    tokenizer::{JackTokenizer, TokenType},
    writer::{Commands, VMSegment, VmWriter},
};

pub struct CompilationEngine {
    tokenizer: JackTokenizer,
    vm_writer: VmWriter,
    class_symbol_table: SymbolTable,
    subroutine_symbol_table: SymbolTable,
}

impl CompilationEngine {
    pub fn new(jack_file: &PathBuf) -> Result<Self, std::io::Error> {
        let jack_tokenizer = JackTokenizer::new(jack_file)?;
        let output_vm_file = File::create(jack_file.with_extension("vm"))?;
        let output_vm_file = LineWriter::new(output_vm_file);
        let class_name = jack_file
            .file_stem()
            .expect("Impossible to be None")
            .to_str()
            .expect("Impossible to be None")
            .to_string();
        let vm_writer = VmWriter::new(output_vm_file, class_name);
        Ok(Self {
            tokenizer: jack_tokenizer,
            vm_writer,
            class_symbol_table: SymbolTable::new(),
            subroutine_symbol_table: SymbolTable::new(),
        })
    }

    pub fn compile_class(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut bad_token = String::from("");
        self.tokenizer.advance()?;
        let token = self.tokenizer.get_token();
        if token == "class" {
            self.tokenizer.advance()?;
            self.tokenizer.advance()?;
            let mut token = self.tokenizer.get_token();
            if token == "{" {
                self.tokenizer.advance()?;
                token = self.tokenizer.get_token();

                while token == "static" || token == "field" {
                    self.compile_class_variable_declaration()?;
                    self.tokenizer.advance()?;
                    token = self.tokenizer.get_token();
                }

                while token == "constructor" || token == "function" || token == "method" {
                    self.subroutine_symbol_table.reset();
                    self.compile_subroutine_declaration()?;
                    self.tokenizer.advance()?;
                    token = self.tokenizer.get_token();
                }

                if token != "}" {
                    bad_token = token.into();
                }
            } else {
                bad_token = token.into();
            }
        } else {
            bad_token = token.into();
        }

        if !bad_token.is_empty() {
            Err(format!(
                "Class compilation failed, invalid token type or wrong token: {bad_token}"
            ))?;
        }

        Ok(())
    }

    fn compile_class_variable_declaration(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut bad_token = String::from("");
        let mut curr_kind: Kind;
        let mut curr_type: String;
        let mut curr_name: String;
        let mut token = self.tokenizer.get_token();
        while token == "static" || token == "field" {
            curr_kind = if token == "static" {
                Kind::STATIC
            } else {
                Kind::THIS
            };

            self.tokenizer.advance()?;
            token = self.tokenizer.get_token();
            curr_type = token.into();

            self.tokenizer.advance()?;
            token = self.tokenizer.get_token();
            curr_name = token.into();
            self.class_symbol_table.define(
                curr_name.clone(),
                curr_type.clone(),
                curr_kind.clone(),
            )?;

            self.tokenizer.advance()?;
            token = self.tokenizer.get_token();
            while token == "," {
                self.tokenizer.advance()?;
                token = self.tokenizer.get_token();
                curr_name = token.into();
                self.class_symbol_table.define(
                    curr_name.clone(),
                    curr_type.clone(),
                    curr_kind.clone(),
                )?;
                self.tokenizer.advance()?;
                token = self.tokenizer.get_token();
            }

            if token != ";" {
                bad_token = token.into();
            }
        }

        if !bad_token.is_empty() {
            Err(format!("Class variable declaration compilation failed, invalid token type or wrong token: {bad_token}"))?;
        }

        Ok(())
    }

    fn compile_subroutine_declaration(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut bad_token = String::from("");
        let mut token = self.tokenizer.get_token();
        let mut is_constructor = false;
        let mut is_function = false;
        if token == "method" {
            self.subroutine_symbol_table.define(
                "this".to_string(),
                self.vm_writer.get_class_name().to_string(),
                Kind::ARG,
            )?;
        } else if token == "constructor" {
            is_constructor = true;
        } else if token == "function" {
            is_function = true;
        }
        self.tokenizer.advance()?;
        self.tokenizer.advance()?;
        token = self.tokenizer.get_token();
        let curr_name: String = token.into();
        self.tokenizer.advance()?;
        token = self.tokenizer.get_token();
        if token == "(" {
            self.compile_parameter_list()?;
            token = self.tokenizer.get_token();
            if token == ")" {
                self.tokenizer.advance()?;
                token = self.tokenizer.get_token();
                if token == "{" {
                    let var_count = self.compile_variable_declaration()?;
                    self.vm_writer.write_function(&curr_name, var_count)?;
                    if is_constructor {
                        self.vm_writer.write_push(
                            &VMSegment::CONSTANT,
                            self.class_symbol_table.var_count(&Kind::THIS),
                        )?;
                        self.vm_writer.write_call("Memory.alloc", 1)?;
                    } else if let Some(this_arg_index) =
                        self.subroutine_symbol_table.index_of("this")
                    {
                        self.vm_writer
                            .write_push(&VMSegment::ARGUMENT, this_arg_index)?;
                    }

                    if !is_function {
                        self.vm_writer.write_pop(&VMSegment::POINTER, 0)?;
                    }

                    self.compile_statements()?;
                    token = self.tokenizer.get_token();
                    if token != "}" {
                        bad_token = token.into();
                    }
                } else {
                    bad_token = token.into();
                }
            } else {
                bad_token = token.into();
            }
        } else {
            bad_token = token.into();
        }

        if !bad_token.is_empty() {
            Err(format!(
                "Subroutine declaration compilation failed, invalid token type or wrong token: {bad_token}"
            ))?;
        }

        Ok(())
    }

    fn compile_parameter_list(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        let mut curr_kind: Kind;
        let mut curr_type: String;
        let mut curr_name: String;
        self.tokenizer.advance()?;
        let mut token = self.tokenizer.get_token();
        let mut var_count: usize = 0;
        if token != ")" {
            while token != ")" {
                curr_kind = Kind::ARG;
                curr_type = token.into();

                self.tokenizer.advance()?;
                token = self.tokenizer.get_token();
                curr_name = token.into();
                self.subroutine_symbol_table.define(
                    curr_name.clone(),
                    curr_type.clone(),
                    curr_kind.clone(),
                )?;

                self.tokenizer.advance()?;
                token = self.tokenizer.get_token();
                if token == "," {
                    self.tokenizer.advance()?;
                    token = self.tokenizer.get_token();
                }

                var_count += 1;
            }
        }

        Ok(var_count)
    }

    fn compile_variable_declaration(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        let mut curr_kind: Kind;
        let mut curr_type: String;
        let mut curr_name: String;
        let mut var_count: usize = 0;
        while self.tokenizer.get_file_content().starts_with("var") {
            self.tokenizer.advance()?;
            self.tokenizer.advance()?;
            let mut token = self.tokenizer.get_token();
            curr_type = token.into();
            curr_kind = Kind::LOCAL;
            self.tokenizer.advance()?;
            token = self.tokenizer.get_token();

            while token != ";" {
                curr_name = token.into();
                self.subroutine_symbol_table.define(
                    curr_name.clone(),
                    curr_type.clone(),
                    curr_kind.clone(),
                )?;
                var_count += 1;
                self.tokenizer.advance()?;
                token = self.tokenizer.get_token();
                if token == "," {
                    self.tokenizer.advance()?;
                    token = self.tokenizer.get_token();
                }
            }
        }

        Ok(var_count)
    }

    fn compile_statements(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut bad_token = String::from("");
        self.tokenizer.advance()?;
        let mut token = self.tokenizer.get_token();
        let mut token_type = self.tokenizer.get_token_type();
        if *token_type == TokenType::Keyword {
            while *token_type == TokenType::Keyword {
                match token {
                    "let" => {
                        self.compile_let_statement()?;
                    }
                    "if" => {
                        self.compile_if_statement()?;
                    }
                    "while" => {
                        self.compile_while_statement()?;
                    }
                    "do" => {
                        self.compile_do_statement()?;
                    }
                    "return" => {
                        self.compile_return_statement()?;
                    }
                    _ => {
                        bad_token = token.into();
                        break;
                    }
                }

                self.tokenizer.advance()?;
                token = self.tokenizer.get_token();
                token_type = self.tokenizer.get_token_type();
            }
        }

        if !bad_token.is_empty() {
            Err(format!(
                "Statements compilation failed, invalid token type or wrong token: {bad_token}"
            ))?;
        }

        Ok(())
    }

    fn compile_let_statement(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut bad_token = String::from("");
        let mut is_array = false;
        self.tokenizer.advance()?;
        let mut token = self.tokenizer.get_token();
        let (mut curr_kind, mut curr_index) = (
            self.subroutine_symbol_table.kind_of(token),
            self.subroutine_symbol_table.index_of(token),
        );

        if curr_kind.is_none() || curr_index.is_none() {
            (curr_kind, curr_index) = (
                self.class_symbol_table.kind_of(token),
                self.class_symbol_table.index_of(token),
            );
        }

        if curr_kind.is_none() || curr_index.is_none() {
            bad_token = token.into();
        } else {
            let curr_kind = curr_kind.unwrap();
            let curr_index = curr_index.unwrap();
            self.tokenizer.advance()?;
            token = self.tokenizer.get_token();
            if token == "[" {
                is_array = true;
                self.compile_expression()?;
                token = self.tokenizer.get_token();
                match curr_kind {
                    Kind::STATIC => self.vm_writer.write_push(&VMSegment::STATIC, curr_index)?,
                    Kind::ARG => self
                        .vm_writer
                        .write_push(&VMSegment::ARGUMENT, curr_index)?,
                    Kind::LOCAL => self.vm_writer.write_push(&VMSegment::LOCAL, curr_index)?,
                    Kind::THIS => self.vm_writer.write_push(&VMSegment::THIS, curr_index)?,
                    Kind::NONE => bad_token = token.into(),
                }

                if bad_token.is_empty() {
                    self.vm_writer.write_arithmetic(&Commands::ADD)?;

                    if token != "]" {
                        bad_token = token.into();
                    }

                    if bad_token.is_empty() {
                        self.tokenizer.advance()?;
                        token = self.tokenizer.get_token();
                    }
                }
            }

            if token == "=" {
                self.compile_expression()?;
                token = self.tokenizer.get_token();
                if token == ";" {
                    if is_array {
                        self.vm_writer.write_pop(&VMSegment::TEMP, 0)?;
                        self.vm_writer.write_pop(&VMSegment::POINTER, 1)?;
                        self.vm_writer.write_push(&VMSegment::TEMP, 0)?;
                        self.vm_writer.write_pop(&VMSegment::THAT, 0)?;
                    } else {
                        match curr_kind {
                            Kind::STATIC => {
                                self.vm_writer.write_pop(&VMSegment::STATIC, curr_index)?
                            }
                            Kind::ARG => {
                                self.vm_writer.write_pop(&VMSegment::ARGUMENT, curr_index)?
                            }
                            Kind::LOCAL => {
                                self.vm_writer.write_pop(&VMSegment::LOCAL, curr_index)?
                            }
                            Kind::THIS => self.vm_writer.write_pop(&VMSegment::THIS, curr_index)?,
                            Kind::NONE => bad_token = token.into(),
                        }
                    }
                } else {
                    bad_token = token.into();
                }
            } else {
                bad_token = token.into();
            }
        }

        if !bad_token.is_empty() {
            Err(format!(
                "Statements compilation failed, invalid token type or wrong token: {bad_token}"
            ))?;
        }

        Ok(())
    }

    fn compile_if_statement(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut bad_token = String::from("");
        let mut has_else_statement = false;
        let goto_label: usize = self.vm_writer.get_label_index();
        self.tokenizer.advance()?;
        let mut token = self.tokenizer.get_token();
        if token == "(" {
            self.compile_expression()?;
            self.vm_writer.write_arithmetic(&Commands::NOT)?;
            let if_goto_label = self.vm_writer.get_label_index();
            self.vm_writer.write_if(if_goto_label)?;
            token = self.tokenizer.get_token();
            if token == ")" {
                self.tokenizer.advance()?;
                token = self.tokenizer.get_token();
                if token == "{" {
                    self.compile_statements()?;
                    self.vm_writer.write_goto(goto_label)?;
                    token = self.tokenizer.get_token();
                    if token == "}" {
                        if self.tokenizer.get_file_content().starts_with("else") {
                            has_else_statement = true;
                            self.vm_writer.write_label(if_goto_label)?;
                            self.tokenizer.advance()?;
                            self.tokenizer.advance()?;
                            token = self.tokenizer.get_token();
                            if token == "{" {
                                self.compile_statements()?;
                                token = self.tokenizer.get_token();
                                if token != "}" {
                                    bad_token = token.into();
                                }
                            } else {
                                bad_token = token.into();
                            }
                        }

                        if !has_else_statement {
                            self.vm_writer.write_label(if_goto_label)?;
                        }

                        self.vm_writer.write_label(goto_label)?;
                    } else {
                        bad_token = token.into();
                    }
                } else {
                    bad_token = token.into();
                }
            } else {
                bad_token = token.into();
            }
        } else {
            bad_token = token.into();
        }

        if !bad_token.is_empty() {
            Err(format!(
                "If statement compilation failed, invalid token type or wrong token: {bad_token}"
            ))?;
        }

        Ok(())
    }

    fn compile_while_statement(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut bad_token = String::from("");
        let continue_while_label: usize = self.vm_writer.get_label_index();
        self.vm_writer.write_label(continue_while_label)?;
        self.tokenizer.advance()?;
        let mut token = self.tokenizer.get_token();
        if token == "(" {
            self.compile_expression()?;
            self.vm_writer.write_arithmetic(&Commands::NOT)?;
            let skip_while_label = self.vm_writer.get_label_index();
            self.vm_writer.write_if(skip_while_label)?;
            token = self.tokenizer.get_token();
            if token == ")" {
                self.tokenizer.advance()?;
                token = self.tokenizer.get_token();
                if token == "{" {
                    self.compile_statements()?;
                    self.vm_writer.write_goto(continue_while_label)?;
                    token = self.tokenizer.get_token();
                    if token == "}" {
                        self.vm_writer.write_label(skip_while_label)?;
                    } else {
                        bad_token = token.into();
                    }
                } else {
                    bad_token = token.into();
                }
            } else {
                bad_token = token.into();
            }
        } else {
            bad_token = token.into();
        }

        if !bad_token.is_empty() {
            Err(format!(
                "While statement compilation failed, invalid token type or wrong token: {bad_token}"
            ))?;
        }

        Ok(())
    }

    fn compile_do_statement(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut bad_token = String::from("");
        let mut arg_count: usize = 0;
        self.tokenizer.advance()?;
        let mut token = self.tokenizer.get_token();

        let (curr_name, mut curr_kind, mut curr_index, mut curr_type) = (
            token.to_string(),
            self.subroutine_symbol_table.kind_of(token),
            self.subroutine_symbol_table.index_of(token),
            self.subroutine_symbol_table.type_of(token),
        );

        if curr_kind.is_none() || curr_index.is_none() || curr_type.is_none() {
            (curr_kind, curr_index, curr_type) = (
                self.class_symbol_table.kind_of(token),
                self.class_symbol_table.index_of(token),
                self.class_symbol_table.type_of(token),
            );
        }

        if self.tokenizer.get_file_content().starts_with('(') {
            self.tokenizer.advance()?;
            arg_count += 1; // this
            self.vm_writer.write_push(&VMSegment::POINTER, 0)?;
            arg_count += self.compile_expression_list()?;
            token = self.tokenizer.get_token();
            if token == ")" {
                self.vm_writer.write_call(
                    &format!("{}.{curr_name}", self.vm_writer.get_class_name()),
                    arg_count,
                )?;
                self.vm_writer.write_pop(&VMSegment::TEMP, 0)?;
            } else {
                bad_token = token.into();
            }
        } else if self.tokenizer.get_file_content().starts_with('.') {
            self.tokenizer.advance()?;
            self.tokenizer.advance()?;
            let mut token = self.tokenizer.get_token();
            let function_name: String = token.into();
            self.tokenizer.advance()?;
            token = self.tokenizer.get_token();
            if token == "(" {
                if let (Some(curr_kind), Some(curr_index)) = (curr_kind, curr_index) {
                    match curr_kind {
                        Kind::STATIC => {
                            self.vm_writer.write_push(&VMSegment::STATIC, curr_index)?
                        }
                        Kind::ARG => self
                            .vm_writer
                            .write_push(&VMSegment::ARGUMENT, curr_index)?,
                        Kind::LOCAL => self.vm_writer.write_push(&VMSegment::LOCAL, curr_index)?,
                        Kind::THIS => self.vm_writer.write_push(&VMSegment::THIS, curr_index)?,
                        Kind::NONE => bad_token = token.into(),
                    }
                    arg_count += 1; // this
                }

                arg_count += self.compile_expression_list()?;
                token = self.tokenizer.get_token();
                if token == ")" {
                    if let Some(curr_type) = curr_type {
                        self.vm_writer
                            .write_call(&format!("{}.{function_name}", curr_type), arg_count)?;
                    } else {
                        self.vm_writer
                            .write_call(&format!("{}.{function_name}", curr_name), arg_count)?;
                    }

                    self.vm_writer.write_pop(&VMSegment::TEMP, 0)?;
                } else {
                    bad_token = token.into();
                }
            } else {
                bad_token = token.into();
            }
        } else {
            bad_token = token.into();
        }

        self.tokenizer.advance()?;
        token = self.tokenizer.get_token();
        if token != ";" {
            bad_token = token.into();
        }

        if !bad_token.is_empty() {
            Err(format!(
                "Do statement compilation failed, invalid token type or wrong token: {bad_token}"
            ))?;
        }

        Ok(())
    }

    fn compile_return_statement(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut bad_token = String::from("");
        if !self.tokenizer.get_file_content().starts_with(';') {
            self.compile_expression()?;
        } else {
            self.vm_writer.write_push(&VMSegment::CONSTANT, 0)?;
            self.tokenizer.advance()?;
        }

        let token = self.tokenizer.get_token();
        if token == ";" {
            self.vm_writer.write_return()?;
        } else {
            bad_token = token.into();
        }

        if !bad_token.is_empty() {
            Err(format!(
                "Return statement compilation failed, invalid token type or wrong token: {bad_token}"
            ))?;
        }

        Ok(())
    }

    fn compile_expression(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.compile_term()?;
        self.tokenizer.advance()?;
        let mut bad_token = String::from("");
        let mut token = self.tokenizer.get_token();
        let mut token_type = self.tokenizer.get_token_type();
        while *token_type == TokenType::Symbol
            && token != "]"
            && token != ";"
            && token != ")"
            && token != ","
        {
            match token {
                "+" => {
                    self.compile_term()?;
                    self.vm_writer.write_arithmetic(&Commands::ADD)?;
                }
                "-" => {
                    self.compile_term()?;
                    self.vm_writer.write_arithmetic(&Commands::SUB)?;
                }
                "*" => {
                    self.compile_term()?;
                    self.vm_writer.write_call("Math.multiply", 2)?;
                }
                "/" => {
                    self.compile_term()?;
                    self.vm_writer.write_call("Math.divide", 2)?;
                }
                "|" => {
                    self.compile_term()?;
                    self.vm_writer.write_arithmetic(&Commands::OR)?;
                }
                "=" => {
                    self.compile_term()?;
                    self.vm_writer.write_arithmetic(&Commands::EQ)?;
                }
                "<" => {
                    self.compile_term()?;
                    self.vm_writer.write_arithmetic(&Commands::LT)?;
                }
                ">" => {
                    self.compile_term()?;
                    self.vm_writer.write_arithmetic(&Commands::GT)?;
                }
                "&" => {
                    self.compile_term()?;
                    self.vm_writer.write_arithmetic(&Commands::AND)?;
                }
                _ => {
                    bad_token = token.into();
                    break;
                }
            }

            self.tokenizer.advance()?;
            token = self.tokenizer.get_token();
            token_type = self.tokenizer.get_token_type();
        }

        if !bad_token.is_empty() {
            Err(format!(
                "Expression compilation failed, invalid token type or wrong token: {bad_token}"
            ))?;
        }

        Ok(())
    }

    fn compile_term(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut bad_token = String::from("");
        self.tokenizer.advance()?;
        let mut token = self.tokenizer.get_token();
        let token_type = self.tokenizer.get_token_type();
        match *token_type {
            TokenType::IntConstant => {
                self.vm_writer
                    .write_push(&VMSegment::CONSTANT, token.parse::<usize>()?)?;
            }
            TokenType::StringConst => {
                token = token.trim_matches('"');
                self.vm_writer
                    .write_push(&VMSegment::CONSTANT, token.len())?;
                self.vm_writer.write_call("String.new", 1)?;
                let char_token = token.chars();
                char_token.into_iter().for_each(|character| {
                    if self
                        .vm_writer
                        .write_push(&VMSegment::CONSTANT, character as usize)
                        .is_err()
                        || self.vm_writer.write_call("String.appendChar", 2).is_err()
                    {
                        bad_token = token.into();
                    }
                });
            }
            TokenType::Keyword => {
                if token == "true" {
                    self.vm_writer.write_push(&VMSegment::CONSTANT, 1)?;
                    self.vm_writer.write_arithmetic(&Commands::NEG)?;
                } else if token == "false" || token == "null" {
                    self.vm_writer.write_push(&VMSegment::CONSTANT, 0)?;
                } else if token == "this" {
                    self.vm_writer.write_push(&VMSegment::POINTER, 0)?;
                } else {
                    bad_token = token.into();
                }
            }
            TokenType::Identifier => {
                let (curr_name, mut curr_kind, mut curr_index, mut curr_type) = (
                    token.to_string(),
                    self.subroutine_symbol_table.kind_of(token),
                    self.subroutine_symbol_table.index_of(token),
                    self.subroutine_symbol_table.type_of(token),
                );

                if curr_kind.is_none() || curr_index.is_none() || curr_type.is_none() {
                    (curr_kind, curr_index, curr_type) = (
                        self.class_symbol_table.kind_of(token),
                        self.class_symbol_table.index_of(token),
                        self.class_symbol_table.type_of(token),
                    );
                }

                if self.tokenizer.get_file_content().starts_with('[') {
                    self.tokenizer.advance()?;
                    self.compile_expression()?;
                    let token = self.tokenizer.get_token();
                    if curr_kind.is_some() && curr_index.is_some() {
                        match curr_kind.unwrap() {
                            Kind::STATIC => self
                                .vm_writer
                                .write_push(&VMSegment::STATIC, curr_index.unwrap())?,
                            Kind::ARG => self
                                .vm_writer
                                .write_push(&VMSegment::ARGUMENT, curr_index.unwrap())?,
                            Kind::LOCAL => self
                                .vm_writer
                                .write_push(&VMSegment::LOCAL, curr_index.unwrap())?,
                            Kind::THIS => self
                                .vm_writer
                                .write_push(&VMSegment::THIS, curr_index.unwrap())?,
                            Kind::NONE => bad_token = token.into(),
                        }
                    }
                    self.vm_writer.write_arithmetic(&Commands::ADD)?;
                    if token == "]" {
                        self.vm_writer.write_pop(&VMSegment::POINTER, 1)?;
                        self.vm_writer.write_push(&VMSegment::THAT, 0)?;
                    } else {
                        bad_token = token.into();
                    }
                } else if self.tokenizer.get_file_content().starts_with('(') {
                    if curr_kind.is_some() && curr_index.is_some() {
                        match curr_kind.unwrap() {
                            Kind::STATIC => self
                                .vm_writer
                                .write_push(&VMSegment::STATIC, curr_index.unwrap())?,
                            Kind::ARG => self
                                .vm_writer
                                .write_push(&VMSegment::ARGUMENT, curr_index.unwrap())?,
                            Kind::LOCAL => self
                                .vm_writer
                                .write_push(&VMSegment::LOCAL, curr_index.unwrap())?,
                            Kind::THIS => self
                                .vm_writer
                                .write_push(&VMSegment::THIS, curr_index.unwrap())?,
                            Kind::NONE => bad_token = token.into(),
                        }
                    }
                    self.tokenizer.advance()?;
                    let arg_count = self.compile_expression_list()?;
                    let token = self.tokenizer.get_token();
                    if token == ")" {
                        self.vm_writer.write_call(
                            &format!("{}.{curr_name}", self.vm_writer.get_class_name()),
                            arg_count,
                        )?;
                    } else {
                        bad_token = token.into();
                    }
                } else if self.tokenizer.get_file_content().starts_with('.') {
                    if curr_kind.is_some() && curr_index.is_some() {
                        match curr_kind.unwrap() {
                            Kind::STATIC => self
                                .vm_writer
                                .write_push(&VMSegment::STATIC, curr_index.unwrap())?,
                            Kind::ARG => self
                                .vm_writer
                                .write_push(&VMSegment::ARGUMENT, curr_index.unwrap())?,
                            Kind::LOCAL => self
                                .vm_writer
                                .write_push(&VMSegment::LOCAL, curr_index.unwrap())?,
                            Kind::THIS => self
                                .vm_writer
                                .write_push(&VMSegment::THIS, curr_index.unwrap())?,
                            Kind::NONE => bad_token = token.into(),
                        }
                    }
                    self.tokenizer.advance()?;
                    self.tokenizer.advance()?;
                    let mut token = self.tokenizer.get_token();
                    let function_name: String = token.into();
                    self.tokenizer.advance()?;
                    token = self.tokenizer.get_token();
                    if token == "(" {
                        let mut arg_count = self.compile_expression_list()?;
                        token = self.tokenizer.get_token();
                        if token == ")" {
                            if let Some(curr_type) = curr_type {
                                arg_count += 1;
                                self.vm_writer.write_call(
                                    &format!("{curr_type}.{function_name}"),
                                    arg_count,
                                )?;
                            } else {
                                self.vm_writer.write_call(
                                    &format!("{curr_name}.{function_name}"),
                                    arg_count,
                                )?;
                            }
                        } else {
                            bad_token = token.into();
                        }
                    } else {
                        bad_token = token.into();
                    }
                } else if curr_kind.is_some() && curr_index.is_some() {
                    dbg!(&curr_name);
                    curr_kind.clone().inspect(|x| println!("what is x {:?}", x));
                    match curr_kind.unwrap() {
                        Kind::STATIC => self
                            .vm_writer
                            .write_push(&VMSegment::STATIC, curr_index.unwrap())?,
                        Kind::ARG => self
                            .vm_writer
                            .write_push(&VMSegment::ARGUMENT, curr_index.unwrap())?,
                        Kind::LOCAL => self
                            .vm_writer
                            .write_push(&VMSegment::LOCAL, curr_index.unwrap())?,
                        Kind::THIS => self
                            .vm_writer
                            .write_push(&VMSegment::THIS, curr_index.unwrap())?,
                        Kind::NONE => bad_token = token.into(),
                    }
                }
            }
            TokenType::Symbol => {
                if token == "(" {
                    self.compile_expression()?;
                    token = self.tokenizer.get_token();
                    if token != ")" {
                        bad_token = token.into();
                    }
                } else if token == "-" || token == "~" {
                    if token == "-" {
                        self.compile_term()?;
                        self.vm_writer.write_arithmetic(&Commands::NEG)?;
                    } else if token == "~" {
                        self.compile_term()?;
                        self.vm_writer.write_arithmetic(&Commands::NOT)?;
                    }
                } else {
                    bad_token = token.into();
                }
            }
            TokenType::None => {
                bad_token = token.into();
            }
        }

        if !bad_token.is_empty() {
            Err(format!(
                "Term compilation failed, invalid token type or wrong token: {bad_token}"
            ))?;
        }
        Ok(())
    }

    fn compile_expression_list(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        let mut bad_token = String::from("");
        let mut arg_count = 0;
        if !self.tokenizer.get_file_content().starts_with(')') {
            self.compile_expression()?;
            arg_count += 1;
            let mut token = self.tokenizer.get_token();
            while token != ")" {
                if token == "," {
                    self.compile_expression()?;
                    arg_count += 1;
                    token = self.tokenizer.get_token();
                } else {
                    bad_token = token.into();
                }
            }

            if !bad_token.is_empty() {
                Err(format!(
                    "Expression List compilation failed, invalid token type or wrong token: {bad_token}"
                ))?;
            }
        } else {
            self.tokenizer.advance()?
        }

        Ok(arg_count)
    }
}
