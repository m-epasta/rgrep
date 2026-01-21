#[cfg(test)]
mod tests {
    use rsgrep::*;

    #[test]
    fn match_single_literal_ch() {
        is_rgrep_built();

        let result1 = run_rgrep_from_root("echo -n 'apple'", "./target/release/rgrep -E '\\w'");
        assert!(result1);

        let result2 = run_rgrep_from_root("echo -n 'BLUEBERRY'", "./target/release/rgrep -E '\\w'");
        assert!(result2);

        let result3 = run_rgrep_from_root("echo -n '369'", "./target/release/rgrep -E '\\w'");
        assert!(result3);

        let result4 = run_rgrep_from_root("echo -n '=%+_×#%'", "./target/release/rgrep -E '\\w'");
        assert!(result4);

        let result5 = run_rgrep_from_root("echo -n '#%=-÷×'", "./target/release/rgrep -E '\\w'");
        assert!(!result5);
    }
}
