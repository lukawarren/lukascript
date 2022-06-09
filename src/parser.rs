use super::lexer::Token;
use super::lexer::TokenType;
use super::lexer::TokenType::*;
use super::variables::VariableType;
use super::variables::is_token_type_valid_type;
use super::variables::token_type_to_variable_type;
use super::common::error;

#[derive(Debug)]
pub enum Instruction
{
    NoOp,

    // Loops
    FromValueToValue { value: String, start: String, end: String },
    IfValue { left_value: String, last_line: usize },
    IfValueIsValue { left_value: String, right_value: String, last_line: usize },
    IfValueIsNotValue { left_value: String, right_value: String, last_line: usize },
    Done,

    // Functions
    FunctionDeclaration { name: String, first_line: usize, last_line: usize, arguments: Vec<(String, VariableType)> },
    FunctionCall { function: String, values: Vec<String>, target_variable: Option<String> },
    Return { value: String },

    // Variables
    IntDeclaration { name: String, value: String },
    BoolDeclaration { name: String, value: String },
    StringDeclaration { name: String, value: String },
    ArrayDeclaration { name: String },
    Assignment { name: String, value: String }
}

pub fn parse_lines(lines: &Vec<Vec<Token>>) -> Vec<Instruction>
{
    let mut instructions = Vec::<Instruction>::new();

    for i in 0..lines.len()
    {
        let tokens = &lines[i];

        if tokens.is_empty() {
            instructions.push(Instruction::NoOp);
        }

        else if tokens_contain_types(&tokens, &vec![For, Value, From, Value, To, Value])
        {
            instructions.push(Instruction::FromValueToValue {
                value: tokens[1].string.clone(),
                start: tokens[3].string.clone(),
                end: tokens[5].string.clone()
            });
        }

        else if tokens_contain_types(&tokens, &vec![If, Value])
        {
            instructions.push(Instruction::IfValue {
                left_value: tokens[1].string.clone(),
                last_line: get_corresponding_end_of_frame(lines, i)
            });
        }

        else if tokens_contain_types(&tokens, &vec![If, Value, Is, Value])
        {
            instructions.push(Instruction::IfValueIsValue {
                left_value: tokens[1].string.clone(),
                right_value: tokens[3].string.clone(),
                last_line: get_corresponding_end_of_frame(lines, i)
            });
        }

        else if tokens_contain_types(&tokens, &vec![If, Value, Is, Not, Value])
        {
            instructions.push(Instruction::IfValueIsNotValue {
                left_value: tokens[1].string.clone(),
                right_value: tokens[4].string.clone(),
                last_line: get_corresponding_end_of_frame(lines, i)
            });
        }

        else if tokens_contain_types(&tokens, &vec![Done]) {
            instructions.push(Instruction::Done);
        }

        else if tokens_begins_with_types(&tokens, &vec![Function, Value])
        {
            // Parse arguments, if any
            let mut arguments = Vec::<(String, VariableType)>::new();
            if tokens_begins_with_types(&tokens, &vec![Function, Value, Colon])
            {
                // Remove separating pipes
                let mut arg_tokens = tokens[3..tokens.len()].iter().collect::<Vec<&Token>>();
                arg_tokens.retain(|token| {
                    !matches!(token.token_type, TokenType::DoublePipe)
                });

                // Ensure valid types and non-overlapping variable names
                let mut variable_types = Vec::<VariableType>::new();
                let mut variable_names = Vec::<String>::new();
                for j in 0..arg_tokens.len()
                {
                    if j % 2 == 0
                    {
                        if !is_token_type_valid_type(&arg_tokens[j].token_type) {
                            error(format!("unknown variable type in function declaration on line {}", i + 1));
                        }

                        variable_types.push(token_type_to_variable_type(&arg_tokens[j].token_type));
                    }

                    else if j % 2 == 1
                    {
                        if variable_names.contains(&arg_tokens[j].string) {
                            error(format!("duplicate variable name in function declaration on line {}", i+1));
                        }

                        variable_names.push(arg_tokens[j].string.clone());
                    }
                }

                if variable_types.len() != variable_names.len() {
                    error(format!("unbalanced arguments in function declaration on line {}", i+1));
                }

                // Combine into tuple
                for j in 0..variable_types.len()
                {
                    arguments.push((variable_names[j].clone(), variable_types[j].clone()));
                }
            }

            instructions.push(Instruction::FunctionDeclaration {
                name: tokens[1].string.clone(),
                first_line: i,
                last_line: get_corresponding_end_of_frame(lines, i),
                arguments
            });
        }

        else if tokens_contain_types(&tokens, &vec![Int, Value, Equals, Value])
        {
            instructions.push(Instruction::IntDeclaration {
                name: tokens[1].string.clone(),
                value: tokens[3].string.clone()
            });
        }

        else if tokens_contain_types(&tokens, &vec![Bool, Value, Equals, Value])
        {
            instructions.push(Instruction::BoolDeclaration {
                name: tokens[1].string.clone(),
                value: tokens[3].string.clone()
            });
        }

        else if tokens_contain_types(&tokens, &vec![Str, Value, Equals, Value])
        {
            instructions.push(Instruction::StringDeclaration {
                name: tokens[1].string.clone(),
                value: tokens[3].string.clone()
            });
        }

        else if tokens_contain_types(&tokens, &vec![Array, Value])
        {
            instructions.push(Instruction::ArrayDeclaration {
                name: tokens[1].string.clone()
            });
        }

        else if tokens_contain_types(&tokens, &vec![Value, Equals, Value])
        {
            instructions.push(Instruction::Assignment {
                name: tokens[0].string.clone(),
                value: tokens[2].string.clone()
            });
        }

        else if tokens_begins_with_types(&tokens, &vec![Value, LeftBracket]) &&
                tokens_ends_with_type(&tokens, &vec![RightBracket])
        {
            let arguments: Vec<String> = tokens[2..tokens.len()-1].
                                            iter().map(|t| t.string.clone()).collect();

            instructions.push(Instruction::FunctionCall {
                function: tokens[0].string.clone(),
                values: arguments.clone(),
                target_variable: None
            });
        }

        else if tokens_begins_with_types(&tokens, &vec![Value, LeftBracket]) &&
                tokens_ends_with_type(&tokens, &vec![RightBracket, RightArrow, Value])
        {
            let arguments: Vec<String> = tokens[2..tokens.len()-3].
                iter().map(|t| t.string.clone()).collect();

            instructions.push(Instruction::FunctionCall {
                function: tokens[0].string.clone(),
                values: arguments.clone(),
                target_variable: Some(tokens[tokens.len()-1].string.clone())
            });
        }

        else if tokens_contain_types(&tokens, &vec![Return, Value])
        {
            instructions.push(Instruction::Return {
                value: tokens[1].string.clone()
            });
        }

        else {
            error(format!("unknown instruction on line {}:\n{:#?}", i + 1, lines[i]));
        }
    }

    instructions
}

