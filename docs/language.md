# cxgraph language

cxgraph uses a custom expression language that is compiled to WGSL.

## names

names must begin with any alphabetic character (lowercase or capital letters, Greek letters,
etc.) and may contain alphanumeric chararcters as well as underscores (`_`) and apostrophes (`'`).
the words `sum`, `prod`, and `iter` may not be used for names. names may refer to either
functions or variables.

examples of names include:
```
a  A  aaa  ω  z_3  __5__  f'  К'םαl'けx焼__검
```

names may either be **built-in**, **global**, or **local**. global or local names may shadow
built-in names, and local names may shadow global ones.

## declarations

a **function declaration** declares a new function. functions may have zero or more arguments.

```
f(x) = 3
```

a **constant declaration** declares a new constant.

```
n = 5
```

declarations are separated by newlines. declarations may only reference functions and constants in
declarations that precede them. the name used in a declaration (`f` and `n` in the above examples)
is in the global scope

## built-ins


arithmetic functions:
| name          | description                                           |
|---------------|-------------------------------------------------------|
| `pos(z)`      | equivalent to unary `+`                               |
| `neg(z)`      | equivalent to unary `-`                               |
| `conj(z)`     | complex conjugate, equivalent to unary `*`            |
| `re(z)`       | real part                                             |
| `im(z)`       | imaginary part                                        |
| `abs(z)`      | absolute value (distance from `0`)                    |
| `abs_sq(z)`   | square of absolute value                              |
| `arg(z)`      | argument (angle about `0`) in the range `(-τ/2, τ/2]` |
| `argbr(z,br)` | argument in the range `(-τ/2, τ/2] + br`              |
| `add(z,w)`    | equivalent to `z + w`                                 |
| `sub(z,w)`    | equivalent to `z - w`                                 |
| `mul(z,w)`    | equivalent to `z * w`                                 |
| `div(z,w)`    | equivalent to `z / w`                                 |
| `recip(z)`    | reciprocal, equivalent to `1/z`                       |

power/exponential functions:
| name          | description                               |
|---------------|-------------------------------------------|
| `exp(z)`      | exponential function, equivalent to `e^z` |
| `log(z)`      | logarithm base `e`                        |
| `logbr(z,br)` | logarithm base `e` with specified branch  |
| `pow(z)`      | power, equivalent to `^`                  |
| `powbr(z,br)` | `pow` with specified branch               |
| `sqrt(z)`     | square root, equivalent to `z^0.5`        |
| `sqrtbr(z,br)`| square root with specified branch         |
| `cbrt(z)`     | cube root, equivalent to `z^0.5`          |
| `cbrtbr(z,br)`| cube root with specified branch           |

trigonometric functions:
| name       | description                         |
|------------|-------------------------------------|
| `sin(z)`   | sine function                       |
| `cos(z)`   | cosine function                     |
| `tan(z)`   | tangent function                    |
| `sinh(z)`  | hyperbolic sine function            |
| `cosh(z)`  | hyperbolic cosine function          |
| `tanh(z)`  | hyperbolic tangent function         |
| `asin(z)`  | inverse sine function               |
| `acos(z)`  | inverse cosine function             |
| `atan(z)`  | inverse tangent function            |
| `asinh(z)` | inverse hyperbolic sine function    |
| `acosh(z)` | inverse hyperbolic cosine function  |
| `atanh(z)` | inverse hyperbolic tangent function |

special functions:
| function                 | description                                                        |
|--------------------------|--------------------------------------------------------------------|
| `gamma(z)`, `Γ(z)`       | [gamma function](https://en.wikipedia.org/wiki/Gamma_function)     |
| `invgamma(z)`, `invΓ(z)` | reciprocal of the gamma function                                   |
| `loggamma(z)`, `logΓ(z)` | logarithm of the gamma function                                    |
| `digamma(z)`, `ψ(z)`     | [digamma function](https://en.wikipedia.org/wiki/Digamma_function) |

logic functions:
| function        | description                                                                |
|-----------------|----------------------------------------------------------------------------|
| `signre(z)`     | sign of real part (1 if `re(z) > 0`, -1 if `re(z) < 0`, 0 if `re(z) == 0`) |
| `signim(z)`     | sign of imaginary part                                                     |
| `ifgt(p,q,z,w)` | evaluates to `z` if `re(p) > re(q)`, otherwise `w`                         |
| `iflt(p,q,z,w)` | evaluates to `z` if `re(p) < re(q)`, otherwise `w`                         |
| `ifge(p,q,z,w)` | evaluates to `z` if `re(p) ≥ re(q)`, otherwise `w`                         |
| `ifle(p,q,z,w)` | evaluates to `z` if `re(p) ≤ re(q)`, otherwise `w`                         |
| `ifeq(p,q,z,w)` | evaluates to `z` if `re(p) = re(q)`, otherwise `w`                         |
| `ifne(p,q,z,w)` | evaluates to `z` if `re(p) ≠ re(q)`, otherwise `w`                         |
| `ifnan(p,z,w)`  | evaluates to `z` if `p` is `NaN`, otherwise `w`                            |

constants:
| name           | description                                                                                            |
|----------------|--------------------------------------------------------------------------------------------------------|
| `i`            | the imaginary constant, equal to `sqrt(-1)`                                                            |
| `e`            | the [exponential constant](https://en.wikipedia.org/wiki/E_(mathematical_constant)), equal to `exp(1)` |
| `tau`, `τ`     | the [circle constant](https://tauday.com/tau-manifesto)                                                |
| `emgamma`, `γ` | the [Euler-Mascheroni](https://en.wikipedia.org/wiki/Euler%27s_constant) constant, equal to `-ψ(1)`    |
| `phi`, `φ`     | the [golden ratio](https://en.wikipedia.org/wiki/Golden_ratio), equal to `1/2 + sqrt(5)/2`             |

## ebnf grammar

```
Program := Definitions

Definitions := NEWLINE* (Definition NEWLINE+)* Definition?

Definition := NAME "(" (NAME ",") NAME? ")" "=" Exprs
            | NAME "=" Exprs

Exprs := (Expr ",")* Expr ","?

Expr := Store

Store := Store "->" NAME | Sum

Sum := Sum "+" Product
     | Sum "-" Product
     | Product

Product := Product "*" Unary
         | Product "/" Unary
         | Unary

Unary := "+" Unary
       | "-" Unary
       | "*" Unary
       | Juxtapose Power
       | Power

Juxtapose := Juxtapose PreJuxtapose | PreJuxtapose

Power := FnCall "^" Unary | FnCall

FnCall := NAME "(" Exprs ")" | Item

PreJuxtapose := Number | "(" <Expr> ")"

Item := Number
      | NAME
      | "(" Expr ")"
	| "{" Exprs "}"
      | "sum"  "(" NAME ":" INT "," INT ")" "{" Exprs "}"
      | "prod" "(" NAME ":" INT "," INT ")" "{" Exprs "}"
      | "iter" "(" INT "," NAME ":" Expr ")" "{" Exprs "}"

Number = FLOAT | INT
```
