use super::parser::Instruction;
use super::parser::Instruction::*;
use super::variables::Variable;
use super::variables::VariableType;
use crate::variables::is_str_valid_type;
use super::operators::value_contains_operator;
use super::operators::is_char_operator;
use super::operators::operator_char_to_token_type;
use super::operators::evaluate_operator_expression;
use super::operators::OperatorExpression;
use super::stdlib::stdlib_function;
use super::common::error;

use std::collections::HashMap;

type FunctionInfo = (usize, Vec<(String, VariableType)>);

#[derive(Clone)]
enum Frame
{
    Root,
    ForLoop { variable: String, start_line: usize, end_value: String },
    Function { caller_line: usize, target_variable: Option<String> },
    IfStatement
}

struct FrameContext
{
    frame: Frame,
    variables: HashMap<String, Variable>,
    functions: HashMap<String, FunctionInfo> // beginning line, arguments
}

impl FrameContext
{
    pub fn clear(&mut self)
    {
        self.variables.clear();
        self.functions.clear();
    }
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
        self.make_variable_of_type(&String::from("true"), &VariableType::Boolean(true));
        self.make_variable_of_type(&String::from("false"), &VariableType::Boolean(false));

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

                        self.make_variable_of_type(value, &VariableType::Integer(0));
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

                FunctionDeclaration { name, first_line, last_line, arguments } =>
                {
                    // Note function then sally on forth
                    if self.innermost_frame().functions.insert(name.clone(), (*first_line, arguments.clone())).is_some() {
                        self.error("function already declared");
                    }
                    self.line = *last_line;
                },

                FunctionCall { function, values, target_variable } =>
                {
                    // Check for user-defined functions first, then if that fails, assume it's in-built
                    let mut found_function = Option::<FunctionInfo>::default();
                    self.for_each_frame(|frame, _| {
                        if found_function.is_none() && frame.functions.contains_key(function) {
                            let _ = found_function.insert(frame.functions.get(function).unwrap().clone());
                        }
                    });

                    if found_function.is_some()
                    {
                        self.add_frame(Frame::Function { caller_line: self.line, target_variable: target_variable.clone() });

                        // Check argument lengths match
                        let desired_args = &found_function.as_ref().unwrap().1;
                        if desired_args.len() != values.len() {
                            self.error("invalid number of function arguments");
                        }

                        // Pass arguments
                        for i in 0..desired_args.len()
                        {
                            let name = &desired_args[i].0;
                            let variable_type = desired_args[i].1.clone();

                            // Be careful to evaluate the value early, before we make the new one, as if they
                            // have the same name, we'll accidentally use the new one in any evaluating, as may
                            // happen in recursive functions.
                            let value_evaluated_early = self.evaluate_value(&values[i]);
                            self.make_variable_of_type(name, &variable_type);
                            self.get_variable(name).set(&value_evaluated_early);
                        }

                        self.line = found_function.as_ref().unwrap().0;
                    }

                    // Function not found, assume part of the "standard library"
                    else
                    {
                        // Evaluate arguments first
                        let arguments = values.iter().map(|v| {
                            self.evaluate_value(v)
                        }).collect();

                        // Run function (if any)
                        let stdlib_result = stdlib_function(function.as_str(), &arguments);
                        let stdlib_did_run = stdlib_result.0;
                        let stdlib_return = stdlib_result.1;

                        if stdlib_did_run
                        {
                            // Standard library function was found, set target variable if need be
                            if target_variable.is_some() && stdlib_return.is_some()
                            {
                                self.make_variable_of_type(target_variable.as_ref().unwrap(), &stdlib_return.as_ref().unwrap().variable_type);
                                self.get_variable(&target_variable.as_ref().unwrap()).set(&stdlib_return.unwrap());
                            }
                        }
                        else {
                            self.error("unknown function");
                        }
                    }
                },

