use assert_fn::{assert, assert_eq};

#[derive(Debug)]
struct Error {}

mod assert {
    use super::*;

    #[assert]
    fn is_true(val: bool) -> bool {
        val
    }

    #[test]
    fn assert_is_true() {
        assert_is_true!(true);
        assert_is_true!(true, "with text");
    }

    #[test]
    #[should_panic(expected = "I'm gonna fail!")]
    fn assert_is_true_fail() {
        assert_is_true!(false, "I'm gonna fail!");
    }

    #[test]
    #[should_panic(expected = "I'm gonna fail!")]
    fn assert_is_true_fail_format() {
        assert_is_true!(false, "I'm gonna {}!", "fail");
    }

    #[assert(message = "Oh no!")]
    fn has_custom_message(val: bool) -> bool {
        val
    }

    #[test]
    #[should_panic(expected = "Oh no!")]
    fn assert_has_custom_message() {
        assert_has_custom_message!(false);
    }

    #[assert]
    fn both_true(val1: bool, val2: bool) -> bool {
        val1 && val2
    }

    #[test]
    fn assert_both_true() {
        assert_both_true!(true, true);
        assert_both_true!(true, true, "with text");
    }

    #[assert]
    async fn is_true_async(val: bool) -> bool {
        val
    }

    #[tokio::test]
    async fn assert_is_true_async() {
        assert_is_true_async!(true).await;
        assert_is_true_async!(true, "with text").await;
    }

    #[assert]
    fn is_true_result(val: bool) -> Result<bool, Error> {
        Ok(val)
    }

    #[test]
    fn assert_is_true_result() -> Result<(), Error> {
        let result = assert_is_true_result!(true);
        assert!(result.is_ok());

        assert_is_true_result!(true, "with text")?;

        Ok(())
    }

    #[assert]
    async fn is_true_result_async(val: bool) -> Result<bool, Error> {
        Ok(val)
    }

    #[tokio::test]
    async fn assert_is_true_result_async() -> Result<(), Error> {
        let result = assert_is_true_result_async!(true).await;
        assert!(result.is_ok());

        assert_is_true_result_async!(true, "with text").await?;

        Ok(())
    }
}

mod assert_eq {
    use super::*;

    #[assert_eq]
    fn eq_if_doubled(a: usize, b: usize) -> (usize, usize) {
        (a * 2, b)
    }

    #[test]
    fn assert_eq_if_doubled() {
        let (val, _) = assert_eq_if_doubled!(1, 2);
        assert_eq!(val, 2);

        assert_eq_if_doubled!(2, 4, "with text");
    }

    #[test]
    #[should_panic(expected = "1*2 != 1")]
    fn assert_eq_if_doubled_fail() {
        assert_eq_if_doubled!(1, 1, "1*2 != 1");
    }

    #[test]
    #[should_panic(expected = "1*2 != 1")]
    fn assert_eq_if_doubled_fail_format() {
        assert_eq_if_doubled!(1, 1, "{}*2 != {}", 1, 1);
    }

    #[assert_eq]
    fn eq_to_summed(a: usize, b: usize, c: usize, d: usize, sum: usize) -> (usize, usize) {
        (a + b + c + d, sum)
    }

    #[test]
    fn assert_eq_to_summed() {
        let (val, _) = assert_eq_to_summed!(1, 2, 3, 4, 10);
        assert_eq!(val, 10);

        assert_eq_to_summed!(1, 2, 3, 4, 10, "with text");
    }

    #[assert_eq]
    async fn eq_if_doubled_async(a: usize, b: usize) -> (usize, usize) {
        (a * 2, b)
    }

    #[tokio::test]
    async fn assert_eq_if_doubled_async() {
        assert_eq_if_doubled_async!(1, 2).await;
        assert_eq_if_doubled_async!(1, 2, "with text").await;
    }

    #[assert_eq]
    fn eq_if_doubled_result(a: usize, b: usize) -> Result<(usize, usize), Error> {
        Ok((a * 2, b))
    }

    #[test]
    fn assert_eq_if_doubled_result() -> Result<(), Error> {
        let result = assert_eq_if_doubled_result!(1, 2);
        assert!(result.is_ok());

        assert_eq_if_doubled_result!(1, 2, "with text")?;

        Ok(())
    }

    #[assert_eq]
    async fn eq_if_doubled_result_async(a: usize, b: usize) -> Result<(usize, usize), Error> {
        Ok((a * 2, b))
    }

    #[tokio::test]
    async fn assert_eq_if_doubled_result_async() -> Result<(), Error> {
        let result = assert_eq_if_doubled_result_async!(1, 2).await;
        assert!(result.is_ok());

        assert_eq_if_doubled_result_async!(1, 2, "with text").await?;

        Ok(())
    }
}
