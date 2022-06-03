use super::lexer::Token;
use super::lexer::TokenType;
use super::lexer::TokenType::*;
use super::common::error;

#[derive(Debug)]
pub enum Instruction
{
    NoOp,

    // Loops
    FromValueToValue { value: String, start: String, end: String },
    IfValueIsValue { left_value: String, right_value: String, last_line: usize },
    IfValueIsNotValue { left_value: String, right_value: String, last_line: usize },
    Done,

    // Functions
    FunctionDeclaration { name: String, first_line: usize, last_line: usize },
    FunctionCall { function: String, values: Vec<String> },

    // Variables
    IntDeclaration { name: String, value: String },
    BoolDeclaration { name: String, value: String },
    StringDeclaration { name: String, value: String },
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

        else if tokens_contain_types(&tokens, &vec![Function, Value])
        {
            instructions.push(Instruction::FunctionDeclaration {
                name: tokens[1].string.clone(),
                first_line: i,
                last_line: get_corresponding_end_of_frame(lines, i)
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

        else if tokens_contain_types(&tokens, &vec![Value, Equals, Value])
        {
            instructions.push(Instruction::Assignment {
                name: tokens[0].string.clone(),
                value: tokens[2].string.clone()
            });
        }

        else if tokens_begins_with_types(&tokens, &vec![Value])
        {
            let next_tokens: Vec<String> = tokens[1..tokens.len()].iter().map(|t| t.string.clone()).collect();
            instructions.push(Instruction::FunctionCall {
                function: tokens[0].string.clone(),
                values: next_tokens.clone()
            });
        }

        else {
            error(format!("unknown instruction on line {}", i + 1));
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