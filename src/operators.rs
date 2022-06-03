use super::lexer::Token;
use super::lexer::TokenType;
use super::variables::Variable;

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
        _ => TokenType::Value
    }
}

fn is_token_operator(token_type: &TokenType) -> bool
{
    match token_type
    {
        TokenType::Multiply => true,
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
        for i in 1..tokens.len()
        {
            if matches!(tokens[i-1].token_type, TokenType::Value) &&
                matches!(tokens[i+1].token_type, TokenType::Value)
            {
                if matches!(tokens[i].token_type, TokenType::Multiply)
                {
                    let right_value = tokens.remove(i + 1);
                    let left_value = tokens.remove(i - 1);

                    tokens[i - 1] = Token
                    {
                        token_type: TokenType::Value,
                        string: format!("{}*{}", left_value.string, right_value.string)
                    };

                    break;
                }
            }
        }

        if tokens_contain_valid_operator(tokens) {
            collect_operators(tokens);
        }
    }
}

pub fn evaluate_operator_expression(expression: &Vec<OperatorExpression>) -> Variable
{
    println!("Evaluating operator expression {:#?}", expression);

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

                        _ => { todo!(); }
                    }

                    last_was_variable = true;
                }
            }
        }

        initial_variable
    } else { panic!(); }
}