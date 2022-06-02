pub mod lexer;
pub mod parser;
pub mod engine;
pub mod variables;
pub mod common;

use std::fs;
use std::env;

fn main()
{
    // Get debug mode
    let mut debug = false;
    for argument in env::args()
    {
        if argument == "--debug" {
            debug = true;
        }
    }

    let lines: Vec<String> = fs::read_to_string("./src.txt").expect("Could not locate source file, src.txt")
    .lines()
    .map(|l| String::from(l))
    .collect();

    let lexer_output = lexer::tokenise_lines(&lines);
    let parser_output = parser::parse_lines(&lexer_output);

    if debug
    {
        println!("{:#?}", lexer_output);
        println!("{:#?}", parser_output);
    }

    let mut state = engine::State::default();
    state.execute(parser_output);

    if debug { state.print_variables(); }
}