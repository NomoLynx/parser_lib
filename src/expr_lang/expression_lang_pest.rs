use pest::{iterators::{Pair, Pairs}, pratt_parser::PrattParser, Parser};
use pest_derive::Parser;

use core_utils::debug::*;
use core_utils::number::get_i64_from_str;
use super::expr_lang_error::*;
pub use super::ExprValue;

#[derive(Parser)]
#[grammar = "basic_type_grammar.pest"]
#[grammar = "expr_lang/expression_lang.pest"]
pub struct BasicTypeParser;

pub fn create_pratt() -> PrattParser<Rule> {
    use pest::pratt_parser::{Assoc::*, Op};
    use Rule::*;

    let pratt = PrattParser::new()
        // Addition and subtract have equal precedence
        .op(Op::infix(operator_equal_or_greater, Left) | Op::infix(operator_greater, Left) | Op::infix(operator_equal_or_less, Left) | Op::infix(operator_less, Left) | Op::infix(logic_equal, Left))
        .op(Op::infix(logic_and, Left) | Op::infix(logic_or, Left))
        .op(Op::infix(bit_left_move, Left) | Op::infix(bit_right_move, Left) | Op::infix(bit_and, Left) | Op::infix(bit_or, Left) | Op::infix(bit_exclusive_or, Left))
        .op(Op::infix(addition, Left) | Op::infix(subtraction, Left))
        .op(Op::infix(multiplication, Left) | Op::infix(division, Left))
        .op(Op::infix(pow, Left))
        .op(Op::prefix(unary_op) | Op::prefix(left_bracket))
        .op(Op::postfix(right_bracket));

    pratt
}

pub fn pratt_parse(pratt:&PrattParser<Rule>, ps:Pairs<Rule>) -> String {
    pratt.map_primary(|primary| {
        match primary.as_rule() {
            Rule::identifier => {
                let s = primary.as_str().to_string();
                s
            }
            Rule::expression => {
                pratt_parse(pratt, primary.into_inner())
            }
            Rule::integer => {
                let s = primary.as_str().to_string();
                s
            }
            Rule::bool_value => {
                let s = primary.as_str().to_string();
                s
            }
            Rule::tenary_expression => {
                let inner = primary.into_inner();
                if inner.len() == 1 {
                    pratt_parse(pratt, inner)
                }
                else {
                    todo!("need to implement the full 3 element of tenary expression")
                }
            }
            rule => {
                unreachable!("wrong rule present here: {rule:?}")
            }
        }}).map_infix(|left, op, right| {
            let s = match op.as_rule() {
                Rule::addition => "+",
                Rule::subtraction => "-",
                Rule::multiplication => "*",
                Rule::division => "/",
                _ => todo!("{op:?} not processed"),
            };
            format!("({left} {s} {right})")
        }).map_prefix(|op, expr| {
            let s = match op.as_rule() {
                Rule::unary_op => op.as_str(),
                Rule::left_bracket => "(",
                _ => todo!("{op:?} not processed")
            };

            format!("({s} {expr})")
        }).map_postfix(|expr, op| {
            let s = match op.as_rule() {
                Rule::right_bracket => ")", 
                _ => todo!("{op:?} not processed")
            };

            format!("({expr} {s})")
        })
        .parse(ps)
}

pub fn compute_tenary_expression(pratt:&PrattParser<Rule>, ps:Pair<Rule>, context:Option<&dyn ExpressionContextTrait>) -> Result<ExprValue, ExprLangError> {
    let rule = ps.as_rule();
    match rule {
        Rule::expression => {
            compute_expression(pratt, ps.into_inner(), context)
        }
        Rule::tenary_expression => {
            let mut inners = ps.into_inner();
            let b = compute_tenary_expression(pratt, inners.nth(0).unwrap(), context)?;
            
            if b.bool()? {
                let p = inners.nth(0).unwrap();
                debug_string(format!("TRUE and compute {p:?}"));
                let r = compute_tenary_expression(pratt, p, context);
                r
            }
            else {
                let p = inners.nth(1).unwrap();
                debug_string(format!("FALSE and compute {p:?}"));
                let r = compute_tenary_expression(pratt, p, context);
                r
            }
        }
        _ => {
            Err(ExprLangError::NoFound((file!().to_string(), line!()).into(), format!("cannot find code to process {rule:?}")))
        }
    }
}

