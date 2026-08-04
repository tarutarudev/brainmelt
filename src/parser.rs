use crate::ast::Node;
use crate::error::{CompileError, Result};
use crate::lexer::{Token, TokenKind};

pub fn parse(tokens: &[Token]) -> Result<Vec<Node>> {
    struct Frame {
        open_line: usize,
        open_col: usize,
        nodes: Vec<Node>,
    }

    let mut stack: Vec<Frame> = Vec::new();
    let mut current: Vec<Node> = Vec::new();

    for token in tokens {
        match token.kind {
            TokenKind::Inc => current.push(Node::Add(1)),
            TokenKind::Dec => current.push(Node::Add(-1)),
            TokenKind::Left => current.push(Node::Move(-1)),
            TokenKind::Right => current.push(Node::Move(1)),
            TokenKind::Put => current.push(Node::Put),
            TokenKind::Get => current.push(Node::Get),

            TokenKind::LoopStart => {
                let old = std::mem::take(&mut current);
                stack.push(Frame {
                    open_line: token.line,
                    open_col: token.col,
                    nodes: old,
                });
            }

            TokenKind::LoopEnd => {
                let frame = stack.pop().ok_or(CompileError::UnmatchedClose {
                    line: token.line,
                    col: token.col,
                })?;

                let mut parent = frame.nodes;
                parent.push(Node::Loop(current));
                current = parent;
            }
        }
    }

    if let Some(frame) = stack.pop() {
        return Err(CompileError::UnmatchedOpen {
            line: frame.open_line,
            col: frame.open_col,
        });
    }

    Ok(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Node;
    use crate::error::CompileError;
    use crate::lexer::lex;

    #[test]
    fn parse_simple_program() {
        let tokens = lex("+[-<]");
        let ast = parse(&tokens).unwrap();

        assert_eq!(
            ast,
            vec![
                Node::Add(1),
                Node::Loop(vec![Node::Add(-1), Node::Move(-1)]),
            ]
        );
    }

    #[test]
    fn parse_nested_loop() {
        let tokens = lex("[[]]");
        let ast = parse(&tokens).unwrap();

        assert_eq!(
            ast,
            vec![Node::Loop(vec![Node::Loop(vec![])])]
        );
    }

    #[test]
    fn detects_unmatched_close() {
        let err = parse(&lex("]")).unwrap_err();
        assert_eq!(
            err,
            CompileError::UnmatchedClose {
                line: 1,
                col: 1
            }
        );
    }

    #[test]
    fn detects_unmatched_open() {
        let err = parse(&lex("[")).unwrap_err();
        assert_eq!(
            err,
            CompileError::UnmatchedOpen {
                line: 1,
                col: 1
            }
        );
    }

    #[test]
    fn reports_line_and_column() {
        let err = parse(&lex("+\n]")).unwrap_err();
        assert_eq!(
            err,
            CompileError::UnmatchedClose {
                line: 2,
                col: 1
            }
        );
    }
}
