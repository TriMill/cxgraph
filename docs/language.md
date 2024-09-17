# CXGraph language

CXGraph uses a custom expression language that is compiled to WGSL.

## Names

Names must begin with any alphabetic character (lowercase or capital letters, 
Greek letters, etc.) and may contain alphanumeric chararcters as well as
underscores (`_`) and apostrophes (`'`). The words `sum`, `prod`, `iter`,
and `if` may not be used for names. Names may refer to either functions
or variables.

Examples of names include:
```
a  A  aaa  ω  z_3  __5__  f'  К'םαl'けx焼__검
```

Names may either be **built-in**, **global**, or **local**. global or local names
may shadow built-in names, and local names may shadow global ones.

## Declarations

A **function declaration** declares a new function. Functions may have zero
or more arguments.

```
f(x) = 3
```

A **constant declaration** declares a new constant.

```
n = 5
```

Declarations are separated by newlines. Declarations may only reference functions
and constants in declarations that precede them. The name used in a declaration
(`f` and `n` in the above examples) is in the global scope.

The `plot` function is special and serves as the entry point. It must exist and have exactly one argument.

## Operators

Below is a reference to all operators in the CXGraph language.

| operator | description                | precedence |
|----------|----------------------------|------------|
| `,`      | separate expressions       | 0          |
| `->`     | assign to                  | 1          |
| `==`     | equal                      | 2          |
| `!=`     | not equal                  | 2          |
| `>`      | real part greater          | 3          |
| `<`      | real part less             | 3          |
| `>=`     | real part greater or equal | 3          |
| `<=`     | real part less or equal    | 3          |
| `+`      | addition                   | 4          |
| `-`      | subtraction                | 4          |
| `*`      | multiplication             | 5          |
| `/`      | division                   | 5          |
| `^`      | power                      | 6          |

The comma `,` separates expressions in locations where multiple are allowed (ie.
in a definition or block).

The arrow `->` stores the value of the expression to its left to the local
variable on the right.

The equality operators `==` and `!=` compare two values, considering both their
real and imaginary components. The comparison operators `>`, `<`, `>=`, and `<=`
only consider the real component. These all produce `0` if the equality or comparison
is false and `1` if it is true.

`+`, `-`, and `*` also function as the unary plus, minus, and conugation operators.

Multiplication can also be done via juxtaposition - `2x(x+1)` is equivalent to `2*x*(x+1)`.
Juxtaposition takes precedence over other operations, so `1/2x` is `1/(2*x)`, not `1/2*x`,
and `z^5(x+1)` is `z^(5*(x+1))`, not `z^5*(x+1)`.

## Grouping

Parentheses `( )` can be used to group within an expression. Braces `{ }` can be used to group multiple expressions, separated by commas. Newlines, which ordinarily separate declarations, are treated as whitespace between grouping symbols, allowing for multiline definitions.

## Repetition and conditionals

To allow for finite sums and products, the `sum` and `prod` expressions can be used.

```
plot(z) = sum(n: 0, 20) { z^n / n }
```

The first parameter to `sum` or `prod` is the name of the summation index, the next two
values are the lower and upper bounds (inclusive). These may be arbitrary expressions,
they are converted to integers by rounding down the real component. The value of the body
for each index value is summed or multiplied and the result is returned.

`iter` works similarly:

```
plot(c) = iter(20, 0 -> z) { z^2 + c }
```

The first parameter is the number of iterations, the second is an assignment to initialize
the iteration variable. For each iteration, the iteration variable will be updated to the
new value of the body.

`if` can be used to choose between two expressions based on a condition:

```
if(z > 0) { z^2 } { 2z }
```

If the argument's real part is positive, the first body will be evaluated, and otherwise the
second will be.

`while` can be used to repeat while a condition is met. Care should be taken to ensure the loop will always end eventually.

```
0 -> n, while(n < 10) { n + 1 -> n }
```


## Built-in functions and constants

arithmetic functions:
| name          | description                                           |
|---------------|-------------------------------------------------------|
| `pos(z)`      | identity, equivalent to unary `+`                     |
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
| name           | description                               |
|----------------|-------------------------------------------|
| `exp(z)`       | Exponential function, equivalent to `e^z` |
| `log(z)`       | Natural logarithm                         |
| `log2(z)`      | Logarithm base 2                          |
| `log10(z)`     | Logarithm base 10                         |
| `logb(b,z)`    | Logarithm base b                          |
| `logbr(z,br)`  | Natural logarithm with specified branch   |
| `pow(z)`       | Power, equivalent to `^`                  |
| `powbr(z,br)`  | `pow` with specified branch               |
| `sqrt(z)`      | square root, equivalent to `z^0.5`        |
| `sqrtbr(z,br)` | square root with specified branch         |
| `cbrt(z)`      | cube root, equivalent to `z^0.5`          |
| `cbrtbr(z,br)` | cube root with specified branch           |

