use hack_assembler::lib::*;

#[test]
fn parse_clean_instructions() {
    let test_parser1 = AsmParser::new("abc hello // sad\n ".to_string());
    let test_parser2 = AsmParser::new("abc hello  ".to_string());
    let test_parser3 = AsmParser::new("\nabc hello // sad\n test".to_string());
    let cleaned_instructions1 = test_parser1.clean_instructions();
    let cleaned_instructions2 = test_parser2.clean_instructions();
    let cleaned_instructions3 = test_parser3.clean_instructions();

    assert_eq!("abc hello".to_string(), cleaned_instructions1);
    assert_eq!("abc hello".to_string(), cleaned_instructions2);
    assert_eq!("abc hello\ntest".to_string(),cleaned_instructions3);
}

#[test]
fn parse_symbol() {
    let test_symbol_a_instruction = "@poopy";
    let test_symbol_l_instruiction = "(poopy)";
    let test_parser = AsmParser::new("dummy".to_string());
    let symbol_a_instruction = test_parser.get_symbol(&test_symbol_a_instruction,&['@']);
    let symbol_l_instruction1 = test_parser.get_symbol(&test_symbol_l_instruiction,&['(',')']);
    let symbol_l_instruction2 = test_parser.get_symbol(&test_symbol_l_instruiction,&[')','(']);
    assert_eq!("poopy",symbol_a_instruction);
    assert_eq!("poopy",symbol_l_instruction1);
    assert_eq!("poopy",symbol_l_instruction2);
}

#[test]
fn parse_instruction_type() {
    let test_instruction_a = "@hello";
    let test_instruction_l = "(what)";
    let test_instruction_c = "MD=D+A";
    let test_parser = AsmParser::new("dummy".to_string());

    let a_instruction = test_parser.get_instruction_type(&test_instruction_a);
    let l_instruction = test_parser.get_instruction_type(&test_instruction_l);
    let c_instruction = test_parser.get_instruction_type(&test_instruction_c);

    assert_eq!(InstructionType::A, a_instruction);
    assert_eq!(InstructionType::L, l_instruction);
    assert_eq!(InstructionType::C, c_instruction);
}

#[test]
fn parse_instruction_field() {
    let c_instruction_test1 = "MD=D+A;JMP";
    let c_instruction_test2 = "D;JMP";
    let c_instruction_test3 = "D=A";
    let test_parser = AsmParser::new("dummy".to_string());
    let dest_field1 = test_parser.get_c_instruction_field(&c_instruction_test1, CInstructionField::Dest);
    let comp_field1= test_parser.get_c_instruction_field(&c_instruction_test1, CInstructionField::Comp);
    let jump_field1 = test_parser.get_c_instruction_field(&c_instruction_test1, CInstructionField::Jump);
    let dest_field2 = test_parser.get_c_instruction_field(&c_instruction_test2, CInstructionField::Dest);
    let comp_field2= test_parser.get_c_instruction_field(&c_instruction_test2, CInstructionField::Comp);
    let jump_field2 = test_parser.get_c_instruction_field(&c_instruction_test2, CInstructionField::Jump);
    let dest_field3 = test_parser.get_c_instruction_field(&c_instruction_test3, CInstructionField::Dest);
    let comp_field3= test_parser.get_c_instruction_field(&c_instruction_test3, CInstructionField::Comp);
    let jump_field3 = test_parser.get_c_instruction_field(&c_instruction_test3, CInstructionField::Jump);

    assert_eq!(Some("MD"), dest_field1);
    assert_eq!(Some("D+A"),comp_field1);
    assert_eq!(Some("JMP"),jump_field1);
    assert_eq!(None, dest_field2);
    assert_eq!(Some("D"), comp_field2);
    assert_eq!(Some("JMP"), jump_field2);
    assert_eq!(Some("D"),dest_field3);
    assert_eq!(Some("A"),comp_field3);
    assert_eq!(None,jump_field3);
}