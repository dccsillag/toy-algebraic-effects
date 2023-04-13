use std::{any::Any, cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Variable(usize);

#[derive(Clone)]
enum AST {
    Lambda(Variable, Box<AST>),
    Application(Box<AST>, Box<AST>),
    Variable(Variable),
    Const(Value),
    Cond(Box<AST>, Box<AST>, Box<AST>),
}

#[derive(Clone)]
enum Value {
    BuiltinFunction(Rc<dyn Fn(Value, &mut State) -> Result<Value, Error>>),
    BuiltinValue(Rc<dyn Any>),
    Function(Rc<RefCell<Context>>, Variable, Box<AST>),
    Bool(bool),
    Int(i64),
    String(String),
}

#[derive(Clone)]
enum Error {
    NotInScope(Variable),
    NotACallableValue(Value),
    NotABoolValue(Value),
}

struct State {}

#[derive(Clone)]
struct Context(HashMap<Variable, Value>);

impl Context {
    pub fn with_var<T>(
        &mut self,
        var: Variable,
        value: Value,
        func: impl FnOnce(&mut Context) -> T,
    ) -> T {
        let maybe_save = self.0.insert(var, value);
        let out = func(self);
        match maybe_save {
            Some(save) => self.0.insert(var, save),
            None => self.0.remove(&var),
        };
        out
    }

    pub fn lookup(&self, variable: &Variable) -> Option<&Value> {
        self.0.get(variable)
    }
}

fn interpret(ast: &AST, context: &mut Context, state: &mut State) -> Result<Value, Error> {
    match ast {
        AST::Lambda(bound_var, body) => Ok(Value::Function(
            Rc::new(RefCell::new(context.clone())),
            *bound_var,
            body.clone(),
        )),
        AST::Application(f, x) => {
            let arg = interpret(x, context, state)?;
            match interpret(f, context, state)? {
                Value::BuiltinFunction(func) => func(arg, state),
                Value::Function(closure_context, bound_var, body) => closure_context
                    .borrow_mut()
                    .with_var(bound_var, arg, |context| interpret(&body, context, state)),
                val @ (Value::BuiltinValue(_)
                | Value::Bool(_)
                | Value::Int(_)
                | Value::String(_)) => Err(Error::NotACallableValue(val)),
            }
        }
        AST::Variable(var) => match context.lookup(var) {
            Some(out) => Ok(out.clone()),
            None => Err(Error::NotInScope(*var)),
        },
        AST::Const(val) => Ok(val.clone()),
        AST::Cond(cond, then, r#else) => match interpret(cond, context, state)? {
            Value::Bool(b) => interpret(if b { then } else { r#else }, context, state),
            val @ (Value::BuiltinFunction(_)
            | Value::BuiltinValue(_)
            | Value::Function(_, _, _)
            | Value::Int(_)
            | Value::String(_)) => Err(Error::NotABoolValue(val)),
        },
    }
}

fn main() {
    println!("Hello, world!");
}
