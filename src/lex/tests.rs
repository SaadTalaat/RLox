use super::token::TokenType;
use super::Scanner;

#[test]
fn test_scanner_single_char_token_types() {
    let tokens = Scanner::scan(b"(){},.-+/;*").unwrap();
    assert_eq!(tokens.len(), 12);
    assert_eq!(*tokens[0].token_type(), TokenType::LeftParen);
    assert_eq!(*tokens[0].lexeme(), String::from("("));
    assert_eq!(*tokens[1].token_type(), TokenType::RightParen);
    assert_eq!(*tokens[1].lexeme(), String::from(")"));
    assert_eq!(*tokens[2].token_type(), TokenType::LeftBrace);
    assert_eq!(*tokens[2].lexeme(), String::from("{"));
    assert_eq!(*tokens[3].token_type(), TokenType::RightBrace);
    assert_eq!(*tokens[3].lexeme(), String::from("}"));
    assert_eq!(*tokens[4].token_type(), TokenType::Comma);
    assert_eq!(*tokens[4].lexeme(), String::from(","));
    assert_eq!(*tokens[5].token_type(), TokenType::Dot);
    assert_eq!(*tokens[5].lexeme(), String::from("."));
    assert_eq!(*tokens[6].token_type(), TokenType::Minus);
    assert_eq!(*tokens[6].lexeme(), String::from("-"));
    assert_eq!(*tokens[7].token_type(), TokenType::Plus);
    assert_eq!(*tokens[7].lexeme(), String::from("+"));
    assert_eq!(*tokens[8].token_type(), TokenType::Slash);
    assert_eq!(*tokens[8].lexeme(), String::from("/"));
    assert_eq!(*tokens[9].token_type(), TokenType::SemiColon);
    assert_eq!(*tokens[9].lexeme(), String::from(";"));
    assert_eq!(*tokens[10].token_type(), TokenType::Star);
    assert_eq!(*tokens[10].lexeme(), String::from("*"));
    assert_eq!(*tokens[11].token_type(), TokenType::EOF);
    assert_eq!(*tokens[11].lexeme(), String::from("\0"));

    let tokens = Scanner::scan(b"(\n)\n{\n}\n,\n.\n\n-\n+\n;\n/\n*").unwrap();
    assert_eq!(tokens.len(), 12);
    assert_eq!(*tokens[0].token_type(), TokenType::LeftParen);
    assert_eq!(*tokens[0].lexeme(), String::from("("));
    assert_eq!(*tokens[1].token_type(), TokenType::RightParen);
    assert_eq!(*tokens[1].lexeme(), String::from(")"));
    assert_eq!(*tokens[2].token_type(), TokenType::LeftBrace);
    assert_eq!(*tokens[2].lexeme(), String::from("{"));
    assert_eq!(*tokens[3].token_type(), TokenType::RightBrace);
    assert_eq!(*tokens[3].lexeme(), String::from("}"));
    assert_eq!(*tokens[4].token_type(), TokenType::Comma);
    assert_eq!(*tokens[4].lexeme(), String::from(","));
    assert_eq!(*tokens[5].token_type(), TokenType::Dot);
    assert_eq!(*tokens[5].lexeme(), String::from("."));
    assert_eq!(*tokens[6].token_type(), TokenType::Minus);
    assert_eq!(*tokens[6].lexeme(), String::from("-"));
    assert_eq!(*tokens[7].token_type(), TokenType::Plus);
    assert_eq!(*tokens[7].lexeme(), String::from("+"));
    assert_eq!(*tokens[8].token_type(), TokenType::SemiColon);
    assert_eq!(*tokens[8].lexeme(), String::from(";"));
    assert_eq!(*tokens[9].token_type(), TokenType::Slash);
    assert_eq!(*tokens[9].lexeme(), String::from("/"));
    assert_eq!(*tokens[10].token_type(), TokenType::Star);
    assert_eq!(*tokens[10].lexeme(), String::from("*"));
    assert_eq!(*tokens[11].token_type(), TokenType::EOF);
    assert_eq!(*tokens[11].lexeme(), String::from("\0"));

    let tokens = Scanner::scan(b"(  ) {  } , . - + ; / *").unwrap();
    assert_eq!(tokens.len(), 12);
    assert_eq!(*tokens[0].token_type(), TokenType::LeftParen);
    assert_eq!(*tokens[0].lexeme(), String::from("("));
    assert_eq!(*tokens[1].token_type(), TokenType::RightParen);
    assert_eq!(*tokens[1].lexeme(), String::from(")"));
    assert_eq!(*tokens[2].token_type(), TokenType::LeftBrace);
    assert_eq!(*tokens[2].lexeme(), String::from("{"));
    assert_eq!(*tokens[3].token_type(), TokenType::RightBrace);
    assert_eq!(*tokens[3].lexeme(), String::from("}"));
    assert_eq!(*tokens[4].token_type(), TokenType::Comma);
    assert_eq!(*tokens[4].lexeme(), String::from(","));
    assert_eq!(*tokens[5].token_type(), TokenType::Dot);
    assert_eq!(*tokens[5].lexeme(), String::from("."));
    assert_eq!(*tokens[6].token_type(), TokenType::Minus);
    assert_eq!(*tokens[6].lexeme(), String::from("-"));
    assert_eq!(*tokens[7].token_type(), TokenType::Plus);
    assert_eq!(*tokens[7].lexeme(), String::from("+"));
    assert_eq!(*tokens[8].token_type(), TokenType::SemiColon);
    assert_eq!(*tokens[8].lexeme(), String::from(";"));
    assert_eq!(*tokens[9].token_type(), TokenType::Slash);
    assert_eq!(*tokens[9].lexeme(), String::from("/"));
    assert_eq!(*tokens[10].token_type(), TokenType::Star);
    assert_eq!(*tokens[10].lexeme(), String::from("*"));
    assert_eq!(*tokens[11].token_type(), TokenType::EOF);
    assert_eq!(*tokens[11].lexeme(), String::from("\0"));

    let tokens = Scanner::scan(b"(\t)\t{\t}\t,\t.\t-\t+\t;\t/\t*").unwrap();
    assert_eq!(tokens.len(), 12);
    assert_eq!(*tokens[0].token_type(), TokenType::LeftParen);
    assert_eq!(*tokens[0].lexeme(), String::from("("));
    assert_eq!(*tokens[1].token_type(), TokenType::RightParen);
    assert_eq!(*tokens[1].lexeme(), String::from(")"));
    assert_eq!(*tokens[2].token_type(), TokenType::LeftBrace);
    assert_eq!(*tokens[2].lexeme(), String::from("{"));
    assert_eq!(*tokens[3].token_type(), TokenType::RightBrace);
    assert_eq!(*tokens[3].lexeme(), String::from("}"));
    assert_eq!(*tokens[4].token_type(), TokenType::Comma);
    assert_eq!(*tokens[4].lexeme(), String::from(","));
    assert_eq!(*tokens[5].token_type(), TokenType::Dot);
    assert_eq!(*tokens[5].lexeme(), String::from("."));
    assert_eq!(*tokens[6].token_type(), TokenType::Minus);
    assert_eq!(*tokens[6].lexeme(), String::from("-"));
    assert_eq!(*tokens[7].token_type(), TokenType::Plus);
    assert_eq!(*tokens[7].lexeme(), String::from("+"));
    assert_eq!(*tokens[8].token_type(), TokenType::SemiColon);
    assert_eq!(*tokens[8].lexeme(), String::from(";"));
    assert_eq!(*tokens[9].token_type(), TokenType::Slash);
    assert_eq!(*tokens[9].lexeme(), String::from("/"));
    assert_eq!(*tokens[10].token_type(), TokenType::Star);
    assert_eq!(*tokens[10].lexeme(), String::from("*"));
    assert_eq!(*tokens[11].token_type(), TokenType::EOF);
    assert_eq!(*tokens[11].lexeme(), String::from("\0"));

    let tokens = Scanner::scan(b"\r(\r)\r{\r}\r,\r.\r-\r+\r;\r/\r*").unwrap();
    assert_eq!(tokens.len(), 12);
    assert_eq!(*tokens[0].token_type(), TokenType::LeftParen);
    assert_eq!(*tokens[0].lexeme(), String::from("("));
    assert_eq!(*tokens[1].token_type(), TokenType::RightParen);
    assert_eq!(*tokens[1].lexeme(), String::from(")"));
    assert_eq!(*tokens[2].token_type(), TokenType::LeftBrace);
    assert_eq!(*tokens[2].lexeme(), String::from("{"));
    assert_eq!(*tokens[3].token_type(), TokenType::RightBrace);
    assert_eq!(*tokens[3].lexeme(), String::from("}"));
    assert_eq!(*tokens[4].token_type(), TokenType::Comma);
    assert_eq!(*tokens[4].lexeme(), String::from(","));
    assert_eq!(*tokens[5].token_type(), TokenType::Dot);
    assert_eq!(*tokens[5].lexeme(), String::from("."));
    assert_eq!(*tokens[6].token_type(), TokenType::Minus);
    assert_eq!(*tokens[6].lexeme(), String::from("-"));
    assert_eq!(*tokens[7].token_type(), TokenType::Plus);
    assert_eq!(*tokens[7].lexeme(), String::from("+"));
    assert_eq!(*tokens[8].token_type(), TokenType::SemiColon);
    assert_eq!(*tokens[8].lexeme(), String::from(";"));
    assert_eq!(*tokens[9].token_type(), TokenType::Slash);
    assert_eq!(*tokens[9].lexeme(), String::from("/"));
    assert_eq!(*tokens[10].token_type(), TokenType::Star);
    assert_eq!(*tokens[10].lexeme(), String::from("*"));
    assert_eq!(*tokens[11].token_type(), TokenType::EOF);
    assert_eq!(*tokens[11].lexeme(), String::from("\0"));
}

