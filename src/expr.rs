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
mem =.xr+0x20
mem ptr // no need for expression, symbols just work anyway
mem =@(ptr) // deference a pointer
mem '=@(ptr + 0x20)' // do math on a pointer
mem =@(ptr + (0x20*xr)) // more math



*/

use crate::{debugger::core::Debugger, debugger::cpu::Cpu};
use anyhow::{anyhow, Result};
use evalexpr::{eval_int_with_context, Context, EvalexprResult, Value};
use std::ops::RangeInclusive;

impl Context for Debugger {
    fn get_value(&self, key: &str) -> Option<&Value> {
        let val = match key {
            ".ac" => self.read_ac() as i64,
            ".xr" => self.read_xr() as i64,
            ".yr" => self.read_yr() as i64,
            ".sp" => self.read_sp() as i64,
            ".pc" => self.read_pc() as i64,
            ".sr" => self.read_sr() as i64,
            _ => {
                let sym = self.convert_addr(key).ok();
                if let Some(s) = sym {
                    s.0 as i64
                } else {
                    return None;
                }
            }
        };
        // horrible hack because we have to return
        // a calculated value refernce from a read only context
        self.expr_value.replace(Value::Int(val));
        let p = self.expr_value.as_ptr();
        Some(unsafe { &*p })
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
            "@b" => {
                let arg = arg.as_int()?;
                if arg > u16::MAX as i64 {
                    return Err(evalexpr::EvalexprError::WrongFunctionArgumentAmount {
                        expected: RangeInclusive::new(0, 0xffff),
                        actual: arg as usize,
                    });
                }
                let byte = Cpu::read_byte(arg as u16);
                Ok(evalexpr::Value::Int(byte as i64))
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
        eval_int_with_context(expr, self)
            .map_err(|e| anyhow!(e))
            .map(|v| v as u16)
    }
}
