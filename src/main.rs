use std::{any::Any, cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Clone, PartialEq, Eq, Hash)]
struct Variable(String);

macro_rules! var {
    ($name:literal) => {
        Variable(stringify!($name).to_string())
    };
}

#[derive(Clone)]
enum Ast {
    Lambda(Variable, Box<Ast>),
    Fix,
    Application(Box<Ast>, Box<Ast>),
    Variable(Variable),
    Const(Value),
    Cond(Box<Ast>, Box<Ast>, Box<Ast>),
}

#[derive(Clone)]
enum Value {
    Fix,
    BuiltinFunction(Rc<dyn Fn(Value, &mut State) -> Result<Value, Error>>),
    BuiltinValue(Rc<dyn Any>),
    Function(Rc<RefCell<Context>>, Variable, Box<Ast>),
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

struct State {
    content: Vec<String>,
    document_size: usize,
}

#[derive(Clone)]
struct Context(HashMap<Variable, Value>);

impl Context {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, var: Variable, value: Value) {
        let ret = self.0.insert(var, value);
        assert!(ret.is_none());
    }

    pub fn with_var<T>(
        &mut self,
        var: &Variable,
        value: Value,
        func: impl FnOnce(&mut Context) -> T,
    ) -> T {
        let maybe_save = self.0.insert(var.clone(), value);
        let out = func(self);
        match maybe_save {
            Some(save) => self.0.insert(var.clone(), save),
            None => self.0.remove(var),
        };
        out
    }

    pub fn lookup(&self, variable: &Variable) -> Option<&Value> {
        self.0.get(variable)
    }
}

fn interpret(ast: &Ast, context: &mut Context, state: &mut State) -> Result<Value, Error> {
    match ast {
        Ast::Lambda(bound_var, body) => Ok(Value::Function(
            Rc::new(RefCell::new(context.clone())),
            bound_var.clone(),
            body.clone(),
        )),
        Ast::Fix => Ok(Value::Fix),
        Ast::Application(f, x) => {
            let arg = interpret(x, context, state)?;
            match interpret(f, context, state)? {
                Value::BuiltinFunction(func) => func(arg, state),
                Value::Function(closure_context, bound_var, body) => closure_context
                    .borrow_mut()
                    .with_var(&bound_var, arg, |context| interpret(&body, context, state)),
                Value::Fix => interpret(
                    &Ast::Application(Box::new(Ast::Const(arg)), Box::new(ast.clone())),
                    context,
                    state,
                ),
                val @ (Value::BuiltinValue(_)
                | Value::Bool(_)
                | Value::Int(_)
                | Value::String(_)) => Err(Error::NotACallableValue(val)),
            }
        }
        Ast::Variable(var) => match context.lookup(var) {
            Some(out) => Ok(out.clone()),
            None => Err(Error::NotInScope(var.clone())),
        },
        Ast::Const(val) => Ok(val.clone()),
        Ast::Cond(cond, then, r#else) => match interpret(cond, context, state)? {
            Value::Bool(b) => interpret(if b { then } else { r#else }, context, state),
            val @ (Value::BuiltinFunction(_)
            | Value::Fix
            | Value::BuiltinValue(_)
            | Value::Function(_, _, _)
            | Value::Int(_)
            | Value::String(_)) => Err(Error::NotABoolValue(val)),
        },
    }
}

fn initialize(expected_document_size: usize) -> (Context, State) {
    let mut context = Context::new();
    let state = State {
        content: Vec::new(),
        document_size: expected_document_size,
    };

    context.insert(var!("true"), Value::Bool(true));
    context.insert(var!("false"), Value::Bool(false));
    context.insert(
        var!("content"),
        Value::BuiltinFunction(Rc::new(|input, state| match input {
            Value::String(str) => {
                state.content.push(str.to_string());
                Ok(Value::String(str))
            }
            Value::BuiltinFunction(_)
            | Value::Fix
            | Value::BuiltinValue(_)
            | Value::Function(_, _, _)
            | Value::Bool(_)
            | Value::Int(_) => todo!(),
        })),
    );
    context.insert(
        var!("location"),
        Value::BuiltinFunction(Rc::new(|_input, state| {
            Ok(Value::Int(state.content.len().try_into().unwrap()))
        })),
    );
    context.insert(
        var!("size"),
        Value::BuiltinFunction(Rc::new(|_input, state| {
            Ok(Value::Int(state.document_size.try_into().unwrap()))
        })),
    );
    context.insert(
        var!("percent"),
        Value::BuiltinFunction(Rc::new(|_input, state| {
            let k: i64 = state.content.len().try_into().unwrap();
            let n: i64 = state.document_size.try_into().unwrap();
            Ok(Value::Int(k / n))
        })),
    );

    (context, state)
}

fn compile(ast: &Ast) -> Result<Vec<String>, Error> {
    let mut document_size = 0;
    let mut out = None;
    for i in 0..5 {
        println!("Iteration #{i}");

        let (mut context, mut state) = initialize(document_size);

        interpret(ast, &mut context, &mut state)?;

        let new_out = Some(state.content);
        if out == new_out {
            println!("Converged!");
            break;
        }
        out = new_out;
        document_size = state.document_size;
    }
    Ok(out.unwrap())
}

fn main() {
    println!("Hello, world!");
}
