use super::common::error;
use super::lexer::TokenType;
use std::cmp::Ordering;
use std::ops;

#[derive(Clone, PartialEq, Debug)]
pub enum VariableType
{
    Integer(isize),
    Boolean(bool),
    Str(String)
}

pub fn is_token_type_valid_type(token_type: &TokenType) -> bool
{
    match token_type
    {
        TokenType::Int => true,
        TokenType::Bool => true,
        TokenType::Str => true,
        _ => false
    }
}

pub fn token_type_to_variable_type(token_type: &TokenType) -> VariableType
{
    match token_type
    {
        TokenType::Int => VariableType::Integer(0),
        TokenType::Bool => VariableType::Boolean(false),
        TokenType::Str => VariableType::Str(String::new()),
        _ => panic!()
    }
}

#[derive(Clone, Debug)]
pub struct Variable
{
    pub variable_type: VariableType
}

impl Variable
{
    pub fn set(&mut self, variable: &Variable)
    {
        if self.is_string_and_so_is(variable)
        {
            self.variable_type = variable.variable_type.clone();
            return;
        }

        self.detect_conflicting_string_types(variable);
        self.set_from_integer(variable.as_integer());
    }

    fn is_string(&self) -> bool
    {
        matches!(self.variable_type, VariableType::Str(_))
    }

    fn is_string_and_so_is(&self, variable: &Variable) -> bool
    {
        self.is_string() && variable.is_string()
    }

    fn detect_conflicting_string_types(&self, variable: &Variable)
    {
        fn is_numeric(value: &String) -> bool
        {
            for character in value.chars()
            {
                if !character.is_numeric() {
                    return false
                }
            }
            true
        }

        let is_error = match &self.variable_type
        {
            VariableType::Str(a) => match &variable.variable_type
            {
                VariableType::Str(b) =>
                {
                    if is_numeric(&a) && is_numeric(&b) { false }
                    else { true }
                },
                _ =>
                {
                    if is_numeric(&a) { false }
                    else { false }
                }
            },

            _ =>
            {
                match &variable.variable_type
                {
                    VariableType::Str(b) =>
                    {
                        if is_numeric(&b) { false }
                        else { true }
                    },
                    _ => { false }
                }
            }
        };

        if is_error {
            error("attempt to cast non-numeric string with other type".to_string());
        }
    }

    fn as_integer(&self) -> isize
    {
        match &self.variable_type
        {
            VariableType::Integer(value) => *value,
            VariableType::Boolean(value) => bool_to_int(value),
            VariableType::Str(value) => string_to_int(&value)
        }
    }

    fn set_from_integer(&mut self, value: isize)
    {
        let variable_type = match &self.variable_type
        {
            VariableType::Integer(_) => VariableType::Integer(value),
            VariableType::Boolean(_) => VariableType::Boolean(int_to_bool(value)),
            VariableType::Str(_) => VariableType::Str(int_to_string(value))
        };

        self.variable_type = variable_type;
    }

    pub fn printed_string(&self) -> String
    {
        match &self.variable_type
        {
            VariableType::Integer(value) => format!("{}", value),
            VariableType::Boolean(value) => format!("{}", value),
            VariableType::Str(value) => value.clone()
        }
    }
}

fn bool_to_int(value: &bool) -> isize
{
    if *value == false { 0 } else { 1 }
}
fn int_to_bool(value: isize) -> bool
{
    if value == 0 { false } else { true }
}

fn string_to_int(value: &String) -> isize { value.parse::<isize>().unwrap() }
fn int_to_string(value: isize) -> String { value.to_string() }

impl ops::Add<Variable> for Variable
{
    type Output = Variable;

    fn add(self, rhs: Variable) -> Variable
    {
        let self_as_int = self.as_integer();
        let rhs_as_int = rhs.as_integer();

        let mut new = self.clone();
        new.set_from_integer(self_as_int + rhs_as_int);
        new
    }
}

impl ops::AddAssign<Variable> for Variable
{
    fn add_assign(&mut self, rhs: Variable)
    {
        self.set_from_integer(self.as_integer() + rhs.as_integer());
    }
}

impl ops::MulAssign<Variable> for Variable
{
    fn mul_assign(&mut self, rhs: Variable)
    {
        self.set_from_integer(self.as_integer() * rhs.as_integer());
    }
}

impl PartialEq<Self> for Variable
{
    fn eq(&self, rhs: &Self) -> bool
    {
        self.as_integer() == rhs.as_integer()
    }
}

impl PartialOrd for Variable
{
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering>
    {
        let self_as_int = self.as_integer();
        let rhs_as_int = rhs.as_integer();

        if self_as_int < rhs_as_int { Some(Ordering::Less) }
        else if self_as_int > rhs_as_int { Some(Ordering::Greater) }
        else { Some(Ordering::Equal) }
    }
}