#[test]
fn test_scanner_single_conditional_token_types() {
    let tokens = Scanner::scan(b"==!==!>>=<<=").unwrap();
    assert_eq!(tokens.len(), 9);
    assert_eq!(*tokens[0].token_type(), TokenType::EqEq);
    assert_eq!(*tokens[0].lexeme(), String::from("=="));
    assert_eq!(*tokens[1].token_type(), TokenType::BangEq);
    assert_eq!(*tokens[1].lexeme(), String::from("!="));
    assert_eq!(*tokens[2].token_type(), TokenType::Equal);
    assert_eq!(*tokens[2].lexeme(), String::from("="));
    assert_eq!(*tokens[3].token_type(), TokenType::Bang);
    assert_eq!(*tokens[3].lexeme(), String::from("!"));
    assert_eq!(*tokens[4].token_type(), TokenType::GreaterThan);
    assert_eq!(*tokens[4].lexeme(), String::from(">"));
    assert_eq!(*tokens[5].token_type(), TokenType::GreaterThanEq);
    assert_eq!(*tokens[5].lexeme(), String::from(">="));
    assert_eq!(*tokens[6].token_type(), TokenType::LessThan);
    assert_eq!(*tokens[6].lexeme(), String::from("<"));
    assert_eq!(*tokens[7].token_type(), TokenType::LessThanEq);
    assert_eq!(*tokens[7].lexeme(), String::from("<="));
    assert_eq!(*tokens[8].token_type(), TokenType::EOF);
    assert_eq!(*tokens[8].lexeme(), String::from("\0"));

    let tokens = Scanner::scan(b"== != = ! > >= < <=").unwrap();
    assert_eq!(tokens.len(), 9);
    assert_eq!(*tokens[0].token_type(), TokenType::EqEq);
    assert_eq!(*tokens[0].lexeme(), String::from("=="));
    assert_eq!(*tokens[1].token_type(), TokenType::BangEq);
    assert_eq!(*tokens[1].lexeme(), String::from("!="));
    assert_eq!(*tokens[2].token_type(), TokenType::Equal);
    assert_eq!(*tokens[2].lexeme(), String::from("="));
    assert_eq!(*tokens[3].token_type(), TokenType::Bang);
    assert_eq!(*tokens[3].lexeme(), String::from("!"));
    assert_eq!(*tokens[4].token_type(), TokenType::GreaterThan);
    assert_eq!(*tokens[4].lexeme(), String::from(">"));
    assert_eq!(*tokens[5].token_type(), TokenType::GreaterThanEq);
    assert_eq!(*tokens[5].lexeme(), String::from(">="));
    assert_eq!(*tokens[6].token_type(), TokenType::LessThan);
    assert_eq!(*tokens[6].lexeme(), String::from("<"));
    assert_eq!(*tokens[7].token_type(), TokenType::LessThanEq);
    assert_eq!(*tokens[7].lexeme(), String::from("<="));
    assert_eq!(*tokens[8].token_type(), TokenType::EOF);
    assert_eq!(*tokens[8].lexeme(), String::from("\0"));

    let tokens = Scanner::scan(b"==\r!=\r=\r!\r>\r>=\r<\r<=\r").unwrap();
    assert_eq!(tokens.len(), 9);
    assert_eq!(*tokens[0].token_type(), TokenType::EqEq);
    assert_eq!(*tokens[0].lexeme(), String::from("=="));
    assert_eq!(*tokens[1].token_type(), TokenType::BangEq);
    assert_eq!(*tokens[1].lexeme(), String::from("!="));
    assert_eq!(*tokens[2].token_type(), TokenType::Equal);
    assert_eq!(*tokens[2].lexeme(), String::from("="));
    assert_eq!(*tokens[3].token_type(), TokenType::Bang);
    assert_eq!(*tokens[3].lexeme(), String::from("!"));
    assert_eq!(*tokens[4].token_type(), TokenType::GreaterThan);
    assert_eq!(*tokens[4].lexeme(), String::from(">"));
    assert_eq!(*tokens[5].token_type(), TokenType::GreaterThanEq);
    assert_eq!(*tokens[5].lexeme(), String::from(">="));
    assert_eq!(*tokens[6].token_type(), TokenType::LessThan);
    assert_eq!(*tokens[6].lexeme(), String::from("<"));
    assert_eq!(*tokens[7].token_type(), TokenType::LessThanEq);
    assert_eq!(*tokens[7].lexeme(), String::from("<="));
    assert_eq!(*tokens[8].token_type(), TokenType::EOF);
    assert_eq!(*tokens[8].lexeme(), String::from("\0"));

    let tokens = Scanner::scan(b"==\n!=\n=\n!\n>\n>=\n<\n<=\n").unwrap();
    assert_eq!(tokens.len(), 9);
    assert_eq!(*tokens[0].token_type(), TokenType::EqEq);
    assert_eq!(*tokens[0].lexeme(), String::from("=="));
    assert_eq!(*tokens[1].token_type(), TokenType::BangEq);
    assert_eq!(*tokens[1].lexeme(), String::from("!="));
    assert_eq!(*tokens[2].token_type(), TokenType::Equal);
    assert_eq!(*tokens[2].lexeme(), String::from("="));
    assert_eq!(*tokens[3].token_type(), TokenType::Bang);
    assert_eq!(*tokens[3].lexeme(), String::from("!"));
    assert_eq!(*tokens[4].token_type(), TokenType::GreaterThan);
    assert_eq!(*tokens[4].lexeme(), String::from(">"));
    assert_eq!(*tokens[5].token_type(), TokenType::GreaterThanEq);
    assert_eq!(*tokens[5].lexeme(), String::from(">="));
    assert_eq!(*tokens[6].token_type(), TokenType::LessThan);
    assert_eq!(*tokens[6].lexeme(), String::from("<"));
    assert_eq!(*tokens[7].token_type(), TokenType::LessThanEq);
    assert_eq!(*tokens[7].lexeme(), String::from("<="));
    assert_eq!(*tokens[8].token_type(), TokenType::EOF);
    assert_eq!(*tokens[8].lexeme(), String::from("\0"));
}

