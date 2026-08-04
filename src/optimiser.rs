use crate::ast::Node;

pub fn optimize(program: Vec<Node>) -> Vec<Node> {
    optimize_nodes(program)
}

fn optimize_nodes(nodes: Vec<Node>) -> Vec<Node> {
    let mut out = Vec::new();

    for node in aggregate(nodes) {
        match node {
            Node::Loop(body) => {
                let body = optimize_nodes(body);

                if is_clear_loop(&body) {
                    out.push(Node::Clear);
                } else if let Some(transfer) = recognize_transfer(&body) {
                    out.push(transfer);
                } else {
                    out.push(Node::Loop(body));
                }
            }

            Node::Add(delta) if delta == 0 => {}
            Node::Move(delta) if delta == 0 => {}

            node => out.push(node),
        }
    }

    aggregate(out)
}

fn aggregate(nodes: Vec<Node>) -> Vec<Node> {
    let mut out = Vec::new();

    for node in nodes {
        match node {
            Node::Add(delta) => push_add(&mut out, delta),
            Node::Move(delta) => push_move(&mut out, delta),
            other => out.push(other),
        }
    }

    out
}

fn push_add(out: &mut Vec<Node>, delta: i64) {
    if delta == 0 {
        return;
    }

    if let Some(Node::Add(current)) = out.last_mut() {
        let sum = *current + delta;
        if sum == 0 {
            out.pop();
        } else {
            *current = sum;
        }
    } else {
        out.push(Node::Add(delta));
    }
}

fn push_move(out: &mut Vec<Node>, delta: i64) {
    if delta == 0 {
        return;
    }

    if let Some(Node::Move(current)) = out.last_mut() {
        let sum = *current + delta;
        if sum == 0 {
            out.pop();
        } else {
            *current = sum;
        }
    } else {
        out.push(Node::Move(delta));
    }
}

fn is_clear_loop(body: &[Node]) -> bool {
    if body.len() != 1 {
        return false;
    }

    match &body[0] {
        Node::Add(delta) => *delta == -1,
        Node::Clear => true,
        _ => false,
    }
}

fn recognize_transfer(body: &[Node]) -> Option<Node> {
    if body.len() != 4 {
        return None;
    }

    match (&body[0], &body[1], &body[2], &body[3]) {
        (
            Node::Add(dec),
            Node::Move(offset),
            Node::Add(multiplier),
            Node::Move(back),
        ) if *dec == -1 && *offset != 0 && *back == -*offset => {
            Some(make_transfer(*offset, *multiplier))
        }

        (
            Node::Move(offset),
            Node::Add(multiplier),
            Node::Move(back),
            Node::Add(dec),
        ) if *dec == -1 && *offset != 0 && *back == -*offset => {
            Some(make_transfer(*offset, *multiplier))
        }

        _ => None,
    }
}

fn make_transfer(offset: i64, multiplier: i64) -> Node {
    if multiplier == 0 {
        Node::Clear
    } else {
        Node::Transfer {
            offset,
            multiplier,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Node;
    use crate::lexer::lex;
    use crate::parser::parse;

    fn opt(source: &str) -> Vec<Node> {
        let tokens = lex(source);
        let ast = parse(&tokens).unwrap();
        optimize(ast)
    }

    #[test]
    fn aggregates_increments() {
        assert_eq!(opt("+++"), vec![Node::Add(3)]);
    }

    #[test]
    fn aggregates_decrements() {
        assert_eq!(opt("---"), vec![Node::Add(-3)]);
    }

    #[test]
    fn cancels_moves() {
        assert_eq!(opt("><"), vec![]);
    }

    #[test]
    fn cancels_adds() {
        assert_eq!(opt("+-"), vec![]);
    }

    #[test]
    fn optimizes_clear_loop() {
        assert_eq!(opt("[-]"), vec![Node::Clear]);
    }

    #[test]
    fn optimizes_nested_clear_loop() {
        assert_eq!(opt("[[-]]"), vec![Node::Clear]);
    }

    #[test]
    fn optimizes_transfer_right() {
        assert_eq!(
            opt("[->+<]"),
            vec![Node::Transfer {
                offset: 1,
                multiplier: 1
            }]
        );
    }

    #[test]
    fn optimizes_transfer_left() {
        assert_eq!(
            opt("[<+>-]"),
            vec![Node::Transfer {
                offset: -1,
                multiplier: 1
            }]
        );
    }

    #[test]
    fn optimizes_negative_transfer() {
        assert_eq!(
            opt("[->-<]"),
            vec![Node::Transfer {
                offset: 1,
                multiplier: -1
            }]
        );
    }

    #[test]
    fn optimizes_multiplied_transfer() {
        assert_eq!(
            opt("[->+++<]"),
            vec![Node::Transfer {
                offset: 1,
                multiplier: 3
            }]
        );
    }

    #[test]
    fn does_not_misoptimize_put_loop() {
        assert_eq!(opt("[.]"), vec![Node::Loop(vec![Node::Put])]);
    }
}
