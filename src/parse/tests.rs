use super::Expr;
use super::{ImmutableRDParser, RDParser};
use crate::lex::Scanner;

/// Testing RDParser
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
    assert_eq!("(> (> 1 2) 3)", format!("{}", exprs[0]));

    Ok(())
}

#[test]
fn test_ternary_operators() -> super::Result<()> {
    let tokens = Scanner::scan(b"1 == 2 ? 1 : 2;").unwrap();
    let mut parser = RDParser::new(&tokens);
    let exprs = parser.parse()?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Ternary { .. }));
    assert_eq!("((== 1 2) ? 1 : 2)", format!("{}", exprs[0]));

    let tokens = Scanner::scan(b"1 == 2 ? 1 : 2 ? 3:4;").unwrap();
    let mut parser = RDParser::new(&tokens);
    let exprs = parser.parse()?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Ternary { .. }));
    assert_eq!("((== 1 2) ? 1 : (2 ? 3 : 4))", format!("{}", exprs[0]));

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

/// Testing ImmutableRDParser
#[test]
fn test_immutable_parse_expr_types() -> super::Result<()> {
    let tokens = Scanner::scan(b"1 + 2;").unwrap();
    let exprs = ImmutableRDParser::parse(&tokens)?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));

    let tokens = Scanner::scan(b"1 - 2;").unwrap();
    let exprs = ImmutableRDParser::parse(&tokens)?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));

    let tokens = Scanner::scan(b"1 * 2;").unwrap();
    let exprs = ImmutableRDParser::parse(&tokens)?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));

    let tokens = Scanner::scan(b"1 / 2;").unwrap();
    let exprs = ImmutableRDParser::parse(&tokens)?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));

    let tokens = Scanner::scan(b"1 % 2;").unwrap();
    let exprs = ImmutableRDParser::parse(&tokens)?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));

    let tokens = Scanner::scan(b"- 1;").unwrap();
    let exprs = ImmutableRDParser::parse(&tokens)?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Unary { .. }));

    let tokens = Scanner::scan(b"!1;").unwrap();
    let exprs = ImmutableRDParser::parse(&tokens)?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Unary { .. }));

    let tokens = Scanner::scan(b"(1 + 2);").unwrap();
    let exprs = ImmutableRDParser::parse(&tokens)?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Grouping { .. }));

    let tokens = Scanner::scan(b"\"hello world\";").unwrap();
    let exprs = ImmutableRDParser::parse(&tokens)?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Literal { .. }));
    Ok(())
}

#[test]
fn test_immutable_unary_recursive_right_associative() -> super::Result<()> {
    let tokens = Scanner::scan(b"!!true;").unwrap();
    let exprs = ImmutableRDParser::parse(&tokens)?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Unary { .. }));
    assert_eq!("(! (! true))", format!("{}", exprs[0]));

    let tokens = Scanner::scan(b"!-1;").unwrap();
    let exprs = ImmutableRDParser::parse(&tokens)?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Unary { .. }));
    assert_eq!("(! (- 1))", format!("{}", exprs[0]));

    let tokens = Scanner::scan(b"!-!!-1;").unwrap();
    let exprs = ImmutableRDParser::parse(&tokens)?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Unary { .. }));
    assert_eq!("(! (- (! (! (- 1)))))", format!("{}", exprs[0]));
    Ok(())
}

#[test]
fn test_immutable_factor_recursive_left_associative() -> super::Result<()> {
    let tokens = Scanner::scan(b"1 * 2 / 4;").unwrap();
    let exprs = ImmutableRDParser::parse(&tokens)?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));
    assert_eq!("(/ (* 1 2) 4)", format!("{}", exprs[0]));

    let tokens = Scanner::scan(b"1 / 2 * 4;").unwrap();
    let exprs = ImmutableRDParser::parse(&tokens)?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));
    assert_eq!("(* (/ 1 2) 4)", format!("{}", exprs[0]));

    let tokens = Scanner::scan(b"1 / -2 * 4;").unwrap();
    let exprs = ImmutableRDParser::parse(&tokens)?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));
    assert_eq!("(* (/ 1 (- 2)) 4)", format!("{}", exprs[0]));

    Ok(())
}

#[test]
fn test_immutable_term_recursive_left_associative() -> super::Result<()> {
    let tokens = Scanner::scan(b"1 - 2 + 4;").unwrap();
    let exprs = ImmutableRDParser::parse(&tokens)?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));
    assert_eq!("(+ (- 1 2) 4)", format!("{}", exprs[0]));

    let tokens = Scanner::scan(b"1 + 2 - 4;").unwrap();
    let exprs = ImmutableRDParser::parse(&tokens)?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));
    assert_eq!("(- (+ 1 2) 4)", format!("{}", exprs[0]));

    let tokens = Scanner::scan(b"1 - -2 + 4;").unwrap();
    let exprs = ImmutableRDParser::parse(&tokens)?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));
    assert_eq!("(+ (- 1 (- 2)) 4)", format!("{}", exprs[0]));
    Ok(())
}

