mod accounts;

fn main() {
    for (key, value) in accounts::get_accounts().iter() {
        println!("{} {}", key, value);
    }
}
