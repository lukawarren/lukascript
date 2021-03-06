use super::lexer::Token;
use super::lexer::TokenType;
use super::variables::Variable;
use super::variables::VariableType;

#[derive(Debug, Clone)]
pub enum OperatorExpression
{
    Variable(Variable),
    Operator(TokenType)
}

pub fn operator_char_to_token_type(c: char) -> TokenType
{
    match c
    {
        '*' => TokenType::Multiply,
        '-' => TokenType::Minus,
        '<' => TokenType::LessThan,
        '>' => TokenType::GreaterThan,
        _ => TokenType::Value
    }
}

fn is_token_operator(token_type: &TokenType) -> bool
{
    match token_type
    {
        TokenType::Multiply => true,
        TokenType::Minus => true,
        TokenType::LessThan => true,
        TokenType::GreaterThan => true,
        _ => false
    }
}

pub fn is_char_operator(c: char) -> bool
{
    is_token_operator(&operator_char_to_token_type(c))
}

pub fn value_contains_operator(value: &String) -> bool
{
    for c in value.chars()
    {
        if is_char_operator(c) { return true }
    }
    false
}

pub fn tokens_contain_valid_operator(tokens: &Vec<Token>) -> bool
{
    if tokens.len() < 3 { return false }

    for i in 0..tokens.len()
    {
        if is_token_operator(&tokens[i].token_type) &&
            i != tokens.len()-1 && i != 0 &&
            matches!(tokens[i-1].token_type, TokenType::Value) &&
            matches!(tokens[i+1].token_type, TokenType::Value)
        {
            return true
        }
    }

    false
}

/*
    Responsible for taking a sequence of tokens and combining then when maths is involved.
    For example, [Value, Multiply, Value] should just become [Value], as it is evaluated
    later. Values such as "a*b" are already in the form desired.
*/
pub fn collect_operators(tokens: &mut Vec<Token>)
{
    if tokens_contain_valid_operator(tokens)
    {
        let mut i = 1;

        while i < tokens.len()-1
        {
            if matches!(tokens[i-1].token_type, TokenType::Value) &&
                is_token_operator(&tokens[i].token_type) &&
                matches!(tokens[i+1].token_type, TokenType::Value)
            {

                let right_value = tokens.remove(i + 1);
                let left_value = tokens.remove(i - 1);

                let token = |c|
                {
                    Token
                    {
                        token_type: TokenType::Value,
                        string: format!("{}{}{}", left_value.string, c, right_value.string)
                    }
                };

                     if matches!(tokens[i-1].token_type, TokenType::Multiply)    { tokens[i - 1] = token('*'); break; }
                else if matches!(tokens[i-1].token_type, TokenType::Minus)       { tokens[i - 1] = token('-'); break; }
                else if matches!(tokens[i-1].token_type, TokenType::LessThan)    { tokens[i - 1] = token('<'); break; }
                else if matches!(tokens[i-1].token_type, TokenType::GreaterThan) { tokens[i - 1] = token('>'); break; }
            }

            i += 1;
        }

        if tokens_contain_valid_operator(tokens) {
            collect_operators(tokens);
        }
    }
}

pub fn evaluate_operator_expression(expression: &Vec<OperatorExpression>) -> Variable
{
    if let OperatorExpression::Variable(mut initial_variable) = expression[0].clone()
    {
        let mut last_was_variable = true;
        let mut last_operator = TokenType::Multiply;

        for i in 1..expression.len()
        {
            match expression[i].clone()
            {
                OperatorExpression::Operator(token_type) =>
                {
                    if !last_was_variable { panic!(); }
                    last_operator = token_type;
                    last_was_variable = false;
                },

                OperatorExpression::Variable(variable) =>
                {
                    if last_was_variable { panic!(); }

                    // Actually perform operation
                    match last_operator
                    {
                        TokenType::Multiply =>
                        {
                            initial_variable *= variable;
                        },

                        TokenType::Minus =>
                        {
                            initial_variable -= variable;
                        },

                        TokenType::LessThan =>
                        {
                            initial_variable = Variable {
                                variable_type: VariableType::Boolean(
                                    initial_variable < variable
                                )
                            }
                        },

                        TokenType::GreaterThan =>
                        {
                            initial_variable = Variable {
                                variable_type: VariableType::Boolean(
                                    initial_variable > variable
                                )
                            }
                        },

                        _ => { todo!(); }
                    }

                    last_was_variable = true;
                }
            }
        }

        initial_variable
    } else { panic!(); }
}