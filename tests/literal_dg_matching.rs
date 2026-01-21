#[cfg(test)]
mod tests {
    use rsgrep::*;

    #[test]
    fn match_single_literal_ch() {
        is_rgrep_built();

        let result1 = run_rgrep_from_root("echo -n '123'", "./target/release/rgrep -E '\\d' ");
        assert!(result1);

        let result2 = run_rgrep_from_root("echo -n 'apple'", "./target/release/rgrep -E '\\d'");
        assert!(!result2);

        let result3 = run_rgrep_from_root("echo -n 'abc_0_xyz'", "./target/release/rgrep -E '\\d'");
        assert!(result3);
    }
}
