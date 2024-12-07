use currency::*;

fn main() -> std::result::Result<(), i32> {
    let err_code = run();
    if err_code != 0 {
        return Err(err_code);
    }
    Ok(())
}
