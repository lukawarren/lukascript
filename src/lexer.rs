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
    RightArrow,
    Return,
    Int,
    Bool,
    Str,
    Array,
    If,
    Is,
    Not,
    Multiply,
    Minus,
    LessThan,
    GreaterThan,
    LeftBracket,
    RightBracket
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
    // of a variable called "my_integer", for example. However, if we're inside a string,
    // no new tokens may arise until the string terminates.

    let mut tokens = Vec::<Token>::new();
    let mut word = String::new();
    let mut inside_string = false;
    let mut comment_reached = false;

    // Ignore empty lines
    if input.is_empty() { return tokens }

    for i in 0..input.len()
    {
        if comment_reached { continue }

        let char = input.chars().nth(i).unwrap();
        let single_found = is_single_token(char, input.chars().nth(i+1));

        // Add character to buffer, even if it's a string quote
        word.push(char);

        // Detect comments
        if i < input.len() - 2 && char == '/' && input.chars().nth(i+1).unwrap() == '/'
        {
            // Still process word buffer, but don't include this character, and stop after
            word.pop();
            comment_reached = true;
        }

        // Keep track of state
        if char == '\"' {
            inside_string = !inside_string;
        }
        let string_ended = char == '\"' && !inside_string;
        let normal_word_ended = !inside_string && !single_found && (char == ' ' || i == input.len()-1);

        // If a string's on-going
        if inside_string {}

        // If a string or a normal word just ended, or this is the last loop iteration because we've just found a comment
        if string_ended || normal_word_ended || comment_reached
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

        // If we've just run into a "single token", meaning the previous chars were a token too
        else if single_found && !inside_string
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
    tokens
}

fn is_single_token(c: char, next_char: Option<char>) -> bool
{
    // We must be conscious of the next character (if there is one), because if the whole word
    // is something like "->", then seeing a "-" character is, in this case, not actually a
    // "single token".

    match c
    {
        '=' |
        ':' |
        '(' |
        ')' |
        '*' |
        '-' => next_char.is_none() || next_char.unwrap() != '>',
        '<' |
        '>' |
        _ => false
    }
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
        "->" => TokenType::RightArrow,
        "(" => TokenType::LeftBracket,
        ")" => TokenType::RightBracket,
        "return" => TokenType::Return,
        "int" => TokenType::Int,
        "bool" => TokenType::Bool,
        "string" => TokenType::Str,
        "array" => TokenType::Array,
        "if" => TokenType::If,
        "is" => TokenType::Is,
        "not" => TokenType::Not,
        "*" => TokenType::Multiply,
        "-" => TokenType::Minus,
        "<" => TokenType::LessThan,
        ">" => TokenType::GreaterThan,
        _ => TokenType::Value
    }
}