fn tokens_contain_types(line: &Vec<Token>, types: &Vec<TokenType>) -> bool
{
    if line.len() != types.len() { return false }
    for i in 0..types.len()  {
        if line[i].token_type != types[i] { return false }
    }
    true
}

fn tokens_begins_with_types(line: &Vec<Token>, types: &Vec<TokenType>) -> bool
{
    if line.len() < types.len() { return false }
    for i in 0..types.len() {
        if line[i].token_type != types[i] { return false }
    }
    true
}

fn tokens_ends_with_type(line: &Vec<Token>, types: &Vec<TokenType>) -> bool
{
    if line.len() < types.len() { return false }
    let first_tested_element = line.len() - types.len();

    for i in 0..types.len()
    {
        if line[first_tested_element + i].token_type != types[i] {
            return false
        }
    }

    true
}

fn get_corresponding_end_of_frame(lines: &Vec<Vec<Token>>, line: usize) -> usize
{
    let frame_tokens = vec![For, If, Function];
    let mut inner_frames = 1;

    for i in (line+1)..lines.len()
    {
        if lines[i].len() != 0
        {
            let first_token = &lines[i][0].token_type;
            if frame_tokens.contains(&first_token) { inner_frames += 1; }
            else if matches!(first_token, Done) { inner_frames -= 1; }

            if inner_frames == 0 {
                return i;
            }
        }
    }

    error(format!("frame declared on line {} does not terminate", line));
}