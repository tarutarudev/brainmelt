#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Inc,
    Dec,
    Left,
    Right,
    Put,
    Get,
    LoopStart,
    LoopEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

pub fn lex(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut line = 1usize;
    let mut col = 0usize;

    for ch in source.chars() {
        if ch == '\n' {
            line += 1;
            col = 0;
            continue;
        }

        col += 1;

        if ch == '\r' {
            continue;
        }

        let kind = match ch {
            '+' => TokenKind::Inc,
            '-' => TokenKind::Dec,
            '<' => TokenKind::Left,
            '>' => TokenKind::Right,
            '.' => TokenKind::Put,
            ',' => TokenKind::Get,
            '[' => TokenKind::LoopStart,
            ']' => TokenKind::LoopEnd,
            _ => continue,
        };

        tokens.push(Token { kind, line, col });
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_basic_tokens() {
        let tokens = lex("+-><.,[]\n+");
        let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind).collect();

        assert_eq!(
            kinds,
            vec![
                TokenKind::Inc,
                TokenKind::Dec,
                TokenKind::Left,
                TokenKind::Right,
                TokenKind::Put,
                TokenKind::Get,
                TokenKind::LoopStart,
                TokenKind::LoopEnd,
                TokenKind::Inc,
            ]
        );

        assert_eq!(tokens.last().unwrap().line, 2);
        assert_eq!(tokens.last().unwrap().col, 1);
    }

    #[test]
    fn ignores_comments() {
        let tokens = lex("+++ hello +++");
        assert_eq!(tokens.len(), 6);
    }
}
