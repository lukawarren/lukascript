use super::variables::Variable;
use crate::variables::VariableType;
use std::io;
use std::io::Write;

// Returns if the function exists, followed by an optional variable returned
pub fn stdlib_function(function: &str, arguments: &Vec<Variable>) -> (bool, Option<Variable>)
{
    match function
    {
        "print" =>
        {
            for argument in arguments {
                print!("{}", argument.printed_string());
            }
            println!();

            (true, None)
        },

        "input" =>
        {
            // Print then flush to ensure we actually output before we take input
            for argument in arguments {
                print!("{}", argument.printed_string());
            }
            io::stdout().flush().unwrap();

            // Get input itself, removing newlines at the same time
            let input = io::stdin().lines().next().unwrap().unwrap();

            (true, Some(Variable {
                variable_type: VariableType::Str(input)
            }))
        }

        _ => (false, None)
    }
}