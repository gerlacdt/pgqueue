pub fn hello() {
    println!("Hello World!");
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn hello_test() {
        hello();
        assert!(true);
    }
}
