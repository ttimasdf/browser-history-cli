pub fn print_header(sep: char, columns: &[&str]) {
    let line = columns.join(&sep.to_string());
    println!("{}", line);
}

pub fn sep_for_format(format: &str) -> char {
    match format {
        "csv" => ',',
        _ => '\t',
    }
}