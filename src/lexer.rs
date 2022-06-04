use super::operators::collect_operators;

#[derive(PartialEq, Debug, Clone)]
pub enum TokenType
{
    Value,
    Equals,
    For,
    From,
    To,
    Done,
    Function,
    Colon,
    DoublePipe,
    LeftBracket,
    RightBracket,
    Return,
    Int,
    Bool,
    Str,
    If,
    Is,
    Not,
    Multiply,
    Minus
}

#[derive(Debug)]
pub struct Token
{
    pub token_type: TokenType,
    pub string: String
}

impl Token
{
    pub fn as_number(&self) -> usize
    {
        self.string.parse::<usize>().unwrap()
    }
}

pub fn tokenise_lines(lines: &Vec<String>) -> Vec<Vec<Token>>
{
    let mut tokenised_lines = Vec::<Vec<Token>>::new();

    for line in lines {
        tokenised_lines.push(
            get_tokens_from_line(
                &line.trim().to_string()
            )
        );
    }

    tokenised_lines
}

fn get_tokens_from_line(input: &String) -> Vec<Token>
{
    // There are some tokens that, if found, are definitely tokens, regardless of spaces
    // (e.g. a bracket anywhere is always a bracket, as is a "*", but "int" might be part
    // of a variable called "my_integer", for example.

    let mut tokens = Vec::<Token>::new();
    let mut word = String::new();

    // Ignore empty lines
    if input.is_empty() { return tokens }

    for i in 0..input.len()
    {
        let char = input.chars().nth(i).unwrap();
        word.push(char);

        let single_found = match char
        {
            '=' |
            ':' |
            '(' |
            ')' |
            '*' |
            '-' => true,
            _ => false
        };

        // If a word's just ended
        if !single_found && (char == ' ' || i == input.len()-1)
        {
            if char == ' ' {
                word.pop();
            }

            if !word.is_empty()
            {
                tokens.push(Token {
                    token_type: token_from_string(&word),
                    string: word.clone()
                });
                word.clear();
            }
        }

        else if single_found
        {
            // Last character in word is actually our single token,
            // and everything before is its own token
            word.pop();
            if !word.is_empty()
            {
                tokens.push(Token {
                    token_type: token_from_string(&word),
                    string: word.clone()
                });
            }

            tokens.push(Token {
                token_type: token_from_string(&char.to_string()),
                string: char.to_string()
            });
            word.clear();
        }
    }

    collect_operators(&mut tokens);
    collect_function_calls(&mut tokens);
    tokens
}

fn token_from_string(input: &String) -> TokenType
{
    match input.chars().collect::<String>().as_str()
    {
        "=" => TokenType::Equals,
        "for" => TokenType::For,
        "from" => TokenType::From,
        "to" => TokenType::To,
        "done" => TokenType::Done,
        "fn" => TokenType::Function,
        ":" => TokenType::Colon,
        "||" => TokenType::DoublePipe,
        "(" => TokenType::LeftBracket,
        ")" => TokenType::RightBracket,
        "return" => TokenType::Return,
        "int" => TokenType::Int,
        "bool" => TokenType::Bool,
        "string" => TokenType::Str,
        "if" => TokenType::If,
        "is" => TokenType::Is,
        "not" => TokenType::Not,
        "*" => TokenType::Multiply,
        "-" => TokenType::Minus,
        _ => TokenType::Value
    }
}

fn collect_function_calls(tokens: &mut Vec<Token>)
{
    // When a function is called, in the format [value] [left bracket] [value(s)] [right bracket], collapse
    // all these into a single value, in a similar way to how operators are dealt with.

    todo!();
}