#[test]
fn test_immutable_term_precedence() -> super::Result<()> {
    let tokens = Scanner::scan(b"1 - 2 * 4;").unwrap();
    let exprs = ImmutableRDParser::parse(&tokens)?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));
    assert_eq!("(- 1 (* 2 4))", format!("{}", exprs[0]));

    let tokens = Scanner::scan(b"1 + 2 / 4;").unwrap();
    let exprs = ImmutableRDParser::parse(&tokens)?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));
    assert_eq!("(+ 1 (/ 2 4))", format!("{}", exprs[0]));

    let tokens = Scanner::scan(b"1 - -2 * 4;").unwrap();
    let exprs = ImmutableRDParser::parse(&tokens)?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));
    assert_eq!("(- 1 (* (- 2) 4))", format!("{}", exprs[0]));

    let tokens = Scanner::scan(b"1 - -2 % 4;").unwrap();
    let exprs = ImmutableRDParser::parse(&tokens)?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));
    assert_eq!("(- 1 (% (- 2) 4))", format!("{}", exprs[0]));

    Ok(())
}

#[test]
fn test_immutable_comparison_left_associative() -> super::Result<()> {
    let tokens = Scanner::scan(b"1 > 2 > 3;").unwrap();
    let exprs = ImmutableRDParser::parse(&tokens)?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Binary { .. }));
    assert_eq!("(> (> 1 2) 3)", format!("{}", exprs[0]));

    Ok(())
}

#[test]
fn test_immutable_ternary_operators() -> super::Result<()> {
    let tokens = Scanner::scan(b"1 == 2 ? 1 : 2;").unwrap();
    let exprs = ImmutableRDParser::parse(&tokens)?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Ternary { .. }));
    assert_eq!("((== 1 2) ? 1 : 2)", format!("{}", exprs[0]));

    let tokens = Scanner::scan(b"1 == 2 ? 1 : 2 ? 3:4;").unwrap();
    let exprs = ImmutableRDParser::parse(&tokens)?;
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::Ternary { .. }));
    assert_eq!("((== 1 2) ? 1 : (2 ? 3 : 4))", format!("{}", exprs[0]));

    Ok(())
}

#[test]
#[should_panic(expected = "expected: SemiColon, found: Number")]
fn test_immutable_illegal_literal_after_expr() {
    let tokens = Scanner::scan(b"1 + 2 3;").unwrap();
    ImmutableRDParser::parse(&tokens).unwrap();
}

#[test]
#[should_panic(expected = "expected: SemiColon, found: EOF")]
fn test_immutable_missing_semicolon_illegal_literal_after_expr() {
    let tokens = Scanner::scan(b"1 + 2").unwrap();
    ImmutableRDParser::parse(&tokens).unwrap();
}

#[test]
#[should_panic(expected = "expected: RightParen, found: SemiColon")]
fn test_immutable_missing_right_parentheses() {
    let tokens = Scanner::scan(b"(((1 + 2));").unwrap();
    ImmutableRDParser::parse(&tokens).unwrap();
}

#[test]
#[should_panic(expected = "Unexpected token: ;")]
fn test_immutable_missing_right_equality_operand() {
    let tokens = Scanner::scan(b"(1 + 2) == ;").unwrap();
    ImmutableRDParser::parse(&tokens).unwrap();
}

#[test]
#[should_panic(expected = "Unexpected token: ;")]
fn test_immutable_missing_right_minus_operand() {
    let tokens = Scanner::scan(b"(1 + 2) -;").unwrap();
    ImmutableRDParser::parse(&tokens).unwrap();
}

#[test]
#[should_panic(expected = "Unexpected token: ;")]
fn test_immutable_missing_right_star_operand() {
    let tokens = Scanner::scan(b"(1 + 2)* ;").unwrap();
    ImmutableRDParser::parse(&tokens).unwrap();
}

#[test]
#[should_panic(expected = "Unexpected token: ==")]
fn test_immutable_missing_left_equality_operand() {
    let tokens = Scanner::scan(b"== 1;").unwrap();
    ImmutableRDParser::parse(&tokens).unwrap();
}

#[test]
#[should_panic(expected = "Unexpected token: +")]
fn test_immutable_missing_left_plus_operand() {
    let tokens = Scanner::scan(b"+ 1;").unwrap();
    ImmutableRDParser::parse(&tokens).unwrap();
}