#[test]
fn test_scanner_number_token_types() {
    let tokens = Scanner::scan(b"0 .1 0. 1. 103.1 12.0 5 1.2.3").unwrap();
    assert_eq!(tokens.len(), 14);
    // 0
    assert_eq!(*tokens[0].token_type(), TokenType::Number);
    assert_eq!(*tokens[0].lexeme(), String::from("0"));
    // .
    assert_eq!(*tokens[1].token_type(), TokenType::Dot);
    assert_eq!(*tokens[1].lexeme(), String::from("."));
    // 1
    assert_eq!(*tokens[2].token_type(), TokenType::Number);
    assert_eq!(*tokens[2].lexeme(), String::from("1"));
    // 0
    assert_eq!(*tokens[3].token_type(), TokenType::Number);
    assert_eq!(*tokens[3].lexeme(), String::from("0"));
    // .
    assert_eq!(*tokens[4].token_type(), TokenType::Dot);
    assert_eq!(*tokens[4].lexeme(), String::from("."));
    // 1
    assert_eq!(*tokens[5].token_type(), TokenType::Number);
    assert_eq!(*tokens[5].lexeme(), String::from("1"));
    // .
    assert_eq!(*tokens[6].token_type(), TokenType::Dot);
    assert_eq!(*tokens[6].lexeme(), String::from("."));
    // 103.1
    assert_eq!(*tokens[7].token_type(), TokenType::Number);
    assert_eq!(*tokens[7].lexeme(), String::from("103.1"));
    // 12.0
    assert_eq!(*tokens[8].token_type(), TokenType::Number);
    assert_eq!(*tokens[8].lexeme(), String::from("12.0"));
    // 5
    assert_eq!(*tokens[9].token_type(), TokenType::Number);
    assert_eq!(*tokens[9].lexeme(), String::from("5"));
    // 1.2
    assert_eq!(*tokens[10].token_type(), TokenType::Number);
    assert_eq!(*tokens[10].lexeme(), String::from("1.2"));
    // .
    assert_eq!(*tokens[11].token_type(), TokenType::Dot);
    assert_eq!(*tokens[11].lexeme(), String::from("."));
    // 3
    assert_eq!(*tokens[12].token_type(), TokenType::Number);
    assert_eq!(*tokens[12].lexeme(), String::from("3"));
    // EOF
    assert_eq!(*tokens[13].token_type(), TokenType::EOF);
    assert_eq!(*tokens[13].lexeme(), String::from("\0"));
}