trigonometric functions:
| name       | description                         |
|------------|-------------------------------------|
| `sin(z)`   | Sine function                       |
| `cos(z)`   | Cosine function                     |
| `tan(z)`   | Tangent function                    |
| `sinh(z)`  | Hyperbolic sine function            |
| `cosh(z)`  | Hyperbolic cosine function          |
| `tanh(z)`  | Hyperbolic tangent function         |
| `asin(z)`  | Inverse sine function               |
| `acos(z)`  | Inverse cosine function             |
| `atan(z)`  | Inverse tangent function            |
| `asinh(z)` | Inverse hyperbolic sine function    |
| `acosh(z)` | Inverse hyperbolic cosine function  |
| `atanh(z)` | Inverse hyperbolic tangent function |

special functions:
| function                 | description                                                            |
|--------------------------|------------------------------------------------------------------------|
| `gamma(z)`, `Γ(z)`       | [gamma function](https://en.wikipedia.org/wiki/Gamma_function)         |
| `invgamma(z)`, `invΓ(z)` | reciprocal of the gamma function                                       |
| `loggamma(z)`, `logΓ(z)` | logarithm of the gamma function                                        |
| `digamma(z)`, `ψ(z)`     | [digamma function](https://en.wikipedia.org/wiki/Digamma_function)     |
| `lambertw(z)`            | [Lambert W function](https://en.wikipedia.org/wiki/Lambert_W_function) |
| `lambertwbr(z,br)`       | Lambert W function on specfied branch                                  |
| `erf(z)`                 | The [Error function](https://en.wikipedia.org/wiki/Error_function)     |

logic functions:
| function    | description                                                                |
|-------------|----------------------------------------------------------------------------|
| `signre(z)` | Sign of real part (1 if `re(z) > 0`, -1 if `re(z) < 0`, 0 if `re(z) == 0`) |
| `signim(z)` | Sign of imaginary part                                                     |
| `absre(z)`  | Absolute value of real part                                                |
| `absim(z)`  | Absolute value of imaginary part                                           |
| `isnan(z)`  | 1 if `z` is NaN, 0 otherwise                                               |

other functions:
| function     | description                      |
|--------------|----------------------------------|
| `mix(u,v,a)` | `u*(1-a) + v*a`                  |

constants:
| name           | description                                                                                            |
|----------------|--------------------------------------------------------------------------------------------------------|
| `i`            | The imaginary constant, equal to `sqrt(-1)`                                                            |
| `e`            | The [exponential constant](https://en.wikipedia.org/wiki/E_(mathematical_constant)), equal to `exp(1)` |
| `tau`, `τ`     | The [circle constant](https://tauday.com/tau-manifesto)                                                |
| `emgamma`, `γ` | The [Euler-Mascheroni](https://en.wikipedia.org/wiki/Euler%27s_constant) constant, equal to `-ψ(1)`    |
| `phi`, `φ`     | The [golden ratio](https://en.wikipedia.org/wiki/Golden_ratio), equal to `1/2 + sqrt(5)/2`             |

## ebnf grammar

```
Program := Definitions

Definitions := NEWLINE* (Definition NEWLINE+)* Definition?

Definition := NAME "(" (NAME ",") NAME? ")" "=" Exprs
            | NAME "=" Exprs

Exprs := (Expr ",")* Expr ","?

Expr := Store

Store := Equality "->" NAME | Equality

Equality := Compare "==" Compare
          | Compare "!=" Compare
          | Compare

Compare := Sum ">" Sum
         | Sum "<" Sum
         | Sum ">=" Sum
         | Sum "<=" Sum
         | Sum

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

PreJuxtapose := NUMBER | "(" <Expr> ")"

Block := "{" Exprs "}"

Item := NUMBER
      | NAME
      | "(" Expr ")"
      | Block
      | "sum"  "(" NAME ":" Expr "," Expr ")" Block
      | "prod" "(" NAME ":" Expr "," Expr ")" block
      | "iter" "(" Expr "," Expr "->" NAME ")" Block
      | "if" "(" Expr ")" Block Block
```
