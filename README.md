# lambda-eval
Simple Lambda Calculus evaluator with [hvm-lang syntax](https://github.com/HigherOrderCO/hvm-lang#language-syntax)

```
Usage: lambda-eval [OPTIONS] <MODE> <PATH>

Arguments:
  <MODE>  [possible values: run, run-stepped, interactive]
  <PATH>  Path to the input file

Options:
  -f, --form <FORM>    [default: nf] [possible values: nf, wnf, hnf, whnf]
  -o, --order <ORDER>  [default: normal] [possible values: normal, applicative]
  -h, --help           Print help
  -V, --version        Print version
```