#[test]
fn test_keywords() {
    let tokens =
        Scanner::scan(b"and or while if else for true false nil print return super var class")
            .unwrap();
    assert_eq!(tokens.len(), 15);
    assert_eq!(*tokens[0].token_type(), TokenType::And);
    assert_eq!(*tokens[0].lexeme(), String::from("and"));
    assert_eq!(*tokens[1].token_type(), TokenType::Or);
    assert_eq!(*tokens[1].lexeme(), String::from("or"));
    assert_eq!(*tokens[2].token_type(), TokenType::While);
    assert_eq!(*tokens[2].lexeme(), String::from("while"));
    assert_eq!(*tokens[3].token_type(), TokenType::If);
    assert_eq!(*tokens[3].lexeme(), String::from("if"));
    assert_eq!(*tokens[4].token_type(), TokenType::Else);
    assert_eq!(*tokens[4].lexeme(), String::from("else"));
    assert_eq!(*tokens[5].token_type(), TokenType::For);
    assert_eq!(*tokens[5].lexeme(), String::from("for"));
    assert_eq!(*tokens[6].token_type(), TokenType::True);
    assert_eq!(*tokens[6].lexeme(), String::from("true"));
    assert_eq!(*tokens[7].token_type(), TokenType::False);
    assert_eq!(*tokens[7].lexeme(), String::from("false"));
    assert_eq!(*tokens[8].token_type(), TokenType::Nil);
    assert_eq!(*tokens[8].lexeme(), String::from("nil"));
    assert_eq!(*tokens[9].token_type(), TokenType::Print);
    assert_eq!(*tokens[9].lexeme(), String::from("print"));
    assert_eq!(*tokens[10].token_type(), TokenType::Return);
    assert_eq!(*tokens[10].lexeme(), String::from("return"));
    assert_eq!(*tokens[11].token_type(), TokenType::Super);
    assert_eq!(*tokens[11].lexeme(), String::from("super"));
    assert_eq!(*tokens[12].token_type(), TokenType::Var);
    assert_eq!(*tokens[12].lexeme(), String::from("var"));
    assert_eq!(*tokens[13].token_type(), TokenType::Class);
    assert_eq!(*tokens[13].lexeme(), String::from("class"));
    assert_eq!(*tokens[14].token_type(), TokenType::EOF);
    assert_eq!(*tokens[14].lexeme(), String::from("\0"));
}

