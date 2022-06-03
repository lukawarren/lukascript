use super::parser::Instruction;
use super::parser::Instruction::*;
use super::variables::Variable;
use super::variables::VariableType;
use super::operators::value_contains_operator;
use super::operators::is_char_operator;
use super::operators::operator_char_to_token_type;
use super::operators::evaluate_operator_expression;
use super::operators::OperatorExpression;
use super::common::error;

use std::collections::HashMap;

#[derive(Clone)]
enum Frame
{
    Root,
    ForLoop { variable: String, start_line: usize, end_value: String },
    Function { caller_line: usize },
    IfStatement
}

struct FrameContext
{
    frame: Frame,
    variables: HashMap<String, Variable>,
    functions: HashMap<String, usize> // beginning line
}

#[derive(Default)]
pub struct State
{
    line: usize,
    frames: Vec<FrameContext>
}

impl State
{
    pub fn execute(&mut self, instructions: Vec<Instruction>)
    {
        // Set up root frame
        self.add_frame(Frame::Root);

        // Helper "variables"
        let one = Variable { variable_type: VariableType::Integer(1) };

        // Boolean declarations - TODO: fix
        self.make_variable_of_type(&String::from("true"), VariableType::Boolean(true));
        self.make_variable_of_type(&String::from("false"), VariableType::Boolean(false));

        while self.line != instructions.len()
        {
            match &instructions[self.line]
            {
                FromValueToValue { value, start, end } =>
                {
                    // Don't run if conditions not valid
                    let start_value = self.evaluate_value(start);
                    if start_value < self.evaluate_value(end)
                    {
                        // Ensure the value is a valid variable name
                        if self.is_numeric(value) {
                            self.error("invalid variable name");
                        }

                        self.add_frame(Frame::ForLoop {
                            variable: value.clone(),
                            start_line: self.line,
                            end_value: end.clone()
                        });

                        self.make_variable_of_type(value, VariableType::Integer(0));
                        self.set_variable(value, start);
                    }
                },

                IfValueIsValue { left_value, right_value, last_line } =>
                {
                    if self.evaluate_value(left_value) == self.evaluate_value(right_value) {
                        self.add_frame(Frame::IfStatement);
                    }
                    else {
                        self.line = *last_line;
                    }
                },

                IfValueIsNotValue { left_value, right_value, last_line } =>
                {
                    if self.evaluate_value(left_value) != self.evaluate_value(right_value) {
                        self.add_frame(Frame::IfStatement);
                    }
                    else {
                        self.line = *last_line;
                    }
                }

                FunctionCall { function, values } =>
                {
                    // Check for user-defined functions first, then if that fails, assume it's in-built
                    let mut found_function = Option::<usize>::default();
                    self.for_each_frame(|frame| {
                        if found_function.is_none() && frame.functions.contains_key(function) {
                            let _ = found_function.insert(*frame.functions.get(function).unwrap());
                        }
                    });

                    if found_function.is_some()
                    {
                        self.add_frame(Frame::Function { caller_line: self.line });
                        self.line = found_function.unwrap();
                        // TODO: pass arguments
                    }

                    // Function not found, assume part of the "standard library"
                    else
                    {
                        if function == "print" {
                            for value in values {
                                print!("{} ", self.evaluate_value(value).printed_string());
                            }
                            println!();
                        }
                        else {
                            self.error("unknown function");
                        }
                    }
                },

                Done =>
                {
                    // At the termination of a frame, it's our responsibility to go back to the start (potentially).
                    // In other words, this is the instruction that'll contain the logic for loops.

                    if self.frames.is_empty() {
                        self.error("no appropriate frame");
                    }

                    match self.innermost_frame().frame.clone()
                    {
                        Frame::ForLoop { variable, start_line, end_value } =>
                        {
                            if self.get_variable(&variable).clone() + one.clone() >= self.evaluate_value(&end_value)
                            {
                                // End of loop reached
                                self.frames.pop();
                            }
                            else
                            {
                                // Loop back
                                *self.get_variable(&variable) += one.clone();
                                self.line = start_line;
                            }
                        },

                        Frame::Function { caller_line } => {
                            self.frames.pop();
                            self.line = caller_line;
                        },

                        Frame::IfStatement => {
                            self.frames.pop();
                        },

                        Frame::Root => {
                            self.error("attempt to terminate root frame");
                        }
                    }
                },

                FunctionDeclaration { name, first_line, last_line } =>
                {
                    // Note function then sally on forth - TODO: decide if to allow re-defining functions
                    if self.innermost_frame().functions.insert(name.clone(), *first_line).is_some() {
                        self.error("function already declared");
                    }
                    self.line = *last_line;
                },

                IntDeclaration { name, value } =>
                {
                    self.make_variable_of_type(name, VariableType::Integer(0));
                    self.set_variable(name, value);
                },

                BoolDeclaration { name, value } =>
                {
                    self.make_variable_of_type(name, VariableType::Boolean(false));
                    self.set_variable(name, value);
                },

                StringDeclaration { name, value } =>
                {
                    self.make_variable_of_type(name, VariableType::Str(String::new()));
                    self.set_variable(name, value);
                }

                Assignment { name, value } => { self.set_variable(name, value); }

                NoOp => {},
            }

            self.line += 1;
        }
    }