                Return { value } =>
                {
                    // Search for function frame (if any)
                    let mut frame_info = Option::<(usize, Option<String>)>::default();
                    let mut frame_index = Option::<usize>::default();

                    self.for_each_frame(|frame, index|
                    {
                        if frame_info.is_none()
                        {
                            match &frame.frame
                            {
                                Frame::Function { caller_line, target_variable } =>
                                {
                                    let _ = frame_info.insert((caller_line.clone(), target_variable.clone()));
                                    let _ = frame_index.insert(index);
                                }
                                _ => {}
                            }
                        }
                    });

                    if frame_info.is_some()
                    {
                        // We can't just pop the current frame off because we may be returning from a function,
                        // but within an if statement, for example, so instead we need to put potentially more
                        // than once!

                        let target_variable = &frame_info.as_ref().unwrap().1;
                        let line_number = frame_info.as_ref().unwrap().0;

                        // Evaluate returned variable first, before we pop the frame
                        if target_variable.is_some()
                        {
                            let evaluated = self.evaluate_value(value);

                            for _ in 0..(self.frames.len()-frame_index.unwrap()) {
                                self.frames.pop();
                            }

                            self.make_variable_of_type(target_variable.as_ref().unwrap(), &evaluated.variable_type);
                            self.get_variable(target_variable.as_ref().unwrap()).set(&evaluated);
                            self.line = line_number; // Set last so error names carrying line numbers make sense
                        }
                        else
                        {
                            for _ in 0..(self.frames.len()-frame_index.unwrap()) {
                                self.frames.pop();
                            }

                            self.line = line_number;
                        }
                    }
                    else { self.error("cannot return outside of a function"); }
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
                                // Loop back, but start with (essentially) a new frame
                                *self.get_variable(&variable) += one.clone();
                                let variable_backup = self.get_variable(&variable).clone();
                                self.innermost_frame().clear();
                                self.innermost_frame().variables.insert(variable, variable_backup);
                                self.line = start_line;
                            }
                        },

                        Frame::Function { caller_line, target_variable } =>
                        {
                            self.frames.pop();

                            if target_variable.is_some()
                            {
                                // No value was returned, so raise error
                                self.error("function did not return valid value")
                            }

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

                IntDeclaration { name, value } =>
                {
                    // Evaluate first, before the variable is created, to prevent stuff like "int foo = foo"
                    let evaluated = self.evaluate_value(&value);
                    self.make_variable_of_type(name, &VariableType::Integer(0));
                    self.get_variable(name).set(&evaluated);
                },

                BoolDeclaration { name, value } =>
                {
                    // Evaluate first, before the variable is created, to prevent stuff like "int foo = foo"
                    let evaluated = self.evaluate_value(&value);
                    self.make_variable_of_type(name, &VariableType::Boolean(false));
                    self.get_variable(name).set(&evaluated);
                },

                StringDeclaration { name, value } =>
                {
                    // Evaluate first, before the variable is created, to prevent stuff like "int foo = foo"
                    let evaluated = self.evaluate_value(&value);
                    self.make_variable_of_type(name, &VariableType::Str(String::new()));
                    self.get_variable(name).set(&evaluated);
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
            functions: HashMap::<String, FunctionInfo>::new()
        });
    }

    fn for_each_frame<F: FnMut(&FrameContext, usize)>(&self, mut f: F)
    {
        // Go from inner-most frame to root (i.e. reversed)
        for i in 1..=self.frames.len()
        {
            let index = self.frames.len() - i;
            f(&self.frames[index], index);
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

        self.print_variables();
        self.error(format!("variable \"{}\" does not exist", name).as_str());
    }

    fn set_variable(&mut self, name: &String, value: &String)
    {
        let evaluated = self.evaluate_value(value);
        self.get_variable(name).set(&evaluated);
    }

    fn make_variable_of_type(&mut self, name: &String, variable_type: &VariableType)
    {
        let len = self.frames.len();

        if self.is_numeric(name) || value_contains_operator(name) || is_str_valid_type(name.as_str()) {
            self.error("invalid variable name");
        }

        if !self.frames[len-1].variables.contains_key(name) {
            self.frames[len-1].variables.insert(name.clone(), Variable { variable_type: variable_type.clone() });
        }
        else {
            self.error("variable already exists");
        }
    }
}