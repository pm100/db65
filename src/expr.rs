/*

Evaluates arbitrary address expressions

Any shell command tha expects an address can be given an expression instead.
An expression starts with '=' and is followed by a valid evalexpr expression.
It may need to be quoted to avoid shell expansion and clap confusuion.

The regsisters are variables called ac, xr, yr, sp, pc
All symbols are available as variables
You can deference a pointer using '@(<ptr>)'
the expr command evaluates an expression and prints the result

so you can do

dis =pc
mem =xr+0x20
mem .ptr // no need for expression, symbols just work anyway
mem =@(.ptr) // deference a pointer
mem '=@(.ptr + 0x20)' // do math on a pointer
mem =@(.ptr + (0x20*xr)) // more math



*/

use crate::{cpu::Cpu, debugger::Debugger};
use anyhow::{anyhow, Result};
use evalexpr::{eval_int_with_context, Context, EvalexprResult, Value};
use std::{collections::HashMap, ops::RangeInclusive};

pub struct DB65Context {
    pub symbols: HashMap<String, Value>,
    ac: Value,
    xr: Value,
    yr: Value,
    sp: Value,
    pc: Value,
}

impl DB65Context {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            ac: Value::Int(0),
            xr: Value::Int(0),
            yr: Value::Int(0),
            sp: Value::Int(0),
            pc: Value::Int(0),
        }
    }
    pub fn reload(&mut self) {
        self.ac = Value::Int(Cpu::read_ac() as i64);
        self.xr = Value::Int(Cpu::read_xr() as i64);
        self.yr = Value::Int(Cpu::read_yr() as i64);
        self.sp = Value::Int(Cpu::read_sp() as i64);
        self.pc = Value::Int(Cpu::read_pc() as i64);
    }
}
impl Context for DB65Context {
    fn get_value(&self, key: &str) -> Option<&Value> {
        match key {
            "ac" => Some(&self.ac),
            "xr" => Some(&self.xr),
            "yr" => Some(&self.yr),
            "sp" => Some(&self.sp),
            "pc" => Some(&self.pc),
            _ => self.symbols.get(key),
        }
    }
    fn call_function(&self, key: &str, arg: &Value) -> EvalexprResult<Value> {
        match key {
            "@" => {
                let arg = arg.as_int()?;
                if arg > u16::MAX as i64 {
                    return Err(evalexpr::EvalexprError::WrongFunctionArgumentAmount {
                        expected: RangeInclusive::new(0, 0xffff),
                        actual: arg as usize,
                    });
                }
                let word = Cpu::read_word(arg as u16);
                Ok(evalexpr::Value::Int(word as i64))
            }

            _ => Err(evalexpr::EvalexprError::FunctionIdentifierNotFound(
                key.to_string(),
            )),
        }
    }
    fn are_builtin_functions_disabled(&self) -> bool {
        false
    }
    fn set_builtin_functions_disabled(&mut self, _disabled: bool) -> EvalexprResult<()> {
        Err(evalexpr::EvalexprError::CustomMessage(
            "builtin functions are not supported".to_string(),
        ))
    }
}
impl Debugger {
    pub fn evaluate(&mut self, expr: &str) -> Result<u16> {
        // reload register values
        self.expr_context.reload();
        eval_int_with_context(expr, &mut self.expr_context)
            .map_err(|e| anyhow!(e))
            .map(|v| v as u16)
    }
}
