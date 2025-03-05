use std::{
    fs::File,
    io::{Error, ErrorKind, LineWriter, Write},
};

pub enum VMSegment {
    CONSTANT,
    ARGUMENT,
    LOCAL,
    STATIC,
    THIS,
    THAT,
    POINTER,
    TEMP,
}

pub enum Commands {
    ADD,
    SUB,
    NEG,
    EQ,
    GT,
    LT,
    AND,
    OR,
    NOT,
}

pub struct VmWriter {
    output_vm_file: LineWriter<File>,
    class_name: String,
    label_index: usize,
}

impl VmWriter {
    pub fn new(jack_vm_file: LineWriter<File>, class_name: String) -> Self {
        Self {
            class_name,
            output_vm_file: jack_vm_file,
            label_index: 0,
        }
    }

    pub fn get_class_name(&self) -> &str {
        &self.class_name
    }

    pub fn get_label_index(&mut self) -> usize {
        let curr_label = self.label_index;
        self.label_index += 1;
        curr_label
    }

    pub fn write_push(
        &mut self,
        segment: &VMSegment,
        segment_index: usize,
    ) -> Result<(), std::io::Error> {
        let segment_str = match segment {
            VMSegment::CONSTANT => "constant",
            VMSegment::ARGUMENT => "argument",
            VMSegment::LOCAL => "local",
            VMSegment::STATIC => "static",
            VMSegment::THIS => "this",
            VMSegment::THAT => "that",
            VMSegment::POINTER => "pointer",
            VMSegment::TEMP => "temp",
        };

        self.output_vm_file
            .write_all(format!("push {segment_str} {segment_index}\n").as_bytes())
    }

    pub fn write_pop(
        &mut self,
        segment: &VMSegment,
        segment_index: usize,
    ) -> Result<(), std::io::Error> {
        let segment_str = match segment {
            VMSegment::ARGUMENT => "argument",
            VMSegment::LOCAL => "local",
            VMSegment::STATIC => "static",
            VMSegment::THIS => "this",
            VMSegment::THAT => "that",
            VMSegment::POINTER => "pointer",
            VMSegment::TEMP => "temp",
            VMSegment::CONSTANT => "",
        };

        if segment_str.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Constant segment is invalid for popping",
            ));
        }

        self.output_vm_file
            .write_all(format!("pop {segment_str} {segment_index}\n").as_bytes())
    }

    pub fn write_function(
        &mut self,
        function_name: &str,
        var_count: usize,
    ) -> Result<(), std::io::Error> {
        self.output_vm_file.write_all(
            format!(
                "function {0}.{function_name} {var_count}\n",
                self.class_name
            )
            .as_bytes(),
        )
    }

    pub fn write_arithmetic(&mut self, command_type: &Commands) -> Result<(), std::io::Error> {
        let arithmetic_command = match command_type {
            Commands::ADD => "add",
            Commands::SUB => "sub",
            Commands::NEG => "neg",
            Commands::EQ => "eq",
            Commands::GT => "gt",
            Commands::LT => "lt",
            Commands::AND => "and",
            Commands::OR => "or",
            Commands::NOT => "not",
        };

        self.output_vm_file
            .write_all(format!("{arithmetic_command}\n").as_bytes())
    }

    pub fn write_label(&mut self, label_index: usize) -> Result<(), std::io::Error> {
        self.output_vm_file
            .write_all(format!("label {}_{label_index}\n", self.get_class_name()).as_bytes())?;
        Ok(())
    }

    pub fn write_goto(&mut self, label_index: usize) -> Result<(), std::io::Error> {
        self.output_vm_file
            .write_all(format!("goto {}_{label_index}\n", self.get_class_name()).as_bytes())
    }

    pub fn write_if(&mut self, label_index: usize) -> Result<(), std::io::Error> {
        self.output_vm_file
            .write_all(format!("if-goto {}_{label_index}\n", self.get_class_name()).as_bytes())?;
        Ok(())
    }

    pub fn write_call(
        &mut self,
        function_name: &str,
        arg_count: usize,
    ) -> Result<(), std::io::Error> {
        self.output_vm_file
            .write_all(format!("call {function_name} {arg_count}\n").as_bytes())
    }

    pub fn write_return(&mut self) -> Result<(), std::io::Error> {
        self.output_vm_file.write_all("return\n".as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use core::panic;
    use std::{fs};

    use super::*;


    #[test]
    fn validate_writes() {
        let dummy_class_name = String::from("dummy_output");
        let dummy_output_file = File::create(format!("{dummy_class_name}.vm"));
        if let Ok(dummy_output_file) = dummy_output_file {
            let mut correct_output = Vec::new();
            let dummy_output_file = LineWriter::new(dummy_output_file);
            let mut dummy_writer = VmWriter::new(dummy_output_file,dummy_class_name.clone());
            assert!(dummy_writer.write_push(&VMSegment::LOCAL,0).is_ok());
            correct_output.push("push local 0");
            assert!(dummy_writer.write_pop(&VMSegment::STATIC,0).is_ok());
            correct_output.push("pop static 0");
            assert!(dummy_writer.write_return().is_ok());
            correct_output.push("return");
            assert!(dummy_writer.write_arithmetic(&Commands::EQ).is_ok());
            correct_output.push("eq");
            assert!(dummy_writer.write_call("test_function", 2).is_ok());
            correct_output.push("call test_function 2");
            assert!(dummy_writer.write_function("test_function", 1).is_ok());
            correct_output.push("function dummy_output.test_function 1");
            assert!(dummy_writer.write_goto(0).is_ok());
            correct_output.push("goto dummy_output_0");
            assert!(dummy_writer.write_if(1).is_ok());
            correct_output.push("if-goto dummy_output_1");
            assert!(dummy_writer.write_label(3).is_ok());
            correct_output.push("label dummy_output_3");

            let dummy_output_string = fs::read_to_string(format!("{}.vm",dummy_class_name));
            if let Ok(dummy_output_string) = dummy_output_string {
                let mut vec_index = 0;
                for curr_line in dummy_output_string.lines() {
                    assert_eq!(*curr_line,**correct_output.get(vec_index).unwrap());
                    vec_index+=1;
                }

                assert!(fs::remove_file(format!("{}.vm",dummy_class_name)).is_ok());
            } else {
                panic!("Test validate_writes parsing dummy_output_file failed");
            }
        } else {
            panic!("Test validate_writes creation of dummy_output_file failed");
        }
        
        
    }

}
