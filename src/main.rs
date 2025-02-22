use ratio_calc::Rational;

fn main() {
    for line in std::io::stdin().lines() {
        let Ok(line) = line else { break };
        let res = Rational::run_expr(&line);

        // parse
        println!("{:?}", res);
    }
}
