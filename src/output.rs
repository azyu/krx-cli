use crate::error::Result;

pub fn print_json<T: serde::Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

pub fn print_text(value: &str) -> Result<()> {
    println!("{value}");
    Ok(())
}
