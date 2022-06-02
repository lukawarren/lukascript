pub fn error(message: String) -> !
{
    println!("Error: {}", message);
    std::process::exit(1);
}