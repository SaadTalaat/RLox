use super::Expr;
use super::RDParser;
use crate::lex::Scanner;

#[test]
fn test_parse_expr_types() -> super::Result<()> {
    let tokens = Scanner::scan(b"1 + 2;").unwrap();
    let mut parser = RDParser::new(&tokens);
    let exprs = parser.parse()?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));

    let tokens = Scanner::scan(b"1 - 2;").unwrap();
    let mut parser = RDParser::new(&tokens);
    let exprs = parser.parse()?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));

    let tokens = Scanner::scan(b"1 * 2;").unwrap();
    let mut parser = RDParser::new(&tokens);
    let exprs = parser.parse()?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));

    let tokens = Scanner::scan(b"1 / 2;").unwrap();
    let mut parser = RDParser::new(&tokens);
    let exprs = parser.parse()?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));

    let tokens = Scanner::scan(b"1 % 2;").unwrap();
    let mut parser = RDParser::new(&tokens);
    let exprs = parser.parse()?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));

    let tokens = Scanner::scan(b"- 1;").unwrap();
    let mut parser = RDParser::new(&tokens);
    let exprs = parser.parse()?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Unary { .. }));

    let tokens = Scanner::scan(b"!1;").unwrap();
    let mut parser = RDParser::new(&tokens);
    let exprs = parser.parse()?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Unary { .. }));

    let tokens = Scanner::scan(b"(1 + 2);").unwrap();
    let mut parser = RDParser::new(&tokens);
    let exprs = parser.parse()?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Grouping { .. }));

    let tokens = Scanner::scan(b"\"hello world\";").unwrap();
    let mut parser = RDParser::new(&tokens);
    let exprs = parser.parse()?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Literal { .. }));
    Ok(())
}

#[test]
fn test_unary_recursive_right_associative() -> super::Result<()> {
    let tokens = Scanner::scan(b"!!true;").unwrap();
    let mut parser = RDParser::new(&tokens);
    let exprs = parser.parse()?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Unary { .. }));
    assert_eq!("(! (! true))", format!("{}", exprs[0]));

    let tokens = Scanner::scan(b"!-1;").unwrap();
    let mut parser = RDParser::new(&tokens);
    let exprs = parser.parse()?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Unary { .. }));
    assert_eq!("(! (- 1))", format!("{}", exprs[0]));

    let tokens = Scanner::scan(b"!-!!-1;").unwrap();
    let mut parser = RDParser::new(&tokens);
    let exprs = parser.parse()?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Unary { .. }));
    assert_eq!("(! (- (! (! (- 1)))))", format!("{}", exprs[0]));
    Ok(())
}

#[test]
fn test_factor_recursive_left_associative() -> super::Result<()> {
    let tokens = Scanner::scan(b"1 * 2 / 4;").unwrap();
    let mut parser = RDParser::new(&tokens);
    let exprs = parser.parse()?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));
    assert_eq!("(/ (* 1 2) 4)", format!("{}", exprs[0]));

    let tokens = Scanner::scan(b"1 / 2 * 4;").unwrap();
    let mut parser = RDParser::new(&tokens);
    let exprs = parser.parse()?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));
    assert_eq!("(* (/ 1 2) 4)", format!("{}", exprs[0]));

    let tokens = Scanner::scan(b"1 / -2 * 4;").unwrap();
    let mut parser = RDParser::new(&tokens);
    let exprs = parser.parse()?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));
    assert_eq!("(* (/ 1 (- 2)) 4)", format!("{}", exprs[0]));

    Ok(())
}

#[test]
fn test_term_recursive_left_associative() -> super::Result<()> {
    let tokens = Scanner::scan(b"1 - 2 + 4;").unwrap();
    let mut parser = RDParser::new(&tokens);
    let exprs = parser.parse()?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));
    assert_eq!("(+ (- 1 2) 4)", format!("{}", exprs[0]));

    let tokens = Scanner::scan(b"1 + 2 - 4;").unwrap();
    let mut parser = RDParser::new(&tokens);
    let exprs = parser.parse()?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));
    assert_eq!("(- (+ 1 2) 4)", format!("{}", exprs[0]));

    let tokens = Scanner::scan(b"1 - -2 + 4;").unwrap();
    let mut parser = RDParser::new(&tokens);
    let exprs = parser.parse()?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));
    assert_eq!("(+ (- 1 (- 2)) 4)", format!("{}", exprs[0]));
    Ok(())
}

