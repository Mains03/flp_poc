# flp_poc

Proof of concept for a functional logic programming language. In particular, the language has an `exists` keyword and uses equational constraints to restrict the possible values a variable bound to an `exists` statement could be.

## Syntax

A program consists of a sequence of declarations which is either a function type, a function definition, or a statement ending with a period. Periods are required so that whitespace is not needed in the syntax, that is so that indentation does not matter unlike other functional languages.

A function type has the following syntax:

```
[identifier] :: [type]
```

A function has the following syntax:

```
[identifier] [arg 1] ... [arg n] = [stm].
```

where the arguments are either identifiers or pairs of arguments - `([arg], [arg])`.

### Types

The following are all the values:
 - natural numbers - `Nat`,
 - pairs - `(a,b)`,
 - lists - `[...]`,
 - functions `A -> B`.

Polymorphic types are supported.

### Statements

A statement is one of the following:

 - if statement - `if [stm] then [stm] else [stm]`,
 - let statement - `let [identifier] = [stm] in [stm]`,
 - exists statement - `exists [identifier] :: [type]. [stm]`,
 - equate statement - `[expr] =:= [expr]. [stm]`,
 - choice statement - `[expr] <> ... <> [expr]`,
 - case statement - `case [identifier] of [case 1] -> [expr]. [case 2] -> [expr]. ... . [case n] -> [expr]`

where choice can be repeated any number of times and case runs over all the possible cases (`Zero`, `Succ n` for natural numbers and `[]`, `(x:xs)` for lists). See the example programs below for clarification on the syntax.

### Expressions

An expression is one of the following:

 - cons - `[expr] : [expr]`,
 - add - `[expr] + [expr]`,
 - app - `[expr] [expr]`,
 - boolean expressions - `[expr] [bexpr op] [expr]`,
 - lambda - `\[identifier]. [stm]`

where `bexpr op` is one of the following:

 - `==`, `!=`, `&&`, `||`.

Note you might need two backslashes when defining a lambda expression.

## Equational Constraints

To understand how equational constraints are used, we'll show some example programs.

```
exists n :: Nat. n =:= 1. n.
```

This program will terminate with `n` set to the value `1` (or `Succ Zero` equivalently) since `n =:= 1` restricts `n` to only be `1`. A more interesting program is the following:

```
add :: Nat -> Nat -> Nat
add n m = case n of
  Zero -> m.
  (Succ n) -> Succ (add n m).

exists n :: Nat. add n n =:= 2. n.
```

It sill terminates with `n` set to the value `1` however clearly the way this is reached is not so trivial. Such a program can be used to define a division by two function.

## Example Programs

The following program determines the last element in a list.

```
concat :: [a] -> [a] -> [a]
concat xs ys = case xs of
  [] -> ys.
  (x:xs) -> x : (concat xs ys).

last :: [a] -> a
last xs = exists ys :: [a]
  exists y :: a.
  concat ys [y] =:= xs.
  y.
```

The following program splits a list in half such that the two returned lists differ in length by at most one. Note we use choice in the let statement to deal with the case where the two lists are not the same length.

```
concat :: [a] -> [a] -> [a]
concat xs ys = case xs of
  [] -> ys.
  (x:xs) -> x : (concat xs ys).

length :: [a] -> a
length xs = case xs of
  [] -> 0.
  (x:xs) -> 1 + (length xs).

split :: [a] -> ([a], [a])
split xs = exists ys :: [a].
  exists zs :: [a].
  concat ys zs =:= xs.
  let zs_len = length zs in
  let ys_len = zs_len <> zs_len + 1 in
  length ys =:= ys_len.
  (ys, zs).
```
