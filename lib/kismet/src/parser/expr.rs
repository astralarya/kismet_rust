use nom::{combinator::opt, sequence::tuple as nom_tuple, Err};

use crate::ast::{Expr, Primary};
use crate::types::{Node, Span};

use super::{atom, numeric_literal, token_if, token_tag, Error, Input, KResult, Token};

pub fn expr<'input>(i: Input<'input>) -> KResult<'input, Node<Expr>> {
    a_expr(i)
}

pub fn a_expr<'input>(i: Input<'input>) -> KResult<'input, Node<Expr>> {
    let (i, lhs) = m_expr(i)?;
    let (i, rhs) = opt(nom_tuple((adds, a_expr)))(i)?;
    match rhs {
        Some((op, rhs)) => Ok((
            i,
            Node::new(
                lhs.span.clone() + rhs.span.clone(),
                Expr::Op(lhs, op.clone(), rhs),
            ),
        )),
        None => Ok((i, lhs)),
    }
}

pub fn m_expr<'input>(i: Input<'input>) -> KResult<'input, Node<Expr>> {
    let (i, lhs) = p_expr(i)?;
    let (i, rhs) = opt(nom_tuple((muls, p_expr)))(i)?;
    match rhs {
        Some((op, rhs)) => Ok((
            i,
            Node::new(
                lhs.span.clone() + rhs.span.clone(),
                Expr::Op(lhs, op.clone(), rhs),
            ),
        )),
        None => Ok((i, lhs)),
    }
}

pub fn p_expr<'input>(i: Input<'input>) -> KResult<'input, Node<Expr>> {
    let (i, lhs) = u_expr(i)?;
    let (i, rhs) = opt(nom_tuple((token_tag(Token::POW), u_expr)))(i)?;
    match rhs {
        Some((op, rhs)) => Ok((
            i,
            Node::new(
                lhs.span.clone() + rhs.span.clone(),
                Expr::Op(lhs, op.clone(), rhs),
            ),
        )),
        None => Ok((i, lhs)),
    }
}

pub fn u_expr<'input>(i: Input<'input>) -> KResult<'input, Node<Expr>> {
    let (i, op) = opt(adds)(i)?;
    let (i, rhs) = coefficient(i)?;
    match op {
        Some(op) => Ok((
            i,
            Node::new(
                op.span.clone() + rhs.span.clone(),
                Expr::Unary(op.clone(), rhs),
            ),
        )),
        None => Ok((i, rhs)),
    }
}

pub fn coefficient<'input>(i: Input<'input>) -> KResult<'input, Node<Expr>> {
    let (i, lhs) = opt(numeric_literal)(i)?;
    let (i, rhs) = opt(die)(i)?;
    match (lhs, rhs) {
        (Some(lhs), Some(rhs)) => Ok((
            i,
            Node::new(
                lhs.span.clone() + rhs.span.clone(),
                Expr::Coefficient(lhs, rhs),
            ),
        )),
        (Some(lhs), None) => Ok((
            i,
            Node::new(lhs.span.clone(), Expr::Primary(Primary::Atom(*lhs.data))),
        )),
        (None, Some(rhs)) => Ok((i, rhs)),
        (None, None) => Err(Err::Error(Node::new(Span::from_iter(i), Error::Grammar))),
    }
}

pub fn die<'input>(i: Input<'input>) -> KResult<'input, Node<Expr>> {
    let (i, die_val) = opt(nom_tuple((token_tag(Token::DIE), numeric_literal)))(i)?;
    match die_val {
        Some((op, rhs)) => Ok((i, Node::new(op.span + rhs.span, Expr::Die(rhs)))),
        None => expr_node(i),
    }
}

pub fn expr_node<'input>(i: Input<'input>) -> KResult<'input, Node<Expr>> {
    let (i, val) = atom(i)?;
    Ok((
        i,
        Node::new(val.span.clone(), Expr::Primary(Primary::Atom(*val.data))),
    ))
}

pub fn adds<'input>(i: Input<'input>) -> KResult<'input, &Node<Token>> {
    token_if(|x| match *x.data {
        Token::ADD | Token::SUB => true,
        _ => false,
    })(i)
}

pub fn muls<'input>(i: Input<'input>) -> KResult<'input, &Node<Token>> {
    token_if(|x| match *x.data {
        Token::MOD | Token::MUL | Token::DIV => true,
        _ => false,
    })(i)
}