#[test]
fn test_term_precedence() -> super::Result<()> {
    let tokens = Scanner::scan(b"1 - 2 * 4;").unwrap();
    let mut parser = RDParser::new(&tokens);
    let exprs = parser.parse()?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));
    assert_eq!("(- 1 (* 2 4))", format!("{}", exprs[0]));

    let tokens = Scanner::scan(b"1 + 2 / 4;").unwrap();
    let mut parser = RDParser::new(&tokens);
    let exprs = parser.parse()?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));
    assert_eq!("(+ 1 (/ 2 4))", format!("{}", exprs[0]));

    let tokens = Scanner::scan(b"1 - -2 * 4;").unwrap();
    let mut parser = RDParser::new(&tokens);
    let exprs = parser.parse()?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));
    assert_eq!("(- 1 (* (- 2) 4))", format!("{}", exprs[0]));

    let tokens = Scanner::scan(b"1 - -2 % 4;").unwrap();
    let mut parser = RDParser::new(&tokens);
    let exprs = parser.parse()?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));
    assert_eq!("(- 1 (% (- 2) 4))", format!("{}", exprs[0]));

    Ok(())
}

#[test]
fn test_comparison_left_associative() -> super::Result<()> {
    let tokens = Scanner::scan(b"1 > 2 > 3;").unwrap();
    let mut parser = RDParser::new(&tokens);
    let exprs = parser.parse()?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));
    assert_eq!("(- 1 (% (- 2) 4))", format!("{}", exprs[0]));

    Ok(())
}

#[test]
#[should_panic(expected = "expected: SemiColon, found: Number")]
fn test_illegal_literal_after_expr() {
    let tokens = Scanner::scan(b"1 + 2 3;").unwrap();
    let mut parser = RDParser::new(&tokens);
    parser.parse().unwrap();
}

#[test]
#[should_panic(expected = "expected: SemiColon, found: EOF")]
fn test_missing_semicolon_illegal_literal_after_expr() {
    let tokens = Scanner::scan(b"1 + 2").unwrap();
    let mut parser = RDParser::new(&tokens);
    parser.parse().unwrap();
}

#[test]
#[should_panic(expected = "expected: RightParen, found: SemiColon")]
fn test_missing_right_parentheses() {
    let tokens = Scanner::scan(b"(((1 + 2));").unwrap();
    let mut parser = RDParser::new(&tokens);
    parser.parse().unwrap();
}

#[test]
#[should_panic(expected = "Unexpected token: ;")]
fn test_missing_right_equality_operand() {
    let tokens = Scanner::scan(b"(1 + 2) == ;").unwrap();
    let mut parser = RDParser::new(&tokens);
    parser.parse().unwrap();
}

#[test]
#[should_panic(expected = "Unexpected token: ;")]
fn test_missing_right_minus_operand() {
    let tokens = Scanner::scan(b"(1 + 2) -;").unwrap();
    let mut parser = RDParser::new(&tokens);
    parser.parse().unwrap();
}

#[test]
#[should_panic(expected = "Unexpected token: ;")]
fn test_missing_right_star_operand() {
    let tokens = Scanner::scan(b"(1 + 2)* ;").unwrap();
    let mut parser = RDParser::new(&tokens);
    parser.parse().unwrap();
}

#[test]
#[should_panic(expected = "Unexpected token: ==")]
fn test_missing_left_equality_operand() {
    let tokens = Scanner::scan(b"== 1;").unwrap();
    let mut parser = RDParser::new(&tokens);
    parser.parse().unwrap();
}

#[test]
#[should_panic(expected = "Unexpected token: +")]
fn test_missing_left_plus_operand() {
    let tokens = Scanner::scan(b"+ 1;").unwrap();
    let mut parser = RDParser::new(&tokens);
    parser.parse().unwrap();
}
