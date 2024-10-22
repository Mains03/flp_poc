# flp_poc

Proof of concept for a functional logic programming language. In particular, the language has an `exists` keyword and uses equational constraints to restrict the possible values a variable bound to an `exists` statement could be.

## Syntax

### Statements

Todo.

### Expressions

Todo.

## Exists Keyword

The syntax is the following:

```
exists [var] :: [type]. [stm]
```

where the type must be a 'first-order type', that is essentially not a function type.

## Equational Constraints

The syntax is the following:

```
[expr] =:= [expr]. [stm]
```

Consider the following program:

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

It sill terminates with `n` set to the value `1` however clearly the way this is reached is not so trivial.

## Example Programs

The following program can be used to determine the last element in a list.

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

The following program can be used to split a list in half where the two returned lists differ in length by at most one. 

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