    pub fn print_variables(&self)
    {
        for i in 0..self.frames.len()
        {
            for variable in &self.frames[i].variables
            {
                for _ in 0..i { print!("    "); }
                println!("{}: {:?}", variable.0, variable.1.variable_type);
            }
        }
    }

    fn error(&self, message: &str) -> !
    {
        error(format!("{} - line {}", message, self.line + 1));
    }

    fn is_numeric(&self, value: &String) -> bool
    {
        for character in value.chars()
        {
            if !character.is_numeric() {
                return false
            }
        }
        true
    }

    fn innermost_frame(&mut self) -> &mut FrameContext
    {
        let index = self.frames.len()-1;
        &mut self.frames[index]
    }

    fn add_frame(&mut self, frame: Frame)
    {
        self.frames.push(FrameContext {
            frame,
            variables: HashMap::<String, Variable>::new(),
            functions: HashMap::<String, usize>::new()
        });
    }

    fn for_each_frame<F: FnMut(&FrameContext)>(&self, mut f: F)
    {
        // Go from inner-most frame to root (i.e. reversed)
        for i in 1..=self.frames.len()
        {
            let index = self.frames.len() - i;
            f(&self.frames[index]);
        }
    }

    fn evaluate_value(&mut self, value: &String) -> Variable
    {
        // A value may simply be something like "3" or "my_variable_name", but may also contain operators like "+" or "-".
        // To this end, parse each individual "actual value" (inner value) and combine them with any operators to form an
        // expression of sorts that can be evaluated separately, containing only numbers and operators. Of course, for values
        // not containing any operators, this can be skipped.

        if !value_contains_operator(value) { return self.evaluate_inner_value(value) }

        let mut expression = Vec::<OperatorExpression>::new();
        let mut word = Vec::<char>::new();

        // March along, growing each accumulated "word" until an operator is found (or the string ends)
        for i in 0..value.len()
        {
            let char = value.chars().nth(i).unwrap();
            let is_operator = is_char_operator(char);
            word.push(char);

            if is_operator || i == value.len() -1
            {
                if i != value.len() - 1 {
                    word.pop(); // Final character will be the operator, so remove
                }

                expression.push(OperatorExpression::Variable(
                    self.evaluate_inner_value(&word.iter().collect())
                ));

                if is_operator
                {
                    expression.push(OperatorExpression::Operator(
                        operator_char_to_token_type(char)
                    ));
                }

                word.clear();
            }
        }

        return evaluate_operator_expression(&expression)
    }

    fn evaluate_inner_value(&mut self, value: &String) -> Variable
    {
        // Treat numbers as temporary ints
        if self.is_numeric(value) {
            Variable {
                variable_type: VariableType::Integer(value.parse().unwrap())
            }
        }

        // Strings
        else if value.len() >= 2 && value.chars().nth(0).unwrap() == '\"' && value.chars().nth(value.len()-1).unwrap() == '\"'
        {
            let mut new_value = value.clone();
            new_value.pop();
            new_value.remove(0);

            Variable {
                variable_type: VariableType::Str(new_value)
            }
        }

        // Otherwise it must be a variable name
        else { self.get_variable(value).clone() }
    }

    fn get_variable(&mut self, name: &String) -> &mut Variable
    {
        for i in 1..=self.frames.len()
        {
            let index = self.frames.len() - i;

            if self.frames[index].variables.contains_key(name) {
                return self.frames[index].variables.get_mut(name).unwrap();
            }
        }

        self.error(format!("variable \"{}\" does not exist", name).as_str());
    }

    fn set_variable(&mut self, name: &String, value: &String)
    {
        let len = self.frames.len();
        let evaluated = self.evaluate_value(value);

        if self.frames[len-1].variables.contains_key(name)
        {
            self.frames[len-1].variables.get_mut(name).unwrap().set(&evaluated);
        }
        else {
            self.error("variable does not exist");
        }
    }

    fn make_variable_of_type(&mut self, name: &String, variable_type: VariableType)
    {
        let len = self.frames.len();

        if self.is_numeric(name) || value_contains_operator(name) {
            self.error("invalid variable name");
        }

        if !self.frames[len-1].variables.contains_key(name) {
            self.frames[len-1].variables.insert(name.clone(), Variable { variable_type });
        }
        else {
            self.error("variable already exists");
        }
    }
}