pub fn compute_expression(pratt:&PrattParser<Rule>, ps:Pairs<Rule>, context:Option<&dyn ExpressionContextTrait>) -> Result<ExprValue, ExprLangError> {
    Ok(pratt.map_primary(|primary| {
        let rule = primary.as_rule();
        match rule {
            Rule::expression => {
                compute_expression(pratt, primary.into_inner(), context)
            }
            Rule::integer => {
                let s = primary.as_str().to_string();
                let v = get_i64_from_str(&s)?;
                Ok(ExprValue::from_i64(v))
            }
            Rule::float => {
                let s = primary.as_str().to_string();

                if let Some(ctx) = context {
                    if let Some(v) = ctx.get_value(&s) {
                        return Ok(v);
                    }
                }

                if s == "." {
                    Ok(ExprValue::Float64(0.0))
                }
                else {
                    let v = s.parse::<f64>()
                                                .map(|x| ExprValue::Float64(x))
                                                .map_err(|_| ExprLangError::ParameterError);
                    v
                }
            }
            Rule::bool_value => {
                let v = ExprValue::bool_from_string(primary.as_str());
                Ok(v?)
            }
            Rule::tenary_expression => {
                let inner = primary.into_inner();
                if inner.len() == 1 {
                    compute_expression(pratt, inner, context)
                }
                else {
                    todo!("need to implement the full 3 element of tenary expression")
                }
            }
            Rule::identifier => {
                let id = primary.as_str();
                if let Some(ctx) = context {
                    if let Some(v) = ctx.get_value(id) {
                        return Ok(v);
                    }
                }
                
                let r = match id {
                    "." => ExprValue::from_u64(0),
                    _ => ExprValue::Invalid,
                };
                Ok(r)
            }
            rule => {
                unreachable!("wrong rule present here: {rule:?} and primay = {primary:?}@{:?}", 
                    primary.line_col())
            }
        }}).map_infix(|left, op, right| {
            let a = left?;
            let b = right?;

            if a.is_invalid() || b.is_invalid() {
                return Ok(ExprValue::Invalid)
            }

            let s = match op.as_rule() {
                Rule::addition => {
                    a.add(&b)
                }
                Rule::subtraction => {
                    a.sub(&b)
                }
                Rule::multiplication => {
                    a.mul(&b)
                }
                Rule::division => {
                    a.div(&b)
                }
                Rule::pow => {
                    a.pow(&b)
                }
                Rule::bit_and => {
                    a.and(&b)
                }
                Rule::bit_or => {
                    a.or(&b)
                }
                Rule::bit_exclusive_or => {
                    a.xor(&b)
                }
                Rule::bit_left_move => {
                    a.shl(&b)
                }
                Rule::bit_right_move => {
                    a.shr(&b)
                }
                Rule::logic_and => {
                    a.bool_and(&b)
                }
                Rule::logic_or => {
                    a.bool_or(&b)
                }
                Rule::operator_greater => {
                    let r = a.i64()? > b.i64()?;
                    Ok(ExprValue::from_bool(r))
                }
                Rule::operator_equal_or_greater => {
                    Ok(ExprValue::from_bool(a.i64()? >= b.i64()?))
                }
                Rule::operator_equal_or_less => {
                    Ok(ExprValue::from_bool(a.i64()? <= b.i64()?))
                }
                Rule::operator_less => {
                    Ok(ExprValue::from_bool(a.i64()? < b.i64()?))
                }
                Rule::logic_equal => {
                    Ok(ExprValue::from_bool(a.i64()? == b.i64()?))
                }
                _ => todo!("todo: {op:?} not processed"),
            };
            
            Ok(s?)
        }).map_prefix(|op, expr| {
            let s = match op.as_rule() {
                Rule::unary_op => {
                    let n = expr?;
                    if n.is_invalid() {
                        return Ok(ExprValue::Invalid)
                    }

                    let op2 = op.into_inner().nth(0).unwrap();
                    match op2.as_rule() {
                        Rule::subtraction => {
                            n.neg().map_err(ExprLangError::from)
                        }
                        Rule::logic_not => {
                            if let Ok(b) = n.bool() {
                                Ok(ExprValue::from_bool(!b))
                            }
                            else {
                                n.not().map_err(ExprLangError::from)
                            }
                        }
                        _ => {
                            todo!("{op2:?} not processed")
                        }
                    }
                }
                Rule::left_bracket => expr,
                _ => todo!("{op:?} not processed")
            };

            Ok(s?)
        }).map_postfix(|expr, op| {
            let s = match op.as_rule() {
                Rule::right_bracket => expr, 
                _ => todo!("{op:?} not processed")
            };

            Ok(s?)
        })
        .parse(ps)?)
}

pub fn expr_to_clrobj(expr_str:&str, context:Option<&dyn ExpressionContextTrait>) -> Result<ExprValue, ExprLangError> 
{
    let mut expr = BasicTypeParser::parse(Rule::expr, expr_str).unwrap();
    let expression = expr.nth(0).unwrap().into_inner().nth(0).unwrap();
    let pratt = create_pratt();
    let output = compute_tenary_expression(&pratt, expression, context);
    match output {
        Err(e) => {
            let error_msg = format!("cannot compute expression '{expr_str}', error is {e:?}");
            error_string(error_msg.clone());
            Err(ExprLangError::Other(error_msg))
        }
        Ok(y) => Ok(y)
    }
}

pub trait ExpressionContextTrait {
    fn get_value(&self, identifier:&str) -> Option<ExprValue>;
}