#[test]
fn test_identifiers() {
    let tokens = Scanner::scan(
        b"and_ or_ while_ if_ else_ for_ true_ false_ nil_ print_ return_ super_ var_ class_",
    )
    .unwrap();

    assert_eq!(*tokens[0].token_type(), TokenType::Identifier);
    assert_eq!(*tokens[0].lexeme(), String::from("and_"));
    assert_eq!(*tokens[1].token_type(), TokenType::Identifier);
    assert_eq!(*tokens[1].lexeme(), String::from("or_"));
    assert_eq!(*tokens[2].token_type(), TokenType::Identifier);
    assert_eq!(*tokens[2].lexeme(), String::from("while_"));
    assert_eq!(*tokens[3].token_type(), TokenType::Identifier);
    assert_eq!(*tokens[3].lexeme(), String::from("if_"));
    assert_eq!(*tokens[4].token_type(), TokenType::Identifier);
    assert_eq!(*tokens[4].lexeme(), String::from("else_"));
    assert_eq!(*tokens[5].token_type(), TokenType::Identifier);
    assert_eq!(*tokens[5].lexeme(), String::from("for_"));
    assert_eq!(*tokens[6].token_type(), TokenType::Identifier);
    assert_eq!(*tokens[6].lexeme(), String::from("true_"));
    assert_eq!(*tokens[7].token_type(), TokenType::Identifier);
    assert_eq!(*tokens[7].lexeme(), String::from("false_"));
    assert_eq!(*tokens[8].token_type(), TokenType::Identifier);
    assert_eq!(*tokens[8].lexeme(), String::from("nil_"));
    assert_eq!(*tokens[9].token_type(), TokenType::Identifier);
    assert_eq!(*tokens[9].lexeme(), String::from("print_"));
    assert_eq!(*tokens[10].token_type(), TokenType::Identifier);
    assert_eq!(*tokens[10].lexeme(), String::from("return_"));
    assert_eq!(*tokens[11].token_type(), TokenType::Identifier);
    assert_eq!(*tokens[11].lexeme(), String::from("super_"));
    assert_eq!(*tokens[12].token_type(), TokenType::Identifier);
    assert_eq!(*tokens[12].lexeme(), String::from("var_"));
    assert_eq!(*tokens[13].token_type(), TokenType::Identifier);
    assert_eq!(*tokens[13].lexeme(), String::from("class_"));
    assert_eq!(*tokens[14].token_type(), TokenType::EOF);
    assert_eq!(*tokens[14].lexeme(), String::from("\0"));
}

#[test]
fn test_comments() {
    let tokens = Scanner::scan(b"// test me here\nabc").unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(*tokens[0].token_type(), TokenType::Identifier);
    assert_eq!(*tokens[1].token_type(), TokenType::EOF);

    let tokens = Scanner::scan(b"/* normal block comment */").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(*tokens[0].token_type(), TokenType::EOF);

    let tokens = Scanner::scan(b"/* /* nested comment */ */").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(*tokens[0].token_type(), TokenType::EOF);

    let tokens = Scanner::scan(b"/* /* /* even more nested comment */ */ */").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(*tokens[0].token_type(), TokenType::EOF);

    let tokens = Scanner::scan(b"/* /* unbalanced nested comment */ */ */").unwrap();
    assert_eq!(tokens.len(), 3);
    assert_eq!(*tokens[0].token_type(), TokenType::Star);
    assert_eq!(*tokens[1].token_type(), TokenType::Slash);
    assert_eq!(*tokens[2].token_type(), TokenType::EOF);
}
