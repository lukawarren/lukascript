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
    Int,
    Bool,
    Str,
    If,
    Is,
    Not,
    Multiply
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
        tokenised_lines.push(get_tokens_for_line(&line));
    }

    tokenised_lines
}

fn get_tokens_for_line(input: &String) -> Vec<Token>
{
    let mut characters = input.chars().collect::<Vec<char>>();
    let mut tokens = Vec::<Token>::new();
    let mut word = Vec::<char>::new();
    let mut inside_string = false;

    // The below code will miss out the last word sometimes, so as a simple fix to avoid duplicating code, it's easier to just... add another word :)
    characters.push(' ');

    // March along until we find each new token (which is usually just a single word, but could also be a quote also, hence the roundabout way)
    for i in 0..characters.len()
    {
        if characters[i] == '"' {
            inside_string = !inside_string;
        }

        if characters[i] == ' ' && !inside_string
        {
            if !word.is_empty()
            {
                // New token found
                tokens.push(Token {
                    token_type: get_token_from_word(&word),
                    string: word.iter().collect()
                });

                word.clear();
            }
        }
        else { word.push(characters[i]); }
    }

    collect_operators(&mut tokens);

    if inside_string {
        println!("Error: unterminated string on line: {}", input);
        std::process::exit(1);
    }

    tokens
}

fn get_token_from_word(input: &Vec<char>) -> TokenType
{
    match input.iter().collect::<String>().as_str()
    {
        "=" => TokenType::Equals,
        "for" => TokenType::For,
        "from" => TokenType::From,
        "to" => TokenType::To,
        "done" => TokenType::Done,
        "fn" => TokenType::Function,
        "int" => TokenType::Int,
        "bool" => TokenType::Bool,
        "string" => TokenType::Str,
        "if" => TokenType::If,
        "is" => TokenType::Is,
        "not" => TokenType::Not,
        "*" => TokenType::Multiply,
        _ => TokenType::Value
